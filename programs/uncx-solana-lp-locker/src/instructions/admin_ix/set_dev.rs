use self::accounts_ix::AdminIx;

use super::*;

#[inline(never)]

pub fn handle_set_dev(ctx: Context<AdminIx>, new_dev_addr: Pubkey) -> Result<()> {

    require_keys_neq!(ctx.accounts.config_account.config.dev_addr, new_dev_addr);

    ctx.accounts
        .config_account
        .set_developer_address(new_dev_addr);

    Ok(())
}
