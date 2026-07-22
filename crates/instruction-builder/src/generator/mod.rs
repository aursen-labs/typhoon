mod client;
mod cpi;

pub use {client::*, cpi::*};
use {
    hashbrown::HashMap,
    proc_macro2::TokenStream,
    quote::format_ident,
    syn::{parse_quote, Ident, Type},
    typhoon_syn::{Arguments, Context, Instruction},
};

pub trait Generator {
    fn generate_token(
        instructions: &HashMap<usize, Instruction>,
        context: &HashMap<String, Context>,
        extra_token: TokenStream,
    ) -> TokenStream;
}

fn generate_ctx_arg(
    ctx: &Context,
    gen_arg: fn((&Ident, &Type)) -> (TokenStream, TokenStream),
) -> (Option<TokenStream>, Option<TokenStream>) {
    ctx.arguments
        .as_ref()
        .map(|args| {
            let arg_ty = match args {
                Arguments::Values(_) => format_ident!("{}Args", ctx.name),
                Arguments::Struct(ident) => ident.clone(),
            };
            gen_arg((&format_ident!("args"), &parse_quote!(#arg_ty)))
        })
        .unzip()
}
