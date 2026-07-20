use {
    crate::{
        anchor::{gen_docs, gen_type_ref},
        idl::{Field, Instruction, InstructionAccountItem},
    },
    heck::ToUpperCamelCase,
    proc_macro2::{Span, TokenStream},
    quote::{format_ident, quote},
    syn::{Expr, Ident},
};

pub fn gen_instructions(ixs: &[Instruction]) -> TokenStream {
    let instructions = ixs.iter().map(|instruction| {
        let ident = format_ident!("{}", instruction.name.to_upper_camel_case());
        let (metas, accounts) = gen_account_instruction(&instruction.accounts);
        let docs = gen_docs(&instruction.docs);

        let account_metas = gen_account_metas(&metas);
        let discriminator = &instruction.discriminator.value();
        let (arg_fields, instruction_data) =
            gen_instruction_data(&instruction.args, discriminator);
        let len = metas.len();

        quote! {
            /// Used for Cross-Program Invocation (CPI) calls.
            #docs
            pub struct #ident<'a> {
                #(pub #accounts: &'a AccountView,)*
                #(#arg_fields)*
            }

            impl #ident<'_> {
                #[inline(always)]
                pub fn invoke(&self) -> ProgramResult {
                    self.invoke_signed(&[])
                }

                pub fn invoke_signed(&self, seeds: &[CpiSigner]) -> ProgramResult {
                    #account_metas
                    #instruction_data

                    cpi::invoke_signed(
                        &instruction,
                        &[#(self.#accounts),*],
                        seeds
                    ).map_err(Into::into)
                }

                #[inline(always)]
                pub fn invoke_with_remaining<'a>(&self, remaining: impl ExactSizeIterator<Item = &'a AccountView>) -> ProgramResult {
                    self.invoke_signed_with_remaining(&[], remaining)
                }

                pub fn invoke_signed_with_remaining<'a>(&self, seeds: &[CpiSigner], remaining: impl ExactSizeIterator<Item = &'a AccountView>) -> ProgramResult {
                    let accounts_len: usize = core::cmp::min(remaining.len() + #len, 64);
                    let mut account_infos = [bytes::UNINIT_ACC_VIEW; 64];
                    let mut account_metas = [bytes::UNINIT_INS_ACC; 64];

                    for (d, s) in account_metas[..#len].iter_mut().zip([#(#metas),*]) {
                        d.write(s);
                    }

                    for (d, s) in account_infos[..#len].iter_mut().zip([#(self.#accounts),*]) {
                        d.write(s);
                    }

                    for (i, account) in remaining.enumerate() {
                        account_metas[#len + i].write(instruction::InstructionAccount::new(account.address(), account.is_writable(), account.is_signer()));
                        account_infos[#len + i].write(account);
                    }

                    let account_metas =  unsafe { core::slice::from_raw_parts(account_metas.as_ptr() as _, accounts_len) };
                    #instruction_data

                    cpi::invoke_signed_with_slice(
                        &instruction,
                        unsafe { core::slice::from_raw_parts(account_infos.as_ptr() as *const &AccountView, accounts_len) },
                        seeds
                    ).map_err(Into::into)
                }
            }
        }
    });

    quote! {
        #(#instructions)*
    }
}

fn gen_instruction_data(args: &[Field], discriminator: &[u8]) -> (Vec<TokenStream>, TokenStream) {
    let discriminator_len = discriminator.len();
    let discriminator_expr: Expr = syn::parse_quote!([#(#discriminator),*]);
    let mut arg_fields = Vec::with_capacity(args.len());
    let mut arg_sizes = Vec::with_capacity(args.len());
    let mut arg_ser = Vec::with_capacity(args.len());
    for arg in args {
        let ident = Ident::new(&arg.name, Span::call_site());
        let ty_ref = gen_type_ref(&arg.ty);
        arg_fields.push(quote!(pub #ident: #ty_ref,));
        arg_sizes.push(quote!(BorshStrategy::<false>::size(&self.#ident)?));
        arg_ser.push(quote!(BorshStrategy::<false>::write_into(&mut writer, &self.#ident)?;));
    }

    let instruction_data = if arg_ser.is_empty() {
        quote! {
            let instruction_data = #discriminator_expr;

            let instruction = instruction::InstructionView {
                program_id: &PROGRAM_ID,
                accounts: &account_metas,
                data: &instruction_data,
            };
        }
    } else {
        quote! {
            let mut instruction_data = [bytes::UNINIT_BYTE; 1232];
            unsafe {
                let ptr = instruction_data.as_mut_ptr() as *mut u8;
                core::ptr::copy_nonoverlapping(#discriminator_expr.as_ptr(), ptr, #discriminator_len);
            }

            let data_len = #discriminator_len #(+ #arg_sizes)*;
            let mut writer = &mut instruction_data[#discriminator_len..];
            #(#arg_ser)*

            let instruction = instruction::InstructionView {
                program_id: &PROGRAM_ID,
                accounts: &account_metas,
                data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, data_len) },
            };
        }
    };

    (arg_fields, instruction_data)
}

fn gen_account_instruction(
    accounts: &[InstructionAccountItem],
) -> (Vec<TokenStream>, Vec<syn::Ident>) {
    let mut metas = Vec::with_capacity(accounts.len());
    let mut fields = Vec::with_capacity(accounts.len());

    for account in accounts {
        match account {
            InstructionAccountItem::Composite(composite_accounts) => {
                let (nested_metas, nested_fields) =
                    gen_account_instruction(&composite_accounts.accounts);
                metas.extend(nested_metas);
                fields.extend(nested_fields);
            }
            InstructionAccountItem::Single(account) => {
                let ident = format_ident!("{}", &account.name);
                let is_writable = account.is_mut;
                let is_signer = account.is_signer;

                metas.push(quote! {
                    instruction::InstructionAccount::new(self.#ident.address(), #is_writable, #is_signer)
                });
                fields.push(ident);
            }
        }
    }

    (metas, fields)
}

#[inline]
fn gen_account_metas(metas: &[TokenStream]) -> TokenStream {
    let len = metas.len();

    quote! {
        let account_metas: [instruction::InstructionAccount; #len] = [#(#metas),*];
    }
}

#[cfg(test)]
mod tests {
    use {super::*, crate::idl::InstructionAccount};

    #[test]
    fn test_gen_instruction_data() {
        let args = vec![];
        let discriminator = vec![1, 2, 3, 4];

        let (fields, data) = gen_instruction_data(&args, &discriminator);
        let expected_data = quote! {
            let instruction_data = [1u8, 2u8, 3u8, 4u8];

            let instruction = instruction::InstructionView {
                program_id: &PROGRAM_ID,
                accounts: &account_metas,
                data: &instruction_data,
            };
        };
        assert!(fields.is_empty());
        assert_eq!(data.to_string(), expected_data.to_string());

        let args = vec![Field {
            docs: vec![],
            name: "amount".to_string(),
            ty: crate::idl::Type::U64,
        }];
        let discriminator = vec![1, 2, 3, 4];

        let (fields, data) = gen_instruction_data(&args, &discriminator);
        let expected_data = quote! {
            let mut instruction_data = [bytes::UNINIT_BYTE; 1232];
            unsafe {
                let ptr = instruction_data.as_mut_ptr() as *mut u8;
                core::ptr::copy_nonoverlapping([1u8, 2u8, 3u8, 4u8].as_ptr(), ptr, 4usize);
            }
            let data_len = 4usize + BorshStrategy::<false>::size(&self.amount)?;
            let mut writer = &mut instruction_data[4usize..];
            BorshStrategy::<false>::write_into(&mut writer, &self.amount)?;

            let instruction = instruction::InstructionView {
                program_id: &PROGRAM_ID,
                accounts: &account_metas,
                data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, data_len) },
            };
        };

        assert_eq!(fields.len(), 1);
        assert_eq!(data.to_string(), expected_data.to_string());
    }

    #[test]
    fn test_gen_account_instruction() {
        let accounts = vec![
            InstructionAccountItem::Single(InstructionAccount {
                name: "test_account".to_string(),
                is_mut: true,
                is_signer: false,
            }),
            InstructionAccountItem::Single(InstructionAccount {
                name: "test_account2".to_string(),
                is_mut: false,
                is_signer: true,
            }),
        ];

        let (metas, fields) = gen_account_instruction(&accounts);

        let result = quote! {
            #(#metas),*
        };
        let expected = quote! {
            instruction::InstructionAccount::new(self.test_account.address(), true, false),
            instruction::InstructionAccount::new(self.test_account2.address(), false, true)
        };

        assert_eq!(result.to_string(), expected.to_string());
        assert_eq!(fields[0].to_string(), "test_account");
        assert_eq!(fields[1].to_string(), "test_account2");
    }

    #[test]
    fn test_gen_account_metas() {
        let metas = vec![quote!(meta1), quote!(meta2)];
        let result = gen_account_metas(&metas);
        let expected = quote! {
            let account_metas: [instruction::InstructionAccount; 2usize] = [meta1, meta2];
        };

        assert_eq!(result.to_string(), expected.to_string());
    }
}
