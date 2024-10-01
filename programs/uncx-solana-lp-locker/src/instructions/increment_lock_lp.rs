use super::accounts_ix::IncrementLockLp;
use super::*;
use lock_lp_ix_helper::{calc_fee_ceil, I80F64ToU64};
use std::ops::Deref;

#[allow(unused_variables)]

pub fn handle_increment_lock_lp(
    ctx: Context<IncrementLockLp>,
    lock_id: u64,
    increment_lock_amount: u64,
) -> Result<()> {
    let locker = &mut ctx.accounts.lp_locker_acc;

    let fee_config = &ctx.accounts.config_account.deref().config.fee_config;

    let precise_liq_fee =
        calc_fee_ceil(increment_lock_amount, fee_config.liquidity_fee_bps)?.conv_to_i64()?;

    debug!("precise liq_fee {}", precise_liq_fee);

    locker.current_locked_amount += increment_lock_amount - precise_liq_fee;

    //transfer increment locker locked amount liquidity fee to dev wallet
    utils::token_transfer(
        precise_liq_fee,
        &ctx.accounts.token_program,
        &ctx.accounts.increment_lock_lp_entity_lp_token_acc,
        &ctx.accounts.dev_lp_token_account,
        &ctx.accounts.increment_lock_lp_entity_authority_acc,
    )?;
    //new locker amount after fee cut to be tranferred to locker lp program vault
    utils::token_transfer(
        increment_lock_amount - precise_liq_fee,
        &ctx.accounts.token_program,
        &ctx.accounts.increment_lock_lp_entity_lp_token_acc,
        &ctx.accounts.uncx_lock_lp_vault_acc,
        &ctx.accounts.increment_lock_lp_entity_authority_acc,
    )?;

    #[cfg(not(feature = "cpi-event"))]
    emit_stack(OnIncrementLock {
        lock_id,
        lp_token: locker.amm_id,
        owner: locker.lock_owner,
        amount_remaining_in_lock: locker.current_locked_amount,
        amount_added: increment_lock_amount,
        liquidity_fee: precise_liq_fee,
        entity: ctx.accounts.increment_lock_lp_entity_authority_acc.key(),
    });

    #[cfg(feature = "cpi-event")]
    {
        emit_cpi!(OnIncrementLock {
            lock_id,
            lp_token: locker.amm_id,
            owner: locker.lock_owner,
            amount_remaining_in_lock: locker.current_locked_amount,
            amount_added: increment_lock_amount,
            liquidity_fee: precise_liq_fee,
            entity: ctx.accounts.increment_lock_lp_entity_authority_acc.key(),
        })
    };

    Ok(())
}
