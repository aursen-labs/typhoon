use {
    cargo_manifest::Manifest,
    heck::ToUpperCamelCase,
    proc_macro::TokenStream,
    proc_macro2::Span,
    quote::{format_ident, quote, ToTokens},
    std::env::var,
    syn::{parse::Parse, parse_macro_input, Ident, LitStr},
};

#[proc_macro]
pub fn program_id(item: TokenStream) -> TokenStream {
    parse_macro_input!(item as ProgramId)
        .to_token_stream()
        .into()
}

struct ProgramId {
    pub name: Ident,
    pub id: String,
}

impl Parse for ProgramId {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let id: LitStr = input.parse()?;
        let name = generate_name()?;

        Ok(ProgramId {
            id: id.value(),
            name,
        })
    }
}

impl ToTokens for ProgramId {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let id = &self.id;
        let name = &self.name;

        quote! {
            pinocchio::address::declare_id!(#id);

            pub struct #name;

            impl CheckProgramId for #name {
                #[inline(always)]
                fn address_eq(program_id: &Address) -> bool {
                    pinocchio::address::address_eq(program_id, &crate::ID)
                }
            }
        }
        .to_tokens(tokens);
    }
}

fn get_cargo_toml() -> syn::Result<String> {
    let crate_dir = var("CARGO_MANIFEST_DIR")
        .map_err(|_| syn::Error::new(Span::call_site(), "Not in valid rust project."))?;

    Ok(format!("{crate_dir}/Cargo.toml"))
}

fn generate_name() -> syn::Result<Ident> {
    let cargo_toml = get_cargo_toml()?;
    let manifest = Manifest::from_path(cargo_toml)
        .map_err(|_| syn::Error::new(Span::call_site(), "Invalid Cargo.toml"))?;
    let package_section = manifest.package.ok_or(syn::Error::new(
        Span::call_site(),
        "Invalid package section",
    ))?;

    Ok(format_ident!(
        "{}Program",
        package_section.name.to_upper_camel_case()
    ))
}
