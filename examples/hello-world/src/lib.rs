#![no_std]

use typhoon::prelude::*;

program_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

nostd_panic_handler!();
no_allocator!();
entrypoint!(ROUTER);

pub const ROUTER: EntryFn = basic_router! {
    0 => hello_world
};

pub fn hello_world(ProgramIdArg(program_id): ProgramIdArg) -> ProgramResult {
    solana_msg::sol_log("Hello World");

    assert_eq!(program_id, &crate::ID);

    Ok(())
}
