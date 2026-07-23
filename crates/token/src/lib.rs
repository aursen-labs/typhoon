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
                // A mint (82 bytes) and a token account (165 bytes) share the same
                // owner and have no discriminator, so the buffer length is the
                // only thing that distinguishes them. Without this guard,
                // `from_bytes_unchecked` would read past the end of a too-small buffer
                // (out-of-bounds) and a mint could be deserialized as a token account
                // (and vice versa).
                #[cfg(not(feature = "token2022"))]
                if data.len() != <$wrapper>::LEN {
                    return Err(ProgramError::InvalidAccountData);
                }
                // Token-2022 accounts may carry trailing extension data, so only
                // require that the base state fits.
                #[cfg(feature = "token2022")]
                if data.len() < <$wrapper>::LEN {
                    return Err(ProgramError::InvalidAccountData);
                }

                // SAFETY: `data` holds at least `LEN` bytes (checked above), and
                // `$wrapper` is `#[repr(transparent)]` over `$inner`, so the reference
                // cast preserves layout/alignment/lifetime.
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

#[cfg(test)]
mod tests {
    use {
        super::{Mint, SplStrategy, TokenAccount},
        typhoon_traits::Accessor,
    };

    fn access_mint(data: &[u8]) -> bool {
        <SplStrategy as Accessor<'_, Mint>>::access(data).is_ok()
    }

    fn access_token(data: &[u8]) -> bool {
        <SplStrategy as Accessor<'_, TokenAccount>>::access(data).is_ok()
    }

    #[test]
    fn access_accepts_correct_sizes() {
        assert!(access_mint(&[0u8; Mint::LEN]));
        assert!(access_token(&[0u8; TokenAccount::LEN]));
    }

    #[test]
    fn access_rejects_undersized_buffers() {
        // Holds regardless of the `token2022` feature: a buffer smaller than the
        // base state must be rejected rather than read out of bounds. A
        // mint-sized buffer (82 bytes) is smaller than a token account (165), so
        // it must never deserialize as one — the type-confusion case.
        assert!(!access_mint(&[0u8; 10]));
        assert!(!access_token(&[0u8; 10]));
        assert!(!access_mint(&[]));
        assert!(!access_token(&[0u8; Mint::LEN]));
    }

    // Without the `token2022` feature, classic SPL accounts have exact fixed
    // sizes, so an oversized buffer (e.g. a token account read as a mint) is
    // rejected too. With `token2022`, trailing extension bytes are allowed.
    #[cfg(not(feature = "token2022"))]
    #[test]
    fn access_rejects_oversized_buffer_without_token2022() {
        assert!(!access_mint(&[0u8; TokenAccount::LEN]));
    }
}
