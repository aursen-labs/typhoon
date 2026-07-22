#![no_std]

use {
    core::{mem::transmute, ops::Deref},
    pinocchio::error::ProgramError,
    pinocchio_associated_token_account::ID as ATA_PROGRAM_ID,
    pinocchio_token::{
        state::{Account as SplTokenAccount, Mint as SplMint},
        ID as TOKEN_PROGRAM_ID,
    },
    solana_address::{address_eq, Address},
    typhoon_traits::{Accessor, CheckOwner, CheckProgramId, DataStrategy, Discriminator},
};

mod traits;

pub use {
    pinocchio_associated_token_account::instructions as ata_instructions,
    pinocchio_token::instructions as spl_instructions, traits::*,
};

#[cfg(feature = "token2022")]
const TOKEN_2022_PROGRAM_ID: Address =
    Address::from_str_const("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

pub struct AtaTokenProgram;

impl CheckProgramId for AtaTokenProgram {
    #[inline(always)]
    fn address_eq(program_id: &Address) -> bool {
        address_eq(program_id, &ATA_PROGRAM_ID)
    }
}

pub struct TokenProgram;

impl CheckProgramId for TokenProgram {
    #[inline(always)]
    fn address_eq(program_id: &Address) -> bool {
        #[cfg(feature = "token2022")]
        {
            address_eq(program_id, &TOKEN_PROGRAM_ID)
                || address_eq(program_id, &TOKEN_2022_PROGRAM_ID)
        }
        #[cfg(not(feature = "token2022"))]
        {
            address_eq(program_id, &TOKEN_PROGRAM_ID)
        }
    }
}

pub struct SplStrategy;

// `$wrapper` is `#[repr(transparent)]` over `$inner`, so the reference cast in
// `access` preserves layout/alignment/lifetime.
macro_rules! impl_accessor {
    ($wrapper:ty, $inner:ty) => {
        impl<'a> Accessor<'a, $wrapper> for SplStrategy {
            type Data = &'a $wrapper;

            #[inline(always)]
            fn access(data: &'a [u8]) -> Result<Self::Data, ProgramError> {
                // SAFETY: the caller must guarantee `data` encodes a valid account state.
                Ok(
                    unsafe {
                        transmute::<&$inner, &$wrapper>(<$inner>::from_bytes_unchecked(data))
                    },
                )
            }

            #[inline(always)]
            fn read(data: &mut &'a [u8]) -> Result<Self::Data, ProgramError> {
                let Some((to_read, rem)) = data.split_at_checked(<$wrapper>::LEN) else {
                    return Err(ProgramError::InvalidInstructionData);
                };
                *data = rem;
                <Self as Accessor<$wrapper>>::access(to_read)
            }
        }
    };
}

impl_accessor!(Mint, SplMint);
impl_accessor!(TokenAccount, SplTokenAccount);

#[repr(transparent)]
pub struct Mint(SplMint);

impl Mint {
    pub const LEN: usize = SplMint::LEN;
}

impl DataStrategy for Mint {
    type Strategy = SplStrategy;
}

impl Discriminator for Mint {
    const DISCRIMINATOR: &'static [u8] = &[];
}

impl CheckOwner for Mint {
    #[inline(always)]
    fn owned_by(program_id: &Address) -> bool {
        TokenProgram::address_eq(program_id)
    }
}

impl Deref for Mint {
    type Target = SplMint;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[repr(transparent)]
pub struct TokenAccount(SplTokenAccount);

impl TokenAccount {
    pub const LEN: usize = SplTokenAccount::LEN;
}

impl DataStrategy for TokenAccount {
    type Strategy = SplStrategy;
}

impl Discriminator for TokenAccount {
    const DISCRIMINATOR: &'static [u8] = &[];
}

impl CheckOwner for TokenAccount {
    #[inline(always)]
    fn owned_by(program_id: &Address) -> bool {
        TokenProgram::address_eq(program_id)
    }
}

impl Deref for TokenAccount {
    type Target = SplTokenAccount;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn find_associated_token_address(mint: &Address, owner: &Address) -> Address {
    Address::find_program_address(
        &[owner.as_ref(), TOKEN_PROGRAM_ID.as_ref(), mint.as_ref()],
        &ATA_PROGRAM_ID,
    )
    .0
}
