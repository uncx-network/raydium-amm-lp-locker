use crate::constants::MAX_BASIS_POINTS;
use crate::math_error;
use crate::UncxLpError;
use fixed::types::I80F48;
pub(crate) struct Settlement {
    //actual token amount locked
    pub(crate) lock_amount: u64,
    //lp fee amount given to dev wallet
    pub(crate) lp_fee_amount: u64,
    //native or non-native represents referral fee in token or native
    pub(crate) referral_share_fee: u64,
    //if non-native fee payment, represents burn amount
    pub(crate) secondary_token_burn_amount: u64,
    //represents native dev fee
    pub(crate) dev_fee: u64,
}

#[cfg_attr(feature = "testing", derive(Debug, PartialEq, Eq))]
#[derive(Default)]

pub(crate) struct PreciseSettlement {
    //actual token amount locked
    pub(crate) lock_amount: I80F48,
    //lp fee amount given to dev wallet
    pub(crate) lp_fee_amount: I80F48,
    //native or non-native represents referral fee in token or native
    pub(crate) referral_share_fee: I80F48,
    //if non-native fee payment, represents burn amount
    pub(crate) secondary_token_burn_amount: I80F48,
    //represents native dev fee
    pub(crate) dev_fee: I80F48,
}

impl TryFrom<PreciseSettlement> for Settlement {
    type Error = UncxLpError;

    fn try_from(value: PreciseSettlement) -> std::result::Result<Self, UncxLpError> {
        Ok(Self {
            lock_amount: value.lock_amount.conv_to_i64()?,
            lp_fee_amount: value.lp_fee_amount.conv_to_i64()?,
            referral_share_fee: value.referral_share_fee.conv_to_i64()?,
            secondary_token_burn_amount: value.secondary_token_burn_amount.conv_to_i64()?,
            dev_fee: value.dev_fee.conv_to_i64()?,
        })
    }
}

pub(crate) fn calc_fee_ceil<T, D>(amnt: T, fee: D) -> Result<I80F48, UncxLpError>
where
    T: Into<I80F48> + Copy,
    D: Into<I80F48> + Eq + Copy,
{
    if fee.into() == I80F48::ZERO {
        return Ok(I80F48::ZERO);
    }
    amnt
        .into()
        .checked_mul(fee.into())
        .ok_or_else(math_error!())?
        .checked_div(MAX_BASIS_POINTS.into())
        .ok_or_else(math_error!())?
        .checked_ceil()
        .ok_or_else(math_error!())
}
pub(crate) fn calc_fee<T, D>(amnt: T, fee: D) -> Result<I80F48, UncxLpError>
where
    T: Into<I80F48> + Copy,
    D: Into<I80F48> + Eq + Copy,
{
    if fee.into() == I80F48::ZERO {
        return Ok(I80F48::ZERO);
    }
    amnt
        .into()
        .checked_mul(fee.into())
        .ok_or_else(math_error!())?
        .checked_div(MAX_BASIS_POINTS.into())
        .ok_or_else(math_error!())
}

pub(crate) fn calc_total_after_fee_cuts_ceil(
    amnt_before_cuts: I80F48,
    fee_costs: I80F48,
) -> Result<I80F48, UncxLpError> {
    amnt_before_cuts
        .checked_sub(fee_costs)
        .ok_or_else(math_error!())?
        .checked_ceil()
        .ok_or_else(math_error!())
}
pub(crate) fn calc_total_after_fee_cuts(
    amnt_before_cuts: I80F48,
    fee_costs: I80F48,
) -> Result<I80F48, UncxLpError> {
    amnt_before_cuts
        .checked_sub(fee_costs)
        .ok_or_else(math_error!())
}
pub(crate) trait I80F64ToU64 {
    fn conv_to_i64(self) -> Result<u64, UncxLpError>;
}
impl I80F64ToU64 for I80F48 {
    fn conv_to_i64(self) -> Result<u64, UncxLpError> {
        self.checked_to_num::<u64>().ok_or_else(math_error!())
    }
}
