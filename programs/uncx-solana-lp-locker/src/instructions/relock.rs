use self::accounts_ix::RelockLp;
use super::*;
use lock_lp_ix_helper::{calc_fee_ceil, I80F64ToU64};
use std::ops::Deref;

/// extend a lock with a new unlock date

pub fn handle_relock_lp(ctx: Context<RelockLp>, lock_id: u64, new_unlock_date: i64) -> Result<()> {
    require!(
        new_unlock_date < INVALID_TIME_BOUND,
        UncxLpError::InvalidTimeBoundError
    );

    require_gt!(
        new_unlock_date,
        ctx.accounts.lp_locker_acc.unlock_date,
        UncxLpError::RelockUnlockDateInvalid
    );

    require_gt!(
        new_unlock_date,
        utils::clock_now().0,
        UncxLpError::InvalidUnlockDateError
    );

    let locker = &mut ctx.accounts.lp_locker_acc;

    let fee_config = &ctx.accounts.config_account.deref().config.fee_config;
    let precise_liq_fee =
        calc_fee_ceil(locker.current_locked_amount, fee_config.liquidity_fee_bps)?.conv_to_i64()?;

    debug!("precise liq_fee {}", precise_liq_fee);

    locker.current_locked_amount -= precise_liq_fee;

    locker.unlock_date = new_unlock_date;

    let uncx_authority_seeds = &[
        UNCX_LOCKER_AUTHORITY_SEED,
        &[ctx.accounts.config_account.uncx_authority_bump],
    ];

    //transfer relock liquidity fee to dev wallet
    utils::token_transfer_signed(
        precise_liq_fee,
        &ctx.accounts.token_program,
        &ctx.accounts.uncx_lock_lp_vault_acc,
        &ctx.accounts.dev_lp_token_account,
        &ctx.accounts.uncx_authority_acc,
        uncx_authority_seeds,
    )?;

    #[cfg(not(feature = "cpi-event"))]
    emit_stack(OnRelock {
        lock_id,
        lp_token: locker.amm_id,
        owner: locker.lock_owner,
        amount_remaining_in_lock: locker.current_locked_amount,
        liquidity_fee: precise_liq_fee,
        unlock_date: new_unlock_date,
    });

    #[cfg(feature = "cpi-event")]
    {
        emit_cpi!(OnRelock {
            lock_id,
            lp_token: locker.amm_id,
            owner: locker.lock_owner,
            amount_remaining_in_lock: locker.current_locked_amount,
            liquidity_fee: precise_liq_fee,
            unlock_date: new_unlock_date,
        })
    };

    Ok(())
}
