use std::ops::DerefMut;

use self::accounts_ix::SplitLock;
use super::*;

#[allow(unused_variables)]

pub fn handle_split_lock(
    ctx: Context<SplitLock>,
    old_lock_id: u64,
    amount_split_into_new_locker: u64,
) -> Result<()> {
    let config_acc = &mut ctx.accounts.config_account;

    let old_locker = &mut ctx.accounts.lp_locker_acc;

    let new_locker = &mut ctx.accounts.new_lp_locker_acc;

    utils::system_program_transfer(
        config_acc.config.fee_config.native_fee,
        &ctx.accounts.system_program,
        &ctx.accounts.payer,
        &ctx.accounts.dev_wallet,
    )?;

    let new_locker_struct = TokenLock {
        lock_global_id: config_acc.config.next_locker_unique_id,
        amm_id: old_locker.amm_id,
        lp_mint: old_locker.lp_mint,
        country_code: old_locker.country_code,
        lock_date: old_locker.lock_date,
        lock_owner: old_locker.lock_owner,
        unlock_date: old_locker.unlock_date,
        initial_lock_amount: amount_split_into_new_locker,
        current_locked_amount: amount_split_into_new_locker,

        //Add calculated bump  to locker  acc
        //Fix : Hardcoded Bump May Lead to Unusable Accounts
        bump: ctx.bumps.new_lp_locker_acc,
    };

    new_locker.set_inner(new_locker_struct);

    old_locker.current_locked_amount -= amount_split_into_new_locker;

    //user info
    let user_lp_tracker_acc = &mut ctx.accounts.user_info_lp_tracker_acc;

    //TODO(aaraN) preaudit : Check if the amount splitted is not the whole amount in that case we
    // wont increase the lp locker count as it negates : status : done

    //Increment locker count in the scenario old locker still has some locked amount and not all
    // amount was withdrawn
    if old_locker.current_locked_amount > 0 {
        let user_lp_info_acc = ctx.accounts.user_info_acc.deref_mut();

        user_lp_info_acc.lp_locker_count += 1;
    }
    //since old locker was withdrawn fully, remove it from the current associated index and add the
    // new locker index
    else {
        //TODO(aaraN) preaudit : if the amount split was whole, remove the prev locker id from the
        // associated locker id : status : done
        user_lp_tracker_acc.find_and_remove_locker_from_tracking(old_locker.lock_global_id)?;

        //no need to increment user lp info acc, as old locker got removed and new added so the
        // result is net zero change to the user lp info acc lp locker count
    }

    //store bump  incase it was created for the first itme
    user_lp_tracker_acc.bump = ctx.bumps.user_info_lp_tracker_acc;

    //add new locker id to the tracker
    user_lp_tracker_acc
        .associated_locker_ids
        .push(config_acc.config.next_locker_unique_id);

    //increment next locker id/ nonce
    config_acc.config.next_locker_unique_id += 1;

    #[cfg(not(feature = "cpi-event"))]
    emit_stack(OnSplitLock {
        lock_id: new_locker.lock_global_id,
        old_lock_id: old_locker.lock_global_id,
        lp_token: new_locker.amm_id,
        owner: new_locker.lock_owner,
        amount_remaining_in_old_lock: old_locker.current_locked_amount,
        amount_removed: amount_split_into_new_locker,
        unlock_date: old_locker.unlock_date,
    });

    #[cfg(feature = "cpi-event")]
    {
        emit_cpi!(OnSplitLock {
            lock_id: new_locker.lock_global_id,
            old_lock_id: old_locker.lock_global_id,
            lp_token: new_locker.amm_id,
            owner: new_locker.lock_owner,
            amount_remaining_in_old_lock: old_locker.current_locked_amount,
            amount_removed: amount_split_into_new_locker,
            unlock_date: old_locker.unlock_date,
        })
    };

    Ok(())
}
