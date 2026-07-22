use syn::{Attribute, Expr, ExprLit, Lit};

pub fn parse_docs(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .filter_map(|attr| {
            if let syn::Meta::NameValue(v) = &attr.meta {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(str_lit),
                    ..
                }) = &v.value
                {
                    return Some(str_lit.value().trim().to_string());
                }
            }
            None
        })
        .collect()
}
