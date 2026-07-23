use {
    crate::visitors::{
        ContextVisitor, DefineTypeResolver, InstructionResolver, RouterVisitor, SetAccountVisitor,
        SetErrorsVisitor, SetEventVisitor, SetProgramIdVisitor,
    },
    codama::{
        ApplyTypeModifiersVisitor, ApplyTypeOverridesVisitor, CodamaResult, CombineModulesVisitor,
        IdentifyFieldTypesVisitor, SetDefaultValuesVisitor, SetProgramMetadataVisitor,
    },
    codama_korok_visitors::KorokVisitable,
    codama_plugin_core::KorokPlugin,
};

pub struct TyphoonPlugin;

impl KorokPlugin for TyphoonPlugin {
    fn on_initialized(&self, visitable: &mut dyn KorokVisitable) -> CodamaResult<()> {
        visitable.accept(&mut RouterVisitor::new())?;
        Ok(())
    }

    fn on_fields_set(&self, visitable: &mut dyn KorokVisitable) -> CodamaResult<()> {
        // visitable.accept(&mut SetPdasVisitor::new())?; //TODO seeds and seeded
        visitable.accept(&mut IdentifyFieldTypesVisitor::new())?;
        visitable.accept(&mut ApplyTypeOverridesVisitor::new())?;
        visitable.accept(&mut ApplyTypeModifiersVisitor::new())?;
        visitable.accept(&mut SetDefaultValuesVisitor::new())?;
        Ok(())
    }

    fn on_program_items_set(&self, visitable: &mut dyn KorokVisitable) -> CodamaResult<()> {
        visitable.accept(&mut ContextVisitor::new())?;
        visitable.accept(&mut SetErrorsVisitor::new())?;
        visitable.accept(&mut SetAccountVisitor::new())?;
        visitable.accept(&mut SetEventVisitor::new())?;
        Ok(())
    }

    fn on_root_node_set(&self, visitable: &mut dyn KorokVisitable) -> CodamaResult<()> {
        visitable.accept(&mut SetProgramIdVisitor::new())?;
        visitable.accept(&mut SetProgramMetadataVisitor::new())?;
        visitable.accept(&mut CombineModulesVisitor::new())?;
        visitable.accept(&mut InstructionResolver::new())?;
        visitable.accept(&mut DefineTypeResolver::new())?;
        Ok(())
    }
}
