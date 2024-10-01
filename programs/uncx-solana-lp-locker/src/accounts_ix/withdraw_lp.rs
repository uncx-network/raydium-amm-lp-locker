use super::*;

use anchor_spl::token_interface::{
    TokenAccount,
    TokenInterface,
};
use utils::clock_now;

#[cfg_attr(feature = "cpi-event", event_cpi)]
#[derive(Accounts)]
//user
#[instruction(locker_id : u64,withdraw_amount : u64)]

pub struct WithdrawLp<'info> {
    #[account(mut,seeds=[LP_LOCKER_SEED,locker_id.to_le_bytes().as_ref()],bump=lp_locker_acc.bump,constraint = lp_locker_acc.lock_global_id==locker_id,constraint = lp_locker_acc.current_locked_amount>=withdraw_amount @ UncxLpError::InsufficentLockBalanceForWithdrawalError,constraint = withdraw_amount >0 
     @ UncxLpError::InvalidWithdrawAmount,constraint = clock_now().0>
    lp_locker_acc.unlock_date @
    UncxLpError::LpStillLockedError)]
    pub lp_locker_acc: Account<'info, TokenLock>,

    //added "mut" to credit rent amount if applicable
    #[account(mut,constraint = lock_owner.key()==lp_locker_acc.lock_owner @ UncxLpError::OwnerMismatchError)]
    pub lock_owner: Signer<'info>,
    ///CHECK : `its a pda account, checked via its seeds`

    #[account(constraint=uncx_authority_acc.key()==config_account.uncx_authority_pda_address)]
    pub uncx_authority_acc: UncheckedAccount<'info>,

    #[account(mut,seeds=[UNCX_LP_VAULT_ACCOUNT,lp_locker_acc.amm_id.as_ref()],bump,token::mint = lp_locker_acc.lp_mint,token::authority = uncx_authority_acc)]
    pub uncx_lock_lp_vault_acc: InterfaceAccount<'info, TokenAccount>,

    #[account(seeds=[CONFIG_ACCOUNT_SEED],bump=config_account.bump)]
    pub config_account: Account<'info, ConfigurationAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    //user related accounts
    #[account(mut,token::mint=lp_locker_acc.lp_mint)]
    pub user_lp_token_acc: InterfaceAccount<'info, TokenAccount>,

    #[account(mut,seeds=[USER_INFO_SEED,lock_owner.key().as_ref(),lp_locker_acc.amm_id.as_ref()],bump=user_info_acc.bump)]
    pub user_info_acc: Option<Account<'info, UserInfoAccount>>,
    //Security :will be validated that this acc represent the correct lp account tracker
    //without seeds validation by ensuring it contains a specific locker id
    #[account(mut)]
    pub user_info_lp_tracker_acc: Option<Account<'info, UserLpInfoAccount>>,
}
