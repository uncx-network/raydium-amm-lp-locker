use super::*;

use crate::constants::DISCRIMINATOR_BYTES_SIZE;

#[cfg_attr(feature = "cpi-event", event_cpi)]
#[derive(Accounts)]
//user
#[instruction(locker_id : u64,new_owner:Pubkey)]

pub struct TransferLockOwnership<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut,seeds=[LP_LOCKER_SEED,locker_id.to_le_bytes().as_ref()],bump=lp_locker_acc.bump,constraint = lp_locker_acc.lock_global_id==locker_id,constraint = lp_locker_acc.lock_owner!=new_owner)]
    pub lp_locker_acc: Account<'info, TokenLock>,
    //added "mut" to credit rent amount if applicable
    #[account(mut,constraint =lock_owner.key()==lp_locker_acc.lock_owner @ UncxLpError::OwnerMismatchError)]
    pub lock_owner: Signer<'info>,

    pub system_program: Program<'info, System>,

    //user info acc
    #[account(mut,seeds=[USER_INFO_SEED,lock_owner.key().as_ref(),lp_locker_acc.amm_id.as_ref()],bump=old_user_info_acc.bump)]
    pub old_user_info_acc: Account<'info, UserInfoAccount>,
    //Security :will be validated that this acc represent the correct lp account tracker
    //without seeds validation by ensuring it contains a specific locker id
    #[account(mut)]
    pub old_user_info_lp_tracker_acc: Account<'info, UserLpInfoAccount>,
    //new user acc
    #[account(init_if_needed,payer=payer,seeds=[USER_INFO_SEED,new_owner.as_ref(),lp_locker_acc.amm_id.as_ref()],bump,space=DISCRIMINATOR_BYTES_SIZE+UserInfoAccount::INIT_SPACE)]
    pub new_user_info_acc: Account<'info, UserInfoAccount>,

    #[account(init_if_needed,payer=payer,seeds=[USER_LP_TRACKER_SEED,new_owner.as_ref(),lp_locker_acc.amm_id.as_ref(),new_user_info_acc.next_user_lp_acc_index().as_ref()],bump,space=DISCRIMINATOR_BYTES_SIZE+UserLpInfoAccount::INIT_SPACE)]
    pub new_user_info_lp_tracker_acc: Account<'info, UserLpInfoAccount>,
}
