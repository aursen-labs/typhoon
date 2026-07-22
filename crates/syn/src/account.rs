use {
    crate::{constraints::Constraints, parse_docs},
    proc_macro2::Span,
    syn::{
        parse_quote,
        visit::{visit_path_segment, Visit},
        Field, Ident, PathSegment,
    },
};

#[derive(Default, Clone)]
pub struct AccountMeta {
    pub is_signer: bool,
    pub is_mutable: bool,
    pub is_optional: bool,
}

#[derive(Clone)]
pub struct InstructionAccount {
    pub name: Ident,
    pub constraints: Constraints,
    pub meta: AccountMeta,
    pub docs: Vec<String>,
    ty: Option<PathSegment>,
    pub inner_ty: Ident,
}

impl InstructionAccount {
    pub fn get_ty(&self) -> PathSegment {
        let inner_ty = &self.inner_ty;
        self.ty.clone().unwrap_or(parse_quote!(#inner_ty<'info>))
    }
}

impl TryFrom<&Field> for InstructionAccount {
    type Error = syn::Error;

    fn try_from(value: &Field) -> Result<Self, Self::Error> {
        let mut acc = InstructionAccount {
            name: value
                .ident
                .clone()
                .ok_or(syn::Error::new_spanned(value, "The field need to be named"))?,
            docs: parse_docs(&value.attrs),
            constraints: Constraints::try_from(value.attrs.as_slice())?,
            meta: AccountMeta::default(),
            ty: None,
            inner_ty: Ident::new("UncheckedAccount", Span::call_site()),
        };

        acc.visit_type(&value.ty);
        Ok(acc)
    }
}

impl Visit<'_> for InstructionAccount {
    fn visit_path_segment(&mut self, i: &syn::PathSegment) {
        let ty = i.ident.to_string();

        if ty == "Option" {
            self.meta.is_optional = true;
            visit_path_segment(self, i);
        } else if ty.starts_with("Mut") {
            self.meta.is_mutable = true;
            if self.ty.is_none() {
                self.ty = Some(i.clone());
            }
            visit_path_segment(self, i);
        } else if ty.ends_with("Signer") {
            self.meta.is_signer = true;
            if self.ty.is_none() {
                self.ty = Some(i.clone());
            }
            visit_path_segment(self, i);
        } else {
            if self.ty.is_none() {
                self.ty = Some(i.clone());
            }
            self.inner_ty = i.ident.clone();
            visit_path_segment(self, i);
        }
    }
}

#[cfg(test)]
mod tests {
    use {super::*, syn::parse_quote};

    #[test]
    fn test_instruction_parse_with_context() {
        let field: syn::Field = parse_quote!(pub random2: Option<Mut<Account<Random2>>>);
        let account = InstructionAccount::try_from(&field).unwrap();
        assert_eq!(account.inner_ty, "Random2");
        assert!(account.meta.is_mutable);
        assert!(account.meta.is_optional);
        assert!(!account.meta.is_signer);

        let field: syn::Field = parse_quote!(pub random2: UncheckedSigner<'info, Account<Random2>>);
        let account = InstructionAccount::try_from(&field).unwrap();
        assert_eq!(account.inner_ty, "Random2");
        assert!(!account.meta.is_mutable);
        assert!(!account.meta.is_optional);
        assert!(account.meta.is_signer);

        let field: syn::Field = parse_quote!(pub random2: Signer<'info, Account<Random2>>);
        let account = InstructionAccount::try_from(&field).unwrap();
        assert_eq!(account.inner_ty, "Random2");
        assert!(!account.meta.is_mutable);
        assert!(!account.meta.is_optional);
        assert!(account.meta.is_signer);

        let field: syn::Field = parse_quote!(pub random2: Account<TokenAccount>);
        let account = InstructionAccount::try_from(&field).unwrap();
        assert_eq!(account.inner_ty, "TokenAccount");
        assert!(!account.meta.is_mutable);
        assert!(!account.meta.is_optional);
        assert!(!account.meta.is_signer);

        let field: syn::Field = parse_quote!(pub random2: UncheckedAccount);
        let account = InstructionAccount::try_from(&field).unwrap();
        assert_eq!(account.inner_ty, "UncheckedAccount");
        assert!(!account.meta.is_mutable);
        assert!(!account.meta.is_optional);
        assert!(!account.meta.is_signer);
    }
}
