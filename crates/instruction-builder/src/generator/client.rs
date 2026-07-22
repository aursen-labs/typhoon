use {
    crate::generator::{generate_ctx_arg, Generator},
    heck::ToUpperCamelCase,
    proc_macro2::TokenStream,
    quote::{format_ident, quote},
    syn::{Ident, Type},
    typhoon_syn::{Arguments, Context, Instruction, InstructionAccount, InstructionArg},
};

pub struct ClientGenerator;

fn generate_ctx(ctxs: &hashbrown::HashMap<String, Context>) -> TokenStream {
    let tokens = ctxs.values().map(|ctx| {
        let name = &ctx.name;
        let ctx_name = format_ident!("{}Context", name);
        let (args_field, args_assign) = generate_ctx_arg(ctx, generate_arg);
        let (acc_fields, acc_assigns) = generate_accounts(&ctx.accounts);

        quote! {
            pub struct #ctx_name {
                #(#acc_fields)*
                #args_field
            }

            impl #ctx_name {
                fn append(&self, data: &mut std::vec::Vec<u8>, accounts: &mut std::vec::Vec<::solana_instruction::AccountMeta>) {
                    #(#acc_assigns)*
                    #args_assign
                }
            }
        }
    });

    quote!(#(#tokens)*)
}

fn generate_arg((name, ty): (&Ident, &Type)) -> (TokenStream, TokenStream) {
    (
        quote!(pub #name: #ty,),
        quote!(data.extend_from_slice(bytemuck::bytes_of(&self.#name));),
    )
}

fn generate_accounts(accounts: &[InstructionAccount]) -> (Vec<TokenStream>, Vec<TokenStream>) {
    accounts
        .iter()
        .map(|acc| {
            let name = &acc.name;
            let is_signer = acc.meta.is_signer;
            let field = if acc.meta.is_optional {
                quote!(pub #name: Option<::solana_address::Address>,)
            } else {
                quote!(pub #name: ::solana_address::Address,)
            };

            let push = if acc.meta.is_optional {
                let meta = if acc.meta.is_mutable {
                    quote!(accounts.push(::solana_instruction::AccountMeta::new(#name, #is_signer));)
                }else {
                    quote!(accounts.push(::solana_instruction::AccountMeta::new_readonly(#name, #is_signer));)
                };
                quote! {
                    if let Some(#name) = self.#name {
                        #meta
                    }else {
                        accounts.push(::solana_instruction::AccountMeta::new_readonly(crate::ID.into(), false));
                    }
                }
            }else if acc.meta.is_mutable {
                quote!(accounts.push(::solana_instruction::AccountMeta::new(self.#name, #is_signer));)
            }else {
                quote!(accounts.push(::solana_instruction::AccountMeta::new_readonly(self.#name, #is_signer));)
            };

            (field, push)
        })
        .collect()
}

impl Generator for ClientGenerator {
    fn generate_token(
        instructions: &hashbrown::HashMap<usize, Instruction>,
        context: &hashbrown::HashMap<String, Context>,
        extra_token: TokenStream,
    ) -> TokenStream {
        let mut token = TokenStream::new();

        token.extend(generate_ctx(context));
        token.extend(extra_token);

        instructions.iter().for_each(|(discriminator, ix)| {
            let name = format_ident!("{}Instruction", ix.name.to_string().to_upper_camel_case());
            let mut data_len = Vec::new();
            let mut accounts_len = 0;
            let (fields, assigns): (Vec<_>, Vec<_>) = ix
                .args
                .iter()
                .map(|(arg_name, arg_v)| match arg_v {
                    InstructionArg::Type { ty, .. } => {
                        data_len.push(quote!(core::mem::size_of::<#ty>()));
                        generate_arg((arg_name, ty))
                    }
                    InstructionArg::Context(ident) => {
                        let arg_ty = format_ident!("{ident}Context");
                        if let Some(context) = context.get(&ident.to_string()) {
                            accounts_len += context.accounts.len();

                            if let Some(args) = &context.arguments {
                                let name = match args {
                                    Arguments::Values(_) => format_ident!("{ident}Args"),
                                    Arguments::Struct(struct_name) => struct_name.clone(),
                                };
                                data_len.push(quote!(core::mem::size_of::<#name>()));
                            }

                            (
                                quote!(pub #arg_name: #arg_ty,),
                                quote!(self.#arg_name.append(&mut data, &mut accounts);),
                            )
                        } else {
                            let compile_error = syn::Error::new_spanned(
                                ident,
                                format!(
                                    "Context '{}' not found. Ensure it's defined with #[context]",
                                    ident
                                ),
                            )
                            .to_compile_error();
                            (quote!(), compile_error)
                        }
                    }
                })
                .unzip();
            let dis: u8 = *discriminator as u8;

            token.extend(quote! {
                pub struct #name {
                    #(#fields)*
                }

                impl #name {
                    #[inline(always)]
                    pub fn into_instruction(self) -> ::solana_instruction::Instruction {
                        let mut data = std::vec::Vec::with_capacity(1 #(+ #data_len)*);
                        let mut accounts = std::vec::Vec::with_capacity(#accounts_len);

                        data.push(#dis);

                        #(#assigns)*

                        ::solana_instruction::Instruction {
                            program_id: crate::ID.into(),
                            accounts,
                            data
                        }
                    }
                }
            });
        });

        token
    }
}
