use solana_program_error::{ProgramError, ToStr};

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorCode {
    UnknownInstruction = 100,
    AccountNotSigner,
    AccountDiscriminatorMismatch,
    HasOneConstraint,
    AssertConstraint,
    AddressConstraint,
    TryingToInitPayerAsProgramAccount,
    TokenConstraintViolated,
    BufferFull,
    InvalidReturnData,
    InvalidDataLength,
    InvalidDataAlignment,
    AccountNotMutable,
}

impl TryFrom<u32> for ErrorCode {
    type Error = ProgramError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            100 => Ok(ErrorCode::UnknownInstruction),
            101 => Ok(ErrorCode::AccountNotSigner),
            102 => Ok(ErrorCode::AccountDiscriminatorMismatch),
            103 => Ok(ErrorCode::HasOneConstraint),
            104 => Ok(ErrorCode::AssertConstraint),
            105 => Ok(ErrorCode::AddressConstraint),
            106 => Ok(ErrorCode::TryingToInitPayerAsProgramAccount),
            107 => Ok(ErrorCode::TokenConstraintViolated),
            108 => Ok(ErrorCode::BufferFull),
            109 => Ok(ErrorCode::InvalidReturnData),
            110 => Ok(ErrorCode::InvalidDataLength),
            111 => Ok(ErrorCode::InvalidDataAlignment),
            112 => Ok(ErrorCode::AccountNotMutable),
            _ => Err(ProgramError::InvalidArgument),
        }
    }
}

impl From<ErrorCode> for ProgramError {
    fn from(e: ErrorCode) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl ToStr for ErrorCode {
    fn to_str(&self) -> &'static str {
        match self {
            ErrorCode::UnknownInstruction => "Error: Unknown instruction",
            ErrorCode::AccountNotSigner => "Error: Account is not a signer",
            ErrorCode::AccountDiscriminatorMismatch => {
                "Error: Discriminator did not match what was expected"
            }
            ErrorCode::HasOneConstraint => "Error: has_one constraint violated",
            ErrorCode::AssertConstraint => "Error: assert constraint violated",
            ErrorCode::AddressConstraint => "Error: address constraint violated",
            ErrorCode::TryingToInitPayerAsProgramAccount => {
                "Error: Cannot initialize a program account with the payer account"
            }
            ErrorCode::TokenConstraintViolated => "Error: Token constraint was violated",
            ErrorCode::BufferFull => "Error: Buffer is full",
            ErrorCode::InvalidReturnData => "Error: The return data is invalid",
            ErrorCode::InvalidDataLength => "Error: Invalid data length",
            ErrorCode::InvalidDataAlignment => "Error: Invalid data alignment",
            ErrorCode::AccountNotMutable => "Error: Account is not mutable",
        }
    }
}
