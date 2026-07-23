mod helpers;
pub mod plugin;
mod utils;
pub mod visitors;

use {
    crate::plugin::TyphoonPlugin,
    codama::{Codama, CodamaResult, NodeTrait},
    std::path::Path,
};

pub fn generate(crates: &[&Path]) -> CodamaResult<String> {
    let codama = Codama::load_all(crates)
        .map_err(|_| codama::CodamaError::NodeNotFound)?
        .without_default_plugin()
        .add_plugin(TyphoonPlugin);

    let mut node = codama.get_idl()?;

    for mut program in node.additional_programs.drain(..) {
        if node.program.public_key.is_empty() {
            node.program.public_key = program.public_key.clone();
        }
        node.program.errors.append(&mut program.errors);
        node.program
            .defined_types
            .append(&mut program.defined_types);
        node.program.accounts.append(&mut program.accounts);
        node.program.events.append(&mut program.events);
    }

    node.to_json()
}
