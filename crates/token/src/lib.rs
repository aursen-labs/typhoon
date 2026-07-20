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

impl<'a> Accessor<'a, Mint> for SplStrategy {
    type Data = &'a Mint;

    #[inline(always)]
    fn access(data: &'a [u8]) -> Result<Self::Data, ProgramError> {
        // SAFETY: `Mint` is `#[repr(transparent)]` over `SplMint`,
        // so the reference cast preserves layout/alignment/lifetime. The caller
        // must also guarantee `data` encodes a valid token account state.
        Ok(unsafe { transmute::<&SplMint, &Mint>(SplMint::from_bytes_unchecked(data)) })
    }

    #[inline(always)]
    fn read(data: &mut &'a [u8]) -> Result<Self::Data, ProgramError> {
        let Some((to_read, rem)) = data.split_at_checked(Mint::LEN) else {
            return Err(ProgramError::InvalidInstructionData);
        };
        *data = rem;
        <Self as Accessor<Mint>>::access(to_read)
    }
}

impl<'a> Accessor<'a, TokenAccount> for SplStrategy {
    type Data = &'a TokenAccount;

    #[inline(always)]
    fn access(data: &'a [u8]) -> Result<Self::Data, ProgramError> {
        // SAFETY: `TokenAccount` is `#[repr(transparent)]` over `SplTokenAccount`,
        // so the reference cast preserves layout/alignment/lifetime. The caller
        // must also guarantee `data` encodes a valid token account state.
        Ok(unsafe {
            transmute::<&SplTokenAccount, &TokenAccount>(SplTokenAccount::from_bytes_unchecked(
                data,
            ))
        })
    }

    #[inline(always)]
    fn read(data: &mut &'a [u8]) -> Result<Self::Data, ProgramError> {
        let Some((to_read, rem)) = data.split_at_checked(TokenAccount::LEN) else {
            return Err(ProgramError::InvalidInstructionData);
        };
        *data = rem;
        <Self as Accessor<TokenAccount>>::access(to_read)
    }
}

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
