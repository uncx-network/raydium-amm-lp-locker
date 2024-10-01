use super::*;

use crate::constants::DISCRIMINATOR_BYTES_SIZE;

#[cfg_attr(feature = "cpi-event", event_cpi)]
#[derive(Accounts)]
//user
#[instruction(old_locker_id : u64,new_locker_locked_amount:u64)]

pub struct SplitLock<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut,seeds=[LP_LOCKER_SEED,old_locker_id.to_le_bytes().as_ref()],bump=lp_locker_acc.bump,constraint = lp_locker_acc.lock_global_id==old_locker_id,constraint = lp_locker_acc.current_locked_amount>=new_locker_locked_amount @ UncxLpError::InvalidWithdrawAmount,constraint= new_locker_locked_amount>0 @ UncxLpError::ZeroAmountError)]
    pub lp_locker_acc: Account<'info, TokenLock>,

    #[account(init,payer=payer,seeds=[LP_LOCKER_SEED,config_account.config.next_locker_unique_id.to_le_bytes().as_ref()],bump,space=DISCRIMINATOR_BYTES_SIZE+TokenLock::INIT_SPACE)]
    pub new_lp_locker_acc: Account<'info, TokenLock>,

    //added "mut" to credit rent amount if applicable
    #[account(mut,constraint = lock_owner.key()==lp_locker_acc.lock_owner @ UncxLpError::OwnerMismatchError)]
    pub lock_owner: Signer<'info>,

    ///CHECK : address constraint checked
    #[account(mut,constraint = dev_wallet.key()==config_account.config.dev_addr)]
    pub dev_wallet: UncheckedAccount<'info>,

    #[account(mut,seeds=[CONFIG_ACCOUNT_SEED],bump=config_account.bump)]
    pub config_account: Account<'info, ConfigurationAccount>,

    pub system_program: Program<'info, System>,

    //user info acc
    //TODO(aaraN) preaudit : Account type should be optional since the split amount could be equal
    // to the prev locker amount in that scenario we need, keep it as is until a alternative way
    // can be found to get user_info_lp_tracker_acc without depending on this accadd
    #[account(mut,seeds=[USER_INFO_SEED,lock_owner.key().as_ref(),lp_locker_acc.amm_id.as_ref()],bump=user_info_acc.bump)]
    pub user_info_acc: Account<'info, UserInfoAccount>,
    //Security :will be validated that this acc represent the correct lp account tracker
    //without seeds validation by ensuring it contains a specific locker id
    #[account(init_if_needed,payer=payer,seeds=[USER_LP_TRACKER_SEED,lock_owner.key().as_ref(),lp_locker_acc.amm_id.as_ref(),user_info_acc.next_user_lp_acc_index().as_ref()],bump,space=DISCRIMINATOR_BYTES_SIZE+UserLpInfoAccount::INIT_SPACE)]
    pub user_info_lp_tracker_acc: Account<'info, UserLpInfoAccount>,
}
