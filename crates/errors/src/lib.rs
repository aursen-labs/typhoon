#![no_std]

mod error_code;
mod extension;

use {
    core::num::NonZeroU64,
    solana_address::error::AddressError,
    solana_program_error::{ProgramError, ToStr},
};
pub use {error_code::*, extension::*};

pub struct Error {
    code: NonZeroU64,
    #[cfg(feature = "logging")]
    account_name: Option<&'static str>,
}

impl Error {
    #[inline(always)]
    pub fn new(error: impl Into<ProgramError>) -> Self {
        error.into().into()
    }

    #[cfg(feature = "logging")]
    #[inline(always)]
    pub fn with_account(mut self, name: &'static str) -> Self {
        self.account_name = Some(name);
        self
    }

    #[cfg(not(feature = "logging"))]
    #[inline(always)]
    pub fn with_account(self, _name: &'static str) -> Self {
        self
    }

    pub fn account_name(&self) -> Option<&str> {
        #[cfg(feature = "logging")]
        {
            self.account_name
        }
        #[cfg(not(feature = "logging"))]
        {
            None
        }
    }

    pub fn to_str<E>(&self) -> &'static str
    where
        E: ToStr + TryFrom<u32> + 'static,
    {
        let error = ProgramError::from(self.code.get());
        if let ProgramError::Custom(code) = error {
            if (100..200).contains(&code) {
                return error.to_str::<ErrorCode>();
            }
        }
        error.to_str::<E>()
    }

    #[inline(always)]
    fn from_code(code: u64) -> Self {
        Error {
            code: NonZeroU64::new(code).unwrap_or(NonZeroU64::MAX),
            #[cfg(feature = "logging")]
            account_name: None,
        }
    }
}

impl From<ProgramError> for Error {
    #[inline(always)]
    fn from(error: ProgramError) -> Self {
        Self::from_code(u64::from(error))
    }
}

impl From<ErrorCode> for Error {
    #[inline(always)]
    fn from(value: ErrorCode) -> Self {
        Self::from_code(value as u64)
    }
}

impl From<Error> for ProgramError {
    fn from(value: Error) -> Self {
        ProgramError::from(value.code.get())
    }
}

impl From<Error> for u64 {
    #[inline(always)]
    fn from(value: Error) -> Self {
        value.code.get()
    }
}

impl From<AddressError> for Error {
    #[inline(always)]
    fn from(value: AddressError) -> Self {
        Self::new(ProgramError::from(value))
    }
}

#[cfg(feature = "logging")]
pub type LogError = ErrorCode;

#[cfg(feature = "logging")]
#[cold]
pub fn log_error<E>(error: &Error)
where
    E: ToStr + TryFrom<u32> + 'static,
{
    solana_program_log::log(error.to_str::<E>());

    if let Some(account_name) = error.account_name() {
        let mut logger = solana_program_log::Logger::<50>::default();
        logger.append("Account origin: ");
        logger.append(unsafe { str::from_utf8_unchecked(account_name.as_bytes()) });
        logger.log();
    }
}

#[macro_export]
macro_rules! require {
    ( $constraint:expr, $error:expr ) => {
        if pinocchio::hint::unlikely(!$constraint) {
            return Err($error.into());
        }
    };
}
