use self::accounts_ix::AddWhitelistAcc;

use super::*;

pub fn handle_add_whitelist(ctx: Context<AddWhitelistAcc>) -> Result<()> {

    let whitelist_acc = &mut ctx.accounts.user_whitelist_pda_acc;

    whitelist_acc.bump = ctx.bumps.user_whitelist_pda_acc;

    Ok(())
}
