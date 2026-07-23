use {
    crate::idl::{ArrayLen, Type as IdlType},
    quote::{format_ident, quote},
    syn::{parse_quote, Type},
};

fn gen_primitive(idl_ty: &IdlType) -> Option<Type> {
    Some(match idl_ty {
        IdlType::Bool => parse_quote!(bool),
        IdlType::U8 => parse_quote!(u8),
        IdlType::I8 => parse_quote!(i8),
        IdlType::U16 => parse_quote!(u16),
        IdlType::I16 => parse_quote!(i16),
        IdlType::U32 => parse_quote!(u32),
        IdlType::I32 => parse_quote!(i32),
        IdlType::F32 => parse_quote!(f32),
        IdlType::U64 => parse_quote!(u64),
        IdlType::I64 => parse_quote!(i64),
        IdlType::F64 => parse_quote!(f64),
        IdlType::U128 => parse_quote!(u128),
        IdlType::I128 => parse_quote!(i128),
        IdlType::U256 | IdlType::I256 | IdlType::Generic(_) => unimplemented!(),
        _ => return None,
    })
}

pub fn gen_type(idl_ty: &IdlType) -> Type {
    if let Some(ty) = gen_primitive(idl_ty) {
        return ty;
    }
    match idl_ty {
        IdlType::Bytes => parse_quote!(Vec<u8>),
        IdlType::String => parse_quote!(String),
        IdlType::Pubkey => parse_quote!(Address),
        IdlType::Option(inner) => {
            let ty = gen_type(inner);
            parse_quote!(Option<#ty>)
        }
        IdlType::Vec(inner) => {
            let ty = gen_type(inner);
            parse_quote!(Vec<#ty>)
        }
        IdlType::Defined(defined) => {
            let ident = format_ident!("{}", defined.name());
            parse_quote!(#ident)
        }
        IdlType::Array(inner, len) => {
            let ty = gen_type(inner);
            let size = match len {
                ArrayLen::Generic(size) => quote!(#size),
                ArrayLen::Value(size) => quote!(#size),
            };
            parse_quote!([#ty; #size])
        }
        IdlType::HashMap(key, value) => {
            let key = gen_type(key);
            let value = gen_type(value);

            parse_quote!(HashMap<#key, #value>)
        }
        IdlType::BTreeMap(key, value) => {
            let key = gen_type(key);
            let value = gen_type(value);

            parse_quote!(BTreeMap<#key, #value>)
        }
        IdlType::HashSet(ty) => {
            let ty = gen_type(ty);

            parse_quote!(HashSet<#ty>)
        }
        IdlType::BTreeSet(ty) => {
            let ty = gen_type(ty);

            parse_quote!(BTreeSet<#ty>)
        }
        _ => unreachable!(),
    }
}

pub fn gen_type_ref(idl_ty: &IdlType) -> Type {
    if let Some(ty) = gen_primitive(idl_ty) {
        return ty;
    }
    match idl_ty {
        IdlType::Bytes => parse_quote!(&'a [u8]),
        IdlType::String => parse_quote!(&'a str),
        IdlType::Pubkey => parse_quote!(&'a Address),
        IdlType::Option(inner) => {
            let ty = gen_type_ref(inner);
            parse_quote!(Option<#ty>)
        }
        IdlType::Vec(inner) => {
            let ty = gen_type_ref(inner);
            parse_quote!(&'a [#ty])
        }
        IdlType::Array(inner, len) => {
            let ty = gen_type_ref(inner);
            let size = match len {
                ArrayLen::Generic(size) => quote!(#size),
                ArrayLen::Value(size) => quote!(#size),
            };
            parse_quote!(&'a [#ty; #size])
        }
        IdlType::Defined(defined) => {
            let ident = format_ident!("{}", defined.name());
            parse_quote!(&'a #ident)
        }
        IdlType::HashMap(key, value) => {
            let key = gen_type_ref(key);
            let value = gen_type_ref(value);

            parse_quote!(&'a HashMap<#key, #value>)
        }
        IdlType::BTreeMap(key, value) => {
            let key = gen_type_ref(key);
            let value = gen_type_ref(value);

            parse_quote!(&'a BTreeMap<#key, #value>)
        }
        IdlType::HashSet(ty) => {
            let ty = gen_type_ref(ty);

            parse_quote!(&'a HashSet<#ty>)
        }
        IdlType::BTreeSet(ty) => {
            let ty = gen_type_ref(ty);

            parse_quote!(&'a BTreeSet<#ty>)
        }
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use {super::*, quote::ToTokens};

    #[test]
    fn array_ref_is_reference_to_fixed_array() {
        let arr = IdlType::Array(Box::new(IdlType::U8), ArrayLen::Value(32));

        // The by-ref form must be a reference to the *same* fixed-size array the
        // owned form produces — not a slice — so Borsh serializes it inline.
        let owned = gen_type(&arr);
        let expected: Type = parse_quote!(&'a #owned);
        assert_eq!(
            gen_type_ref(&arr).to_token_stream().to_string(),
            expected.to_token_stream().to_string(),
        );
    }

    #[test]
    fn vec_ref_stays_a_slice() {
        let vec = IdlType::Vec(Box::new(IdlType::U8));
        assert_eq!(
            gen_type_ref(&vec).to_token_stream().to_string(),
            quote!(&'a [u8]).to_string(),
        );
    }
}
