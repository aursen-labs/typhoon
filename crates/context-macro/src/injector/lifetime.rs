use syn::{parse_quote, visit_mut::VisitMut, PathArguments};

pub struct LifetimeInjector;

impl VisitMut for LifetimeInjector {
    fn visit_generics_mut(&mut self, i: &mut syn::Generics) {
        i.params.push(parse_quote!('info));
    }

    fn visit_type_path_mut(&mut self, i: &mut syn::TypePath) {
        if let Some(seg) = i.path.segments.last_mut() {
            let ident = seg.ident.to_string();

            // `Option<T>` is transparent: recurse into its inner type
            // without injecting a lifetime on `Option` itself.
            if ident == "Option" {
                if let PathArguments::AngleBracketed(ref mut angle_args) = seg.arguments {
                    if let Some(first_arg) = angle_args.args.first_mut() {
                        self.visit_generic_argument_mut(first_arg);
                    }
                }
                return;
            }

            // `Mut<T>` now has its own lifetime parameter (the mutable borrow
            // of the slot), so we inject `'info` AND recurse into the inner
            // type to inject `'info` there too.
            if ident.starts_with("Mut") {
                match seg.arguments {
                    PathArguments::AngleBracketed(ref mut gen_args) => {
                        gen_args.args.insert(0, parse_quote!('info));
                        if let Some(arg) = gen_args.args.iter_mut().nth(1) {
                            self.visit_generic_argument_mut(arg);
                        }
                    }
                    PathArguments::None => {
                        seg.arguments = PathArguments::AngleBracketed(parse_quote!(<'info>));
                    }
                    PathArguments::Parenthesized(_) => {}
                }
                return;
            }

            match seg.arguments {
                PathArguments::AngleBracketed(ref mut gen_args) => {
                    gen_args.args.insert(0, parse_quote!('info));
                }
                PathArguments::None => {
                    seg.arguments = PathArguments::AngleBracketed(parse_quote!(<'info>));
                }
                PathArguments::Parenthesized(_) => {}
            }

            if ident.ends_with("Signer") {
                self.visit_path_arguments_mut(&mut seg.arguments);
            }
        }
    }
}
