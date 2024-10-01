use self::accounts_ix::AdminIx;

use super::*;

#[inline(never)]

pub fn handle_set_fees(ctx: Context<AdminIx>, new_fees_data: FeesConfig) -> Result<()> {

    ctx.accounts.config_account.set_fees_config(new_fees_data)?;

    Ok(())
}
