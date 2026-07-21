use {
    codama::{ErrorNode, KorokVisitor, Node, ProgramNode},
    typhoon_syn::Errors,
};

#[derive(Default)]
pub struct SetErrorsVisitor;

impl SetErrorsVisitor {
    pub fn new() -> Self {
        Self
    }
}

impl KorokVisitor for SetErrorsVisitor {
    fn visit_enum(&mut self, korok: &mut codama_koroks::EnumKorok) -> codama::CodamaResult<()> {
        // No overrides.
        if korok.node.is_some() {
            return Ok(());
        };

        if !korok.attributes.has_derive(&[""], "TyphoonError") {
            return Ok(());
        }

        let errors = Errors::try_from(korok.ast)?;
        //TODO inject Typhoon errors here

        let program = match korok
            .node
            .get_or_insert_with(|| ProgramNode::default().into())
        {
            Node::Root(root) => &mut root.program,
            Node::Program(program) => program,
            _ => return Ok(()),
        };

        program.errors = errors
            .variants
            .into_iter()
            .map(|v| ErrorNode::new(v.name.to_string(), v.discriminant, v.msg))
            .collect();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        codama::{CodamaResult, EnumKorok, KorokVisitable},
        syn::{parse_quote, Item},
    };

    #[test]
    fn test_visit_enum() -> CodamaResult<()> {
        let item: Item = parse_quote! {
            #[derive(TyphoonError)]
            pub enum MyError {
                #[msg("My error")]
                MyError = 200,
                #[msg("Another error")]
                AnotherError,
            }
        };

        let mut korok = EnumKorok::parse(&item)?;
        let mut visitor = SetErrorsVisitor::new();
        korok.accept(&mut visitor)?;

        assert_eq!(
            korok.node,
            Some(
                ProgramNode {
                    errors: vec![
                        ErrorNode::new("myError", 200, "My error"),
                        ErrorNode::new("anotherError", 201, "Another error"),
                    ],
                    ..ProgramNode::default()
                }
                .into()
            )
        );

        Ok(())
    }
}
