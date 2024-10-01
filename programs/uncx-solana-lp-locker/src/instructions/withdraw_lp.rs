use std::ops::DerefMut;

use self::accounts_ix::WithdrawLp;
use super::*;
use utils::clean_locker_from_tracker_and_close;

// #[allow(unused_variables)]
/// withdraw a specified amount from a lock

pub fn handle_withdraw_lp(
    ctx: Context<WithdrawLp>,
    lock_id: u64,
    withdraw_amount: u64,
) -> Result<()> {
    let locker = ctx.accounts.lp_locker_acc.deref_mut();

    let config_acc = &ctx.accounts.config_account;

    locker.current_locked_amount -= withdraw_amount;

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
        withdraw_amount,
        &ctx.accounts.token_program,
        &ctx.accounts.uncx_lock_lp_vault_acc,
        &ctx.accounts.user_lp_token_acc,
        &ctx.accounts.uncx_authority_acc,
        uncx_authority_seeds,
    )?;

    #[cfg(not(feature = "cpi-event"))]
    emit_stack(OnWithdraw {
        lock_id,
        lp_token: locker.amm_id,
        owner: locker.lock_owner,
        amount_remaining_in_lock: locker.current_locked_amount,
        amount_removed: withdraw_amount,
    });

    #[cfg(feature = "cpi-event")]
    {
        emit_cpi!(OnWithdraw {
            lock_id,
            lp_token: locker.amm_id,
            owner: locker.lock_owner,
            amount_remaining_in_lock: locker.current_locked_amount,
            amount_removed: withdraw_amount,
        })
    };

    Ok(())
}
