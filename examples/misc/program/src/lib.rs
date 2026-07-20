#![no_std]

pub mod intructions;

use {intructions::*, typhoon::prelude::*};

nostd_panic_handler!();
no_allocator!();
entrypoint!(ROUTER);

pub const ROUTER: EntryFn = basic_router! {
    0 => account_iter,
    1 => initialize,
    2 => assert
};
