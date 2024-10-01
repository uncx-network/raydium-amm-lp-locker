use self::accounts_ix::AddMigrator;

use super::*;

pub fn handle_add_migrator(ctx: Context<AddMigrator>) -> Result<()> {

    let migrator_acc = &mut ctx.accounts.migrator_marker_acc;

    migrator_acc.bump = ctx.bumps.migrator_marker_acc;

    Ok(())
}
