use self::accounts_ix::AdminIx;

use super::*;

#[inline(never)]

pub fn handle_add_country_to_blacklist(ctx: Context<AdminIx>, country_code_to_add: u8) -> Result<()> {
    ctx.accounts
        .config_account
        .add_country_to_blacklist(country_code_to_add)
}

#[inline(never)]

pub fn handle_remove_country_from_blacklist(
    ctx: Context<AdminIx>,
    country_code_to_remove: u8,
) -> Result<()> {
    ctx.accounts
        .config_account
        .remove_country_from_blacklist(country_code_to_remove)
}
