use {
    crate::generator::{generate_ctx_arg, Generator},
    heck::ToUpperCamelCase,
    proc_macro2::TokenStream,
    quote::{format_ident, quote},
    syn::{Ident, Type},
    typhoon_syn::{Context, InstructionAccount, InstructionArg, InstructionReturnData},
};

pub struct CpiGenerator;

fn generate_ctx(ctxs: &hashbrown::HashMap<String, Context>) -> TokenStream {
    let tokens = ctxs.values().map(|ctx| {
        let name = &ctx.name;
        let ctx_name = format_ident!("{}Context", name);
        let (args_field, args_assign) = generate_ctx_arg(ctx, generate_arg);
        let (acc_fields, metas, infos) = generate_accounts(&ctx.accounts);
        let arg_writer = args_assign.map(|el| quote!(writer.write_bytes(#el)?;));
        let has_optional = ctx.accounts.iter().any(|acc| acc.meta.is_optional);
        let program_field = if has_optional {
            Some(quote!(program: &'a AccountView,))
        } else {
            None
        };
        quote! {
            pub struct #ctx_name<'a> {
                #(#acc_fields)*
                #args_field
            }

            impl<'a> #ctx_name<'a> {
                #[inline(always)]
                fn append(
                    &self,
                    #program_field
                    writer: &mut bytes::MaybeUninitWriter,
                    metas: &mut [core::mem::MaybeUninit<instruction::InstructionAccount<'a>>],
                    infos: &mut [core::mem::MaybeUninit<&'a AccountView>],
                ) -> ProgramResult {
                    #arg_writer

                    for ((meta_dest, info_dest), (meta_src, info_src)) in metas
                        .iter_mut()
                        .zip(infos.iter_mut())
                        .zip(core::iter::zip([#(#metas),*], [#(#infos),*]))
                    {
                        meta_dest.write(meta_src);
                        info_dest.write(info_src);
                    }

                    Ok(())
                }
            }
        }
    });

    quote!(#(#tokens)*)
}

fn generate_arg((name, ty): (&Ident, &Type)) -> (TokenStream, TokenStream) {
    (
        quote!(pub #name: &'a #ty,),
        quote!(bytemuck::bytes_of(self.#name)),
    )
}

fn generate_accounts(
    accounts: &[InstructionAccount],
) -> (Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>) {
    let len = accounts.len();
    let mut account_fields = Vec::with_capacity(len);
    let mut metas = Vec::with_capacity(len);
    let mut infos = Vec::with_capacity(len);

    for acc in accounts {
        let name = &acc.name;
        let is_optional = acc.meta.is_optional;
        let is_mutable = acc.meta.is_mutable;
        let is_signer = acc.meta.is_signer;

        let field = if is_optional {
            quote!(pub #name: Option<&'a AccountView>,)
        } else {
            quote!(pub #name: &'a AccountView,)
        };
        let meta = if is_optional {
            quote! {
                if let Some(#name) = self.#name {
                    instruction::InstructionAccount::new(#name.address(), #is_mutable, #is_signer)
                }else {
                    instruction::InstructionAccount::new(program.address(), false, false)
                }
            }
        } else {
            quote!(instruction::InstructionAccount::new(self.#name.address(), #is_mutable, #is_signer))
        };

        let info = if is_optional {
            quote!(self.#name.unwrap_or(program))
        } else {
            quote!(self.#name)
        };

        account_fields.push(field);
        metas.push(meta);
        infos.push(info);
    }

    (account_fields, metas, infos)
}

impl Generator for CpiGenerator {
    fn generate_token(
        instructions: &hashbrown::HashMap<usize, typhoon_syn::Instruction>,
        context: &hashbrown::HashMap<String, typhoon_syn::Context>,
        extra_token: TokenStream,
    ) -> TokenStream {
        let mut token = TokenStream::new();

        token.extend(generate_ctx(context));
        instructions.iter().for_each(|(discriminator, ix)| {
            let instruction_name =
                format_ident!("{}Cpi", ix.name.to_string().to_upper_camel_case());
            let dis = *discriminator as u8;
            let (result_ty, return_data) = if let InstructionReturnData { ty: Some(ref ty), .. } = ix.return_data {
                (
                    Some(quote!(<#ty>)),
                    quote! {
                        bytemuck::pod_read_unaligned(
                            &cpi::get_return_data().ok_or(ErrorCode::InvalidReturnData)?.as_slice(),
                        )
                    },
                )
            } else {
                (None, quote!(()))
            };
            let mut data_len = Vec::new();
            let mut accumulated_len = 0;
            let mut  has_optional = false;
            let mut fields = Vec::with_capacity(ix.args.len());
            let mut assigns = Vec::with_capacity(ix.args.len());
            for (arg_name, v) in &ix.args {
                match v {
                    InstructionArg::Type { ty, .. } => {
                        let (field, bytes) = generate_arg((arg_name, ty));
                        data_len.push(quote!(core::mem::size_of::<#ty>()));
                        fields.push(field);
                        assigns.push(quote!(writer.write_bytes(#bytes)?;));
                    }
                    InstructionArg::Context(ctx_name) => {
                        if let Some(ctx) = context.get(&ctx_name.to_string()) {
                            let ctx_has_optional = ctx.accounts.iter().any(|acc| acc.meta.is_optional);
                            if ctx_has_optional {
                                has_optional = true;
                            }

                            let program_arg = ctx_has_optional.then(|| quote!(self.program,));
                            let ctx_struct = format_ident!("{ctx_name}Context");
                            let acc_len = ctx.accounts.len();
                            let new_len = accumulated_len + acc_len;
                            assigns.push(quote!(self.#arg_name.append(#program_arg &mut writer, &mut metas[#accumulated_len..#new_len], &mut infos[#accumulated_len..#new_len])?;));
                            accumulated_len = new_len;
                            fields.push(quote!(pub #arg_name: #ctx_struct<'a>,));
                        } else {
                            let compile_error = syn::Error::new_spanned(
                                ctx_name,
                                format!(
                                    "Context '{}' not found. Ensure it's defined with #[context]",
                                    ctx_name
                                ),
                            )
                            .to_compile_error();
                            assigns.push(compile_error);
                        }
                    }
                }
            }
            let (program_id_field, program_id_getter) = if has_optional {
                (quote!(&'a AccountView), Some(quote!(.address())))
            } else {
                (quote!(&'a Address), None)
            };

            token.extend(quote! {
                pub struct #instruction_name<'a> {
                    #(#fields)*
                    pub program: #program_id_field,
                }

                impl #instruction_name<'_> {
                    #[inline(always)]
                    pub fn invoke(&self) -> ProgramResult #result_ty {
                        self.invoke_signed(&[])
                    }

                    #[inline(always)]
                    pub fn invoke_signed(&self, seeds: &[CpiSigner]) -> ProgramResult #result_ty {
                        let mut bytes = [bytes::UNINIT_BYTE; 1 #(+ #data_len)*];
                        let mut metas = [bytes::UNINIT_INS_ACC; #accumulated_len];
                        let mut infos = [bytes::UNINIT_ACC_VIEW; #accumulated_len];
                        let mut writer = bytes::MaybeUninitWriter::new(&mut bytes, 0);
                        writer.write_bytes(&[#dis])?;

                        #(#assigns)*

                        let instruction = instruction::InstructionView {
                            program_id:  self.program #program_id_getter,
                            data: writer.initialized(),
                            accounts: unsafe { core::slice::from_raw_parts(metas.as_ptr() as *const _, #accumulated_len) }
                        };

                        cpi::invoke_signed(
                            &instruction,
                            unsafe { &*(infos.as_ptr() as *const [&AccountView; #accumulated_len]) },
                            seeds
                        )?;

                        Ok(#return_data)
                    }
                }
            });
        });

        token.extend(extra_token);
        token
    }
}
