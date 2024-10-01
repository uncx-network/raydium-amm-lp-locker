use anchor_lang::prelude::*;

#[error_code]

pub enum UncxLpError {
    #[msg("Custom error message")]
    CustomError,
    #[msg("Deserialization Failed")]
    ZeroCopyDerializationFailed,
    #[msg("Invalid Percentage, BPS % is between 1-10000")]
    InvalidPercentage,
    RaydiumError,
    #[msg("Invalid Unlock Time Bound")]
    InvalidTimeBoundError,
    #[msg("Only Nonzero Amount Allowed")]
    ZeroAmountError,
    #[msg("Black Listed Country Detected")]
    BlackListedCountryError,
    #[msg("Account not needed passed in")]
    InvalidAccountError,
    #[msg("Required Account not passed in")]
    MissingRequiredAccount,
    #[msg("Relock Unlock Date must be greater than the existing lock date.")]
    RelockUnlockDateInvalid,
    #[msg("Relocking new unlock date should be in a future")]
    InvalidUnlockDateError,
    #[msg("Owner Mismatch")]
    OwnerMismatchError,
    #[msg("Insufficient Lock Amount")]
    InsufficentLockBalanceForWithdrawalError,
    #[msg("Only Non-zero withdrawals Allowed")]
    InvalidWithdrawAmount,
    #[msg("Lp Still Locked")]
    LpStillLockedError,
    #[msg("Invalid lp tracker acc")]
    MissingLockerId,
    #[msg("Fee Payment Method is whitelisted but no whitelist acc provided")]
    MissingWhitelistAccount,
    #[msg("Missing Required Referral Accounts for referral fee eligiblility")]
    MissingReferralAccount,
    #[msg("insufficient referral balance")]
    InsufficientReferralBalance,
    #[msg("u64 to u128 conversion failed")]
    ConversionError,
    #[msg("Math Error")]
    MathError,
    #[msg("Country Code does not Exist")]
    CountryCodeNotPresent,
    #[msg("Country Code Already Exists")]
    CountryCodeAlreadyExists,
    #[msg("Invalid Token Metadta")]
    InvalidTokenMetadata,
    #[msg("Amms sharing liquidity with openbook enabled are not supported")]
    OpenBookAmmNotSupported,
    #[msg("Invalid Associated RaydiumV4Amm Accounts")]
    InvalidRaydiumV4Accounts,
}

impl From<UncxLpError> for ProgramError {
    fn from(error: UncxLpError) -> Self {
        ProgramError::from(Error::from(error))
    }
}
