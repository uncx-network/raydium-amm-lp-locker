use std::ops::DerefMut;

use self::accounts_ix::TransferLockOwnership;
use super::*;
use utils::clean_locker_from_tracker_and_close;

#[allow(unused_variables)]

pub fn handle_transfer_lock_ownership(
    ctx: Context<TransferLockOwnership>,
    lock_id: u64,
    new_owner: Pubkey,
) -> Result<()> {

    let locker = ctx.accounts.lp_locker_acc.deref_mut();

    let old_owner = locker.lock_owner;

    let old_user_lp_info_acc = &mut ctx.accounts.old_user_info_acc;

    let old_user_lp_tracker_acc = &mut ctx.accounts.old_user_info_lp_tracker_acc;

    let new_user_lp_info_acc = &mut ctx.accounts.new_user_info_acc;

    let new_user_lp_tracker_acc = &mut ctx.accounts.new_user_info_lp_tracker_acc;

    //new user stuff
    let new_user_info_acc_struct = UserInfoAccount {
        bump: ctx.bumps.new_user_info_acc,
        lp_mint: old_user_lp_info_acc.lp_mint,
        lp_locker_count: new_user_lp_info_acc.lp_locker_count + 1,
        user: new_owner,
    };

    new_user_lp_info_acc.set_inner(new_user_info_acc_struct);

    new_user_lp_tracker_acc.bump = ctx.bumps.new_user_info_lp_tracker_acc;

    new_user_lp_tracker_acc.associated_locker_ids.push(lock_id);

    //transfer lock ownership
    locker.lock_owner = new_owner;

    clean_locker_from_tracker_and_close(
        old_user_lp_tracker_acc,
        old_user_lp_info_acc,
        ctx.accounts.lock_owner.to_account_info(),
        lock_id,
    )?;

    #[cfg(not(feature = "cpi-event"))]
    emit_stack(OnTransferLockOwnership {
        lock_id,
        lp_token: locker.amm_id,
        old_owner,
        new_owner,
    });

    #[cfg(feature = "cpi-event")]
    {

        emit_cpi!(OnTransferLockOwnership {
            lock_id,
            lp_token: locker.amm_id,
            old_owner,
            new_owner,
        })
    };

    Ok(())
}
