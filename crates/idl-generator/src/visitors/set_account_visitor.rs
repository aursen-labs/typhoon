use {
    base64::{prelude::BASE64_STANDARD, Engine},
    codama::{
        AccountNode, CamelCaseString, CombineTypesVisitor, ConstantDiscriminatorNode,
        ConstantValueNode, DefinedTypeNode, DiscriminatorNode, Docs, KorokVisitor, Node, TypeNode,
    },
    typhoon_discriminator::DiscriminatorBuilder,
    typhoon_syn::parse_docs,
};

pub struct SetAccountVisitor {
    visitor: CombineTypesVisitor,
}

impl Default for SetAccountVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl SetAccountVisitor {
    pub fn new() -> Self {
        Self {
            visitor: CombineTypesVisitor::new(),
        }
    }
}

impl KorokVisitor for SetAccountVisitor {
    fn visit_struct(&mut self, korok: &mut codama_koroks::StructKorok) -> codama::CodamaResult<()> {
        if !korok.attributes.has_derive(&[""], "AccountState") {
            return Ok(());
        };

        self.visitor.visit_struct(korok)?;

        let Some(Node::DefinedType(DefinedTypeNode { r#type, .. })) = korok.node.take() else {
            return Ok(());
        };
        let TypeNode::Struct(ty) = *r#type else {
            return Ok(());
        };

        let dis = DiscriminatorBuilder::new(&korok.ast.ident.to_string()).build();

        korok.node = Some(Node::Account(AccountNode {
            name: CamelCaseString::new(korok.ast.ident.to_string()),
            size: None,
            docs: Docs::from(parse_docs(&korok.ast.attrs)),
            data: codama::NestedTypeNode::Value(ty),
            pda: None,
            discriminators: vec![DiscriminatorNode::Constant(ConstantDiscriminatorNode::new(
                ConstantValueNode::bytes(
                    codama::BytesEncoding::Base64,
                    BASE64_STANDARD.encode(dis),
                ),
                0,
            ))],
        }));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        codama::{CodamaResult, IdentifyFieldTypesVisitor, KorokVisitable, Node, StructKorok},
        syn::{parse_quote, Item},
    };

    #[test]
    fn test_visit_struct() -> CodamaResult<()> {
        let item: Item = parse_quote! {
            #[derive(NoUninit, AnyBitPattern, AccountState, Copy, Clone)]
            #[repr(C)]
            pub struct Counter {
                pub count: u64,
            }
        };

        let mut korok = StructKorok::parse(&item)?;
        korok.accept(&mut IdentifyFieldTypesVisitor::new())?;
        korok.accept(&mut SetAccountVisitor::new())?;

        let Some(Node::Account(account)) = korok.node else {
            panic!("Expected Account node");
        };

        assert_eq!(account.name.as_str(), "counter");
        match account.data {
            codama::NestedTypeNode::Value(s) => {
                assert_eq!(s.fields.len(), 1);
                assert_eq!(s.fields[0].name.as_str(), "count");
            }
            _ => panic!("Expected Struct data"),
        }

        Ok(())
    }
}
