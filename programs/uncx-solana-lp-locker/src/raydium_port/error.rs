use anchor_lang::prelude::*;

#[error_code]

pub enum RaydiumAmmError {
    #[msg("Error: The amm account owner does not  match with the canonical rayium  amm")]
    InvalidAmmAccountOwner,
    #[msg("Error: Size Mismatch")]
    ExpectedAccount,
    #[msg("InvalidStatus")]
    InvalidStatus,
    InvalidFee,
    InvalidState,
    InvalidTargetAccountOwner,
    InvalidTargetOwner,
    CheckedSubOverflow,
    CalcPnlError,
    ConversionFailure,
}

impl From<RaydiumAmmError> for ProgramError {
    fn from(value: RaydiumAmmError) -> Self {
        ProgramError::from(Error::from(value))
    }
}
