use self::accounts_ix::AdminIx;

use super::*;

#[inline(never)]

pub fn handle_change_owner(ctx: Context<AdminIx>, new_admin_key: Pubkey) -> Result<()> {

    require_keys_neq!(ctx.accounts.config_account.config.admin_key, new_admin_key);

    ctx.accounts.config_account.set_new_admin(new_admin_key);

    Ok(())
}
