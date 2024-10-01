use super::*;
use crate::constants::{
    DISCRIMINATOR_BYTES_SIZE,
    WHITELIST_ACC_STATIC_SEED,
};

//use inital declared pubkey instead of programdata pattern to safeguard initial configuration
#[derive(Accounts)]
#[instruction(whitelist_address : Pubkey)]

pub struct AddWhitelistAcc<'info> {
    ///Funding Wallet
    #[account(mut)]
    pub payer: Signer<'info>,
    /// Signer of the Admin Key
    #[account(constraint =admin_sign.key()==config_account.config.admin_key)]
    admin_sign: Signer<'info>,

    ///Configuration Account PDA
    #[account(seeds=[CONFIG_ACCOUNT_SEED],bump=config_account.bump)]
    pub config_account: Account<'info, ConfigurationAccount>,

    ///user to whitelist
    #[account(init,payer=payer,seeds=[WHITELIST_ACC_STATIC_SEED,whitelist_address.as_ref()],bump,space=DISCRIMINATOR_BYTES_SIZE+Whitelisted::INIT_SPACE)]
    pub user_whitelist_pda_acc: Account<'info, Whitelisted>,
    /// Native System Program
    pub system_program: Program<'info, System>,
}
