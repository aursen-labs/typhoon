use {
    base64::{prelude::BASE64_STANDARD, Engine},
    codama::{
        BytesEncoding, CamelCaseString, CombineTypesVisitor, ConstantDiscriminatorNode,
        ConstantValueNode, DiscriminatorNode, Docs, EnumVariantTypeNode, EventNode, KorokVisitor,
        NestedTypeNode, Node, ProgramNode, StructFieldTypeNode, StructTypeNode,
    },
    codama_attributes::{Attribute, Attributes},
    typhoon_syn::parse_docs,
};

/// Returns `true` if the item carries the `#[event]` attribute.
///
/// `#[event]` is a Typhoon attribute macro, so codama parses it as an
/// unsupported attribute rather than a known derive.
fn has_event_attribute(attributes: &Attributes) -> bool {
    attributes.iter().any(|attr| {
        matches!(attr, Attribute::Unsupported(unsupported) if unsupported.ast.path().is_ident("event"))
    })
}

/// Extracts the name and the struct-shaped data of an enum variant.
fn parse_variant(
    korok: &codama_koroks::EnumVariantKorok,
) -> Option<(CamelCaseString, NestedTypeNode<StructTypeNode>)> {
    let node = korok.node.as_ref()?;
    let variant = EnumVariantTypeNode::try_from(node.clone()).ok()?;

    match variant {
        EnumVariantTypeNode::Struct(node) => Some((node.name, node.r#struct)),
        EnumVariantTypeNode::Empty(node) => Some((node.name, StructTypeNode::new(vec![]).into())),
        EnumVariantTypeNode::Tuple(node) => {
            let NestedTypeNode::Value(tuple) = node.tuple else {
                return None;
            };
            let fields = tuple
                .items
                .into_iter()
                .enumerate()
                .map(|(index, item)| StructFieldTypeNode::new(format!("field{index}"), item))
                .collect();
            Some((node.name, StructTypeNode::new(fields).into()))
        }
    }
}

pub struct SetEventVisitor {
    visitor: CombineTypesVisitor,
}

impl Default for SetEventVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl SetEventVisitor {
    pub fn new() -> Self {
        Self {
            visitor: CombineTypesVisitor::new(),
        }
    }
}

impl KorokVisitor for SetEventVisitor {
    fn visit_enum(&mut self, korok: &mut codama_koroks::EnumKorok) -> codama::CodamaResult<()> {
        if !has_event_attribute(&korok.attributes) {
            return Ok(());
        };

        // Resolve the type of every variant (and its fields).
        self.visitor.visit_enum(korok)?;

        // Each variant becomes an event whose discriminator is the index of the
        // variant encoded as a single `u8`.
        let events = korok
            .variants
            .iter()
            .enumerate()
            .filter_map(|(index, variant)| {
                let (name, data) = parse_variant(variant)?;
                Some(EventNode {
                    name,
                    docs: Docs::from(parse_docs(&variant.ast.attrs)),
                    data: Box::new(data.into()),
                    discriminators: vec![DiscriminatorNode::Constant(
                        ConstantDiscriminatorNode::new(
                            ConstantValueNode::bytes(
                                BytesEncoding::Base64,
                                BASE64_STANDARD.encode([index as u8]),
                            ),
                            0,
                        ),
                    )],
                })
            })
            .collect();

        korok.node = Some(Node::Program(ProgramNode {
            events,
            ..ProgramNode::default()
        }));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        codama::{CodamaResult, EnumKorok, IdentifyFieldTypesVisitor, KorokVisitable, Node},
        syn::{parse_quote, Item},
    };

    #[test]
    fn test_visit_enum() -> CodamaResult<()> {
        let item: Item = parse_quote! {
            #[event]
            pub enum CounterEvent {
                Incremented { count: u64 },
                Reset,
            }
        };

        let mut korok = EnumKorok::parse(&item)?;
        korok.accept(&mut IdentifyFieldTypesVisitor::new())?;
        korok.accept(&mut SetEventVisitor::new())?;

        let Some(Node::Program(program)) = korok.node else {
            panic!("Expected Program node");
        };

        assert_eq!(program.events.len(), 2);

        let incremented = &program.events[0];
        assert_eq!(incremented.name.as_str(), "incremented");
        assert_eq!(incremented.discriminators.len(), 1);
        match &*incremented.data {
            codama::TypeNode::Struct(s) => {
                assert_eq!(s.fields.len(), 1);
                assert_eq!(s.fields[0].name.as_str(), "count");
            }
            _ => panic!("Expected Struct data"),
        }

        assert_eq!(program.events[1].name.as_str(), "reset");

        Ok(())
    }
}
