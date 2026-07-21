use {
    crate::context::ParsingContext,
    generators::*,
    injector::FieldInjector,
    proc_macro::TokenStream,
    proc_macro2::TokenStream as TokenStream2,
    quote::{format_ident, quote, ToTokens},
    sorter::sort_accounts,
    syn::{
        parse_macro_input, parse_quote, visit_mut::VisitMut, Attribute, Field, Ident, ItemStruct,
    },
};

mod context;
mod generators;
mod injector;
mod remover;
mod sorter;
mod visitor;

#[proc_macro_attribute]
pub fn context(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let context = parse_macro_input!(item as ParsingContext);
    let generator = match TokenGenerator::new(context) {
        Ok(gen) => gen,
        Err(err) => return TokenStream::from(err.into_compile_error()),
    };

    TokenStream::from(generator.into_token_stream())
}

type BumpsStruct = (ItemStruct, TokenStream2);

struct TokenGenerator {
    item_struct: ItemStruct,
    accounts_token: Vec<TokenStream2>,
    bumps: Option<BumpsStruct>,
    args: Option<(Ident, Option<TokenStream2>)>,
    needs_rent: bool,
}

impl TokenGenerator {
    pub fn new(mut context: ParsingContext) -> Result<Self, syn::Error> {
        sort_accounts(&mut context)?;

        let global_context = GlobalContext::from_parsing_context(&context)?;

        for program in &global_context.program_checks {
            if context.accounts.iter().all(|el| el.inner_ty != *program) {
                return Err(syn::Error::new_spanned(
                    &context.item_struct,
                    format!("One constraint requires including the `Program<{program}>` account."),
                ));
            }
        }

        let bumps = global_context.generate_bumps(&context);
        let args = global_context.generate_args(&context);

        let accounts_token = global_context
            .accounts
            .into_iter()
            .map(|acc| acc.generate())
            .collect::<Result<_, _>>()?;

        Ok(TokenGenerator {
            needs_rent: global_context.need_rent,
            item_struct: context.item_struct,
            accounts_token,
            bumps,
            args,
        })
    }
}

impl ToTokens for TokenGenerator {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.item_struct.ident;
        let generics = &self.item_struct.generics;

        let (_, ty_generics, _) = generics.split_for_impl();

        // patch the lifetime of the new context here
        let generics = &mut generics.to_owned();
        generics.params.push(parse_quote!('c));
        if let Some(where_clause) = &mut generics.where_clause {
            where_clause.predicates.push(parse_quote!('c: 'info));
        } else {
            generics.where_clause = Some(parse_quote!(where 'c: 'info));
        }
        let (impl_generics, _, where_clause) = generics.split_for_impl();

        let name_list: Vec<&Ident> = self
            .item_struct
            .fields
            .iter()
            .filter_map(|f| f.ident.as_ref())
            .collect();
        let accounts_token = &self.accounts_token;
        let (bumps_struct, bumps_var) = self.bumps.clone().unzip();

        let mut struct_fields: Vec<&Ident> = name_list.clone();

        let account_struct = &mut self.item_struct.to_owned();

        let bumps_ident = format_ident!("bumps");
        if let Some(ref bumps) = bumps_struct {
            let name = &bumps.ident;
            let bumps_field: Field = parse_quote!(pub #bumps_ident: #name);
            struct_fields.push(&bumps_ident);
            FieldInjector::new(bumps_field).visit_item_struct_mut(account_struct);
        }

        let args_ident = format_ident!("args");
        let (args_assign, args_struct) = self.args.as_ref().map(|(name, args_struct)| {
            let args_field: Field = parse_quote!(pub #args_ident: &'info #name);
            struct_fields.push(&args_ident);
            FieldInjector::new(args_field).visit_item_struct_mut(account_struct);
            let args_assign = quote!(let (Arg(args), _) = Arg::<#name>::from_entrypoint(program_id, &mut [], instruction_data)?;);

            (args_assign, args_struct)
        }).unzip();

        let rent = self
            .needs_rent
            .then_some(quote!(let rent = <Rent as Sysvar>::get()?;));

        let impl_context = quote! {
            impl #impl_generics HandlerContext<'_, 'info, 'c> for #name #ty_generics #where_clause {
                #[inline(always)]
                fn from_entrypoint(
                    program_id: &Address,
                    accounts: &'info mut [AccountView],
                    instruction_data: &mut &'c [u8],
                ) -> ProgramResult<(Self, &'info mut [AccountView])> {
                    let [#(#name_list,)* rem @ ..] = accounts else {
                        return Err(ProgramError::NotEnoughAccountKeys.into());
                    };

                    #args_assign
                    #rent

                    #(#accounts_token)*

                    #bumps_var

                    Ok((#name { #(#struct_fields),* }, rem))
                }
            }

            impl #impl_generics Context for #name #ty_generics #where_clause {}
        };

        let doc = prettyplease::unparse(
            &syn::parse2::<syn::File>(quote! {
                #bumps_struct
                #args_struct

                #impl_context
            })
            .unwrap(),
        );

        let mut doc_attrs: Vec<Attribute> = parse_quote! {
            /// # Generated
            /// ```ignore
            #[doc = #doc]
            /// ```
        };

        account_struct.attrs.append(&mut doc_attrs);

        let expanded = quote! {
            #bumps_struct
            #args_struct

            #account_struct

            #impl_context

        };
        expanded.to_tokens(tokens);
    }
}
