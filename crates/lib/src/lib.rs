#![no_std]

pub mod macros {
    pub use {
        typhoon_account_macro::*, typhoon_context_macro::*, typhoon_cpi_generator_macro::*,
        typhoon_errors_macro::*, typhoon_program_id_macro::*,
    };
}

pub mod lib {
    pub use {
        typhoon_accounts::*, typhoon_context::*, typhoon_errors::*, typhoon_traits::*,
        typhoon_utility_traits::*,
    };
}

pub mod bytes {
    pub use typhoon_utility::bytes::*;
}

pub mod instruction {
    pub use pinocchio::instruction::{InstructionAccount, InstructionView};
}

pub type ProgramResult<T = ()> = Result<T, typhoon_errors::Error>;

/// Derives a program address and bump seed from `seeds` for `program_id` in const context.
pub const fn find_program_address_const(
    seeds: &[&[u8]],
    program_id: &solana_address::Address,
) -> (solana_address::Address, u8) {
    let (bytes, bump) = const_crypto::ed25519::derive_program_address(seeds, program_id.as_array());
    (solana_address::Address::new_from_array(bytes), bump)
}

pub mod prelude {
    #[cfg(feature = "alloc")]
    pub use pinocchio::default_allocator;
    #[cfg(feature = "logging")]
    pub use typhoon_errors::{log_error, LogError};
    pub use {
        super::{bytes, find_program_address_const, instruction, lib::*, macros::*, ProgramResult},
        pinocchio::{
            self,
            address::{self, address_eq, declare_id, MAX_SEEDS},
            cpi::{self, Seed, Signer as CpiSigner},
            default_panic_handler,
            entrypoint::{InstructionContext, MaybeAccount},
            error::{ProgramError, ToStr},
            hint,
            instruction::seeds,
            lazy_program_entrypoint, no_allocator, nostd_panic_handler, program_entrypoint,
            sysvars::{clock::Clock, fees::Fees, rent::Rent, Sysvar},
            AccountView, Address,
        },
    };
}
