use super::*;

use anchor_spl::token_interface::{
    TokenAccount,
    TokenInterface,
};

#[cfg_attr(feature = "cpi-event", event_cpi)]
#[derive(Accounts)]
//user
#[instruction(locker_id : u64)]

pub struct RelockLp<'info> {
    #[account(mut,seeds=[LP_LOCKER_SEED,locker_id.to_le_bytes().as_ref()],bump=lp_locker_acc.bump,constraint = lp_locker_acc.lock_global_id==locker_id)]
    pub lp_locker_acc: Account<'info, TokenLock>,

    #[account(constraint = lock_owner.key()==lp_locker_acc.lock_owner @ UncxLpError::OwnerMismatchError)]
    pub lock_owner: Signer<'info>,
    //less cu units, direct comparison.
    ///CHECK : `its a pda account, checked via its seeds`
    #[account(constraint=uncx_authority_acc.key()==config_account.uncx_authority_pda_address)]
    pub uncx_authority_acc: UncheckedAccount<'info>,

    #[account(mut,seeds=[UNCX_LP_VAULT_ACCOUNT,lp_locker_acc.amm_id.as_ref()],bump,token::mint = lp_locker_acc.lp_mint,token::authority = uncx_authority_acc)]
    pub uncx_lock_lp_vault_acc: InterfaceAccount<'info, TokenAccount>,

    #[account(mut,associated_token::mint = lp_locker_acc.lp_mint,associated_token::authority =config_account.config.dev_addr )]
    pub dev_lp_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(seeds=[CONFIG_ACCOUNT_SEED],bump=config_account.bump)]
    pub config_account: Account<'info, ConfigurationAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}
