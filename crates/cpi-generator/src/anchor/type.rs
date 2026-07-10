use {
    crate::idl::{ArrayLen, Type as IdlType},
    quote::{format_ident, quote},
    syn::{parse_quote, Type},
};

pub fn gen_type(idl_ty: &IdlType) -> Type {
    match idl_ty {
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
        IdlType::Bytes => parse_quote!(Vec<u8>),
        IdlType::String => parse_quote!(Vec<u8>),
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
        IdlType::U256 | IdlType::I256 | IdlType::Generic(_) => unimplemented!(),
    }
}

pub fn gen_type_ref(idl_ty: &IdlType) -> Type {
    match idl_ty {
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
        IdlType::Bytes => parse_quote!(&'a [u8]),
        IdlType::String => parse_quote!(&'a str),
        IdlType::Pubkey => parse_quote!(&'a Address),
        IdlType::Option(inner) => {
            let ty = gen_type_ref(inner);
            parse_quote!(Option<#ty>)
        }
        IdlType::Vec(inner) | IdlType::Array(inner, _) => {
            let ty = gen_type_ref(inner);
            parse_quote!(&'a [#ty])
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
        IdlType::U256 | IdlType::I256 | IdlType::Generic(_) => unimplemented!(),
    }
}
