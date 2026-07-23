use {
    quote::{format_ident, quote, ToTokens},
    syn::{parse_macro_input, spanned::Spanned, Error, Fields, Item},
};

/// Marks an enum as a set of events that can be emitted to the transaction logs.
///
/// Each variant is an event. The discriminator of an event is the index of its
/// variant, encoded as a single `u8`. Emitting a value writes that byte followed
/// by the packed variant fields to the logs, and generates an `Emit`
/// implementation so the event can be logged by calling `.emit()`:
///
/// ```ignore
/// #[event]
/// pub enum CounterEvent {
///     Incremented { count: u64 },
///     Decremented { count: u64 },
/// }
///
/// CounterEvent::Incremented { count: 1 }.emit();
/// ```
///
/// Variant fields must be plain-old-data (`bytemuck::NoUninit`).
#[proc_macro_attribute]
pub fn event(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as Item);
    let Item::Enum(item_enum) = item else {
        return Error::new(item.span(), "`#[event]` can only be applied to enums")
            .into_compile_error()
            .into();
    };

    let name = &item_enum.ident;
    let (impl_generics, ty_generics, where_clause) = item_enum.generics.split_for_impl();

    let arms = item_enum
        .variants
        .iter()
        .enumerate()
        .map(|(index, variant)| {
            let discriminator = index as u8;
            let variant_name = &variant.ident;

            match &variant.fields {
                Fields::Unit => quote! {
                    #name::#variant_name => emit_bytes(&[#discriminator]),
                },
                Fields::Named(fields) => {
                    let bindings: Vec<_> = fields
                        .named
                        .iter()
                        .map(|field| field.ident.clone().unwrap())
                        .collect();
                    let types: Vec<_> = fields.named.iter().map(|field| &field.ty).collect();

                    quote! {
                        #name::#variant_name { #(#bindings),* } => {
                            const SIZE: usize = 1 #(+ ::core::mem::size_of::<#types>())*;
                            let mut buffer = [0u8; SIZE];
                            buffer[0] = #discriminator;
                            let mut offset = 1usize;
                            #(
                                let bytes = ::bytemuck::bytes_of(#bindings);
                                buffer[offset..offset + bytes.len()].copy_from_slice(bytes);
                                offset += bytes.len();
                            )*
                            let _ = offset;
                            emit_bytes(&buffer);
                        }
                    }
                }
                Fields::Unnamed(fields) => {
                    let bindings: Vec<_> = (0..fields.unnamed.len())
                        .map(|i| format_ident!("field_{i}"))
                        .collect();
                    let types: Vec<_> = fields.unnamed.iter().map(|field| &field.ty).collect();

                    quote! {
                        #name::#variant_name( #(#bindings),* ) => {
                            const SIZE: usize = 1 #(+ ::core::mem::size_of::<#types>())*;
                            let mut buffer = [0u8; SIZE];
                            buffer[0] = #discriminator;
                            let mut offset = 1usize;
                            #(
                                let bytes = ::bytemuck::bytes_of(#bindings);
                                buffer[offset..offset + bytes.len()].copy_from_slice(bytes);
                                offset += bytes.len();
                            )*
                            let _ = offset;
                            emit_bytes(&buffer);
                        }
                    }
                }
            }
        });

    quote! {
        #item_enum

        impl #impl_generics Emit for #name #ty_generics #where_clause {
            fn emit(&self) {
                match self {
                    #(#arms)*
                }
            }
        }
    }
    .into_token_stream()
    .into()
}
