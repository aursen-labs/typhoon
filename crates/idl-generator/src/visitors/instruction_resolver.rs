use {
    crate::utils::extract_type,
    codama::{
        CamelCaseString, Docs, InstructionArgumentNode, InstructionNode, KorokVisitor, Node,
        OptionalAccountStrategy, ProgramNode, UnsupportedItemKorok,
    },
    syn::Item,
    typhoon_syn::{Instruction, InstructionArg},
};

#[derive(Default)]
pub struct InstructionResolver {
    router_cache: Option<ProgramNode>,
    context_cache: Vec<InstructionNode>,
    phase: VisitPhase,
}

#[derive(Default, PartialEq)]
enum VisitPhase {
    #[default]
    Preload,
    Unsupported,
}

impl InstructionResolver {
    pub fn new() -> Self {
        Self::default()
    }
}

impl KorokVisitor for InstructionResolver {
    fn visit_root(&mut self, korok: &mut codama_koroks::RootKorok) -> codama::CodamaResult<()> {
        self.phase = VisitPhase::Preload;
        for crate_korok in &mut korok.crates {
            self.visit_crate(crate_korok)?;
        }

        self.phase = VisitPhase::Unsupported;
        for crate_korok in &mut korok.crates {
            self.visit_crate(crate_korok)?;
        }

        let Some(program) = self.router_cache.take() else {
            return Ok(());
        };

        if let Some(Node::Root(root)) = korok.node.as_mut() {
            root.program.public_key = program.public_key;
            root.program.instructions = program.instructions;
        }

        Ok(())
    }

    fn visit_item(&mut self, korok: &mut codama_koroks::ItemKorok) -> codama::CodamaResult<()> {
        match korok {
            codama_koroks::ItemKorok::FileModule(korok) => self.visit_file_module(korok),
            codama_koroks::ItemKorok::Module(korok) => self.visit_module(korok),
            codama_koroks::ItemKorok::Struct(korok) => {
                if self.phase == VisitPhase::Preload {
                    self.visit_struct(korok)
                } else {
                    Ok(())
                }
            }
            codama_koroks::ItemKorok::Enum(korok) => {
                if self.phase == VisitPhase::Preload {
                    self.visit_enum(korok)
                } else {
                    Ok(())
                }
            }
            codama_koroks::ItemKorok::Impl(korok) => {
                if self.phase == VisitPhase::Preload {
                    self.visit_impl(korok)
                } else {
                    Ok(())
                }
            }
            codama_koroks::ItemKorok::Const(korok) => {
                if self.phase == VisitPhase::Preload {
                    self.visit_const(korok)
                } else {
                    Ok(())
                }
            }
            codama_koroks::ItemKorok::Unsupported(korok) => {
                if self.phase == VisitPhase::Unsupported {
                    self.visit_unsupported_item(korok)
                } else {
                    Ok(())
                }
            }
        }
    }

    fn visit_const(&mut self, korok: &mut codama_koroks::ConstKorok) -> codama::CodamaResult<()> {
        let Some(Node::Program(program)) = korok.node.take() else {
            return Ok(());
        };
        self.router_cache = Some(program);

        Ok(())
    }

    fn visit_struct(&mut self, korok: &mut codama_koroks::StructKorok) -> codama::CodamaResult<()> {
        let Some(Node::Instruction(instruction)) = korok.node.take() else {
            return Ok(());
        };
        self.context_cache.push(instruction);
        Ok(())
    }

    fn visit_unsupported_item(
        &mut self,
        korok: &mut UnsupportedItemKorok,
    ) -> codama::CodamaResult<()> {
        let UnsupportedItemKorok {
            ast: Item::Fn(item_fn),
            ..
        } = korok
        else {
            return Ok(());
        };

        let Some(router_cache) = self.router_cache.as_mut() else {
            return Ok(());
        };

        let ix = Instruction::try_from(item_fn)?;
        let Some(cache_ix) = router_cache
            .instructions
            .iter_mut()
            .find(|el| el.name.as_str() == CamelCaseString::new(ix.name.to_string()).as_str())
        else {
            return Ok(());
        };

        for (arg_name, arg_value) in &ix.args {
            match arg_value {
                InstructionArg::Context(ctx) => {
                    let ctx_name = CamelCaseString::new(ctx.to_string());
                    let Some(context) = self
                        .context_cache
                        .iter()
                        .find(|el| el.name.as_str() == ctx_name.as_str())
                    else {
                        return Err(codama::CodamaError::NodeNotFound);
                    };
                    cache_ix.accounts.extend(context.accounts.clone());
                    cache_ix.arguments.extend(context.arguments.clone());
                }
                InstructionArg::Type { ty, .. } => {
                    cache_ix.arguments.push(InstructionArgumentNode {
                        name: CamelCaseString::new(arg_name.to_string()),
                        r#type: Box::new(extract_type(ty.as_ref())?),
                        default_value: Box::new(None),
                        default_value_strategy: None,
                        docs: Docs::new(),
                    });
                }
            }
        }

        cache_ix.optional_account_strategy = Some(OptionalAccountStrategy::ProgramId);

        Ok(())
    }
}
