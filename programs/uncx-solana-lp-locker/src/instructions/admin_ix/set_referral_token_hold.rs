use self::accounts_ix::AdminIx;

use super::*;

#[inline(never)]

pub fn handle_set_referral_token_and_min_balance(
    ctx: Context<AdminIx>,
    new_referral_token_address: Option<Pubkey>,
    new_min_balance: u64,
) -> Result<()> {

    if let (Some(existing_referral_token_address), Some(new_referral_address)) = (
        ctx.accounts.config_account.config.referral_token_address,
        new_referral_token_address,
    ) {

        require_keys_neq!(existing_referral_token_address, new_referral_address);
    };

    ctx.accounts
        .config_account
        .set_referral_token_and_min_hold_balance(new_referral_token_address, new_min_balance);

    Ok(())
}
