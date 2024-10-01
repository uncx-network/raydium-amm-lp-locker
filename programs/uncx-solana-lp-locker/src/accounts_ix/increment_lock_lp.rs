use super::*;

use anchor_spl::token_interface::{
    TokenAccount,
    TokenInterface,
};

#[cfg_attr(feature = "cpi-event", event_cpi)]
#[derive(Accounts)]
//user
#[instruction(locker_id : u64,increase_amount : u64)]

pub struct IncrementLockLp<'info> {
    #[account(mut,seeds=[LP_LOCKER_SEED,locker_id.to_le_bytes().as_ref()],bump=lp_locker_acc.bump,constraint = lp_locker_acc.lock_global_id==locker_id,constraint = increase_amount>0 @
    UncxLpError::ZeroAmountError)]
    pub lp_locker_acc: Account<'info, TokenLock>,

    #[account(mut,associated_token::mint = lp_locker_acc.lp_mint,associated_token::authority =config_account.config.dev_addr )]
    pub dev_lp_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut,seeds=[UNCX_LP_VAULT_ACCOUNT,lp_locker_acc.amm_id.as_ref()],bump,token::mint = lp_locker_acc.lp_mint,token::authority = config_account.uncx_authority_pda_address)]
    pub uncx_lock_lp_vault_acc: InterfaceAccount<'info, TokenAccount>,

    #[account(seeds=[CONFIG_ACCOUNT_SEED],bump=config_account.bump)]
    pub config_account: Account<'info, ConfigurationAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    //user related accounts
    #[account(mut,token::mint=lp_locker_acc.lp_mint,token::authority =increment_lock_lp_entity_authority_acc)]
    pub increment_lock_lp_entity_lp_token_acc: InterfaceAccount<'info, TokenAccount>,

    pub increment_lock_lp_entity_authority_acc: Signer<'info>,
}
