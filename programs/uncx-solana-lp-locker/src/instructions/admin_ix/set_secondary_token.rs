use self::accounts_ix::AdminIx;

use super::*;

#[inline(never)]

pub fn handle_set_secondary_token(
    ctx: Context<AdminIx>,
    new_secondary_token: Option<Pubkey>,
) -> Result<()> {

    if let (Some(existing_secondary_token_address), Some(new_secondary_token_address)) = (
        ctx.accounts.config_account.config.secondary_token_address,
        new_secondary_token,
    ) {

        require_keys_neq!(
            existing_secondary_token_address,
            new_secondary_token_address
        );
    };

    ctx.accounts
        .config_account
        .set_secondary_fee_token(new_secondary_token);

    Ok(())
}
