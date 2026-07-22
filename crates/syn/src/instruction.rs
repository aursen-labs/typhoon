use {
    crate::{helpers::PathHelper, Encoding},
    heck::ToSnakeCase,
    quote::format_ident,
    syn::{
        parse::{Parse, Parser},
        punctuated::Punctuated,
        visit::Visit,
        Expr, FnArg, GenericArgument, Ident, LitInt, Pat, Token, Type, TypePath,
    },
};

pub struct InstructionReturnData {
    pub ty: Option<Type>,
    pub encoding: Encoding,
}

pub enum InstructionArg {
    Type { ty: Box<Type>, encoding: Encoding },
    Context(Ident),
}

pub struct Instruction {
    pub name: Ident,
    pub args: Vec<(Ident, InstructionArg)>,
    pub return_data: InstructionReturnData,
}

impl TryFrom<&syn::ItemFn> for Instruction {
    type Error = syn::Error;

    fn try_from(value: &syn::ItemFn) -> Result<Self, Self::Error> {
        let return_data = value
            .sig
            .output
            .get_element_with_inner()
            .and_then(|(_, inner, _)| inner);

        let mut args = Vec::with_capacity(value.sig.inputs.len());
        for fn_arg in &value.sig.inputs {
            let FnArg::Typed(pat_ty) = fn_arg else {
                continue;
            };

            let Type::Path(ref ty_path) = *pat_ty.ty else {
                continue;
            };

            let (name, ty, size) = ty_path
                .get_element_with_inner()
                .ok_or(syn::Error::new_spanned(fn_arg, "Invalid FnArg."))?;

            if name == "ProgramIdArg" || name == "Remaining" || name == "AccountIter" {
                continue;
            }

            let arg_name = extract_name(&pat_ty.pat)
                .unwrap_or(format_ident!("{}", name.to_string().to_snake_case()));

            if name == "Arg" {
                args.push((
                    arg_name,
                    InstructionArg::Type {
                        ty: Box::new(
                            ty.ok_or(syn::Error::new_spanned(fn_arg, "Invalid argument type."))?,
                        ),
                        encoding: infer_arg_encoding(ty_path),
                    },
                ));
            } else if name == "Array" {
                let size = size.ok_or(syn::Error::new_spanned(fn_arg, "Invalid Array type."))?;
                let ty = ty.ok_or(syn::Error::new_spanned(fn_arg, "Invalid argument type."))?;
                let Type::Path(path) = ty else {
                    return Err(syn::Error::new_spanned(&arg_name, "Invalid ty_path."));
                };
                let (name, _, _) = path
                    .get_element_with_inner()
                    .ok_or(syn::Error::new_spanned(&path, "Invalid Array inner type."))?;
                for i in 0..size {
                    let arg_name = format_ident!("{arg_name}_{i}");
                    args.push((arg_name, InstructionArg::Context(name.clone())));
                }
            } else {
                args.push((arg_name, InstructionArg::Context(name.clone())));
            }
        }

        Ok(Instruction {
            name: value.sig.ident.clone(),
            args,
            return_data: InstructionReturnData {
                ty: return_data,
                encoding: Encoding::Bytemuck,
            },
        })
    }
}

fn infer_arg_encoding(ty_path: &TypePath) -> Encoding {
    let Some(seg) = ty_path.path.segments.last() else {
        return Encoding::Bytemuck;
    };
    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
        return Encoding::Bytemuck;
    };

    let strategy = args
        .args
        .iter()
        .filter_map(|arg| match arg {
            GenericArgument::Type(ty) => Some(ty),
            _ => None,
        })
        .nth(1);

    match strategy {
        None => Encoding::Bytemuck,
        Some(Type::Path(path)) if path.path.is_ident("BytemuckStrategy") => Encoding::Bytemuck,
        Some(Type::Path(path)) if path.path.is_ident("BorshStrategy") => Encoding::Borsh,
        Some(_) => Encoding::Custom,
    }
}

fn extract_name(pat: &Pat) -> Option<Ident> {
    match pat {
        Pat::Ident(ident) => Some(ident.ident.clone()),
        Pat::TupleStruct(tuple_struct) => {
            let pat = tuple_struct.elems.first()?;
            extract_name(pat)
        }
        _ => None,
    }
}

#[derive(Default)]
pub struct InstructionsList(pub Vec<(usize, Ident)>);

struct RouterEntry {
    discriminator: LitInt,
    handler_name: Ident,
}

impl Parse for RouterEntry {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let discriminator = input.parse()?;
        input.parse::<Token![=>]>()?;
        Ok(RouterEntry {
            discriminator,
            handler_name: input.parse()?,
        })
    }
}

impl TryFrom<&syn::ItemConst> for InstructionsList {
    type Error = syn::Error;

    fn try_from(value: &syn::ItemConst) -> syn::Result<Self> {
        let Expr::Macro(expr_macro) = value.expr.as_ref() else {
            return Err(syn::Error::new_spanned(value, "Invalid router type."));
        };

        let instructions = Punctuated::<RouterEntry, syn::Token![,]>::parse_terminated
            .parse2(expr_macro.mac.tokens.clone())?;
        Ok(Self(
            instructions
                .iter()
                .map(|entry| {
                    Ok((
                        entry.discriminator.base10_parse::<usize>()?,
                        entry.handler_name.clone(),
                    ))
                })
                .collect::<Result<_, syn::Error>>()?,
        ))
    }
}

impl<'ast> Visit<'ast> for InstructionsList {
    fn visit_item_const(&mut self, i: &'ast syn::ItemConst) {
        if i.ident != "ROUTER" {
            return;
        }

        if let Ok(ix_list) = InstructionsList::try_from(i) {
            *self = ix_list;
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        syn::{parse_quote, ItemConst, ItemFn},
    };

    #[test]
    fn test_instruction_list() {
        let router: ItemConst = parse_quote! {
            pub const ROUTER: EntryFn = basic_router! {
                0 => account_iter,
                1 => initialize,
                2 => assert
            };
        };

        let ix_list = InstructionsList::try_from(&router).unwrap();
        assert_eq!(ix_list.0[0].0, 0);
        assert_eq!(ix_list.0[1].0, 1);
        assert_eq!(ix_list.0[2].0, 2);
        assert_eq!(ix_list.0[0].1, "account_iter");
        assert_eq!(ix_list.0[1].1, "initialize");
        assert_eq!(ix_list.0[2].1, "assert");
    }

    #[test]
    fn test_instruction_construction() {
        let fn_raw: ItemFn = parse_quote! {
            pub fn instruction_1(ctx: Context1, array: Array<Context2, 2>, arg: Arg<u64>, arg2: Arg<u64, BorshStrategy>) -> ProgramResult {
                Ok(())
            }
        };
        let ix = Instruction::try_from(&fn_raw).unwrap();

        assert_eq!(ix.name, "instruction_1");
        assert_eq!(ix.args.len(), 5);
        assert_eq!(ix.args[0].0, "ctx");
        assert!(matches!(&ix.args[0].1, InstructionArg::Context(x) if x == "Context1"));
        assert_eq!(ix.args[1].0, "array_0");
        assert!(matches!(&ix.args[1].1, InstructionArg::Context(x) if x == "Context2"));
        assert_eq!(ix.args[2].0, "array_1");
        assert!(matches!(&ix.args[2].1, InstructionArg::Context(x) if x == "Context2"));
        assert_eq!(ix.args[3].0, "arg");
        assert!(matches!(
            &ix.args[3].1,
            InstructionArg::Type { ty, encoding }
                if matches!(**ty, Type::Path(ref path) if path.path.is_ident("u64"))
                    && matches!(encoding, Encoding::Bytemuck)
        ));
        assert_eq!(ix.args[4].0, "arg2");
        assert!(matches!(
            &ix.args[4].1,
            InstructionArg::Type { ty, encoding }
                if matches!(**ty, Type::Path(ref path) if path.path.is_ident("u64"))
                    && matches!(encoding, Encoding::Borsh)
        ));
        assert!(ix.return_data.ty.is_none());
        assert!(matches!(ix.return_data.encoding, Encoding::Bytemuck));
    }
}
