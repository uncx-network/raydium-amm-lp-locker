use std::ops::DerefMut;

use super::*;
use utils::clean_locker_from_tracker_and_close;

#[allow(unused_variables)]

pub fn handle_migrate_lp(
    ctx: Context<MigrateLp>,
    lock_id: u64,
    migrate_amount: u64,
    migration_option: u16,
) -> Result<()> {

    let locker = ctx.accounts.lp_locker_acc.deref_mut();

    let config_acc = &ctx.accounts.config_account;

    //already checked in the constrainrs : current amount >- migrate_amount
    locker.current_locked_amount -= migrate_amount;

    //remove locker id reclaim space
    if locker.current_locked_amount == 0 {

        let (Some(user_lp_tracker_acc), Some(user_info_acc)) = (
            &mut ctx.accounts.user_info_lp_tracker_acc,
            &mut ctx.accounts.user_info_acc,
        ) else {

            return err!(UncxLpError::MissingRequiredAccount);
        };

        clean_locker_from_tracker_and_close(
            user_lp_tracker_acc,
            user_info_acc,
            ctx.accounts.lock_owner.to_account_info(),
            lock_id,
        )?;
    }

    let uncx_authority_seeds = &[
        UNCX_LOCKER_AUTHORITY_SEED,
        &[config_acc.uncx_authority_bump],
    ];

    //transfer withdrawel lp to the user lp mint acc
    utils::token_transfer_signed(
        migrate_amount,
        &ctx.accounts.token_program,
        &ctx.accounts.uncx_lock_lp_vault_acc,
        &ctx.accounts.migrator_token_lp_account,
        &ctx.accounts.uncx_authority_acc,
        uncx_authority_seeds,
    )?;

    #[cfg(not(feature = "cpi-event"))]
    emit_stack(OnMigrate {
        lock_id,
        lp_token: locker.amm_id,
        owner: locker.lock_owner,
        amount_remaining_in_lock: locker.current_locked_amount,
        amount_migrated: migrate_amount,
        migration_option,
    });

    #[cfg(feature = "cpi-event")]
    {

        emit_cpi!(OnMigrate {
            lock_id,
            lp_token: locker.amm_id,
            owner: locker.lock_owner,
            amount_remaining_in_lock: locker.current_locked_amount,
            amount_migrated: migrate_amount,
            migration_option,
        })
    };

    Ok(())
}
