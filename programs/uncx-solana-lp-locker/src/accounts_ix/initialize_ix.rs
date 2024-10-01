use super::*;
use crate::constants::{
    DISCRIMINATOR_BYTES_SIZE,
    INITIAL_ADMIN,
};
use anchor_lang::system_program::System;

#[constant]

pub const CONFIG_ACCOUNT_SEED: &[u8] = b"config_account";

#[constant]

pub const BLACKLISTED_COUNTRIES_SEED: &[u8] = b"blacklisted_countries";

//use inital declared pubkey instead of programdata pattern to safeguard initial configuration
#[derive(Accounts)]

// #[instruction(new_admin : Pubkey)]

pub struct InitializeConfig<'info> {
    ///Funding Wallet
    #[account(mut)]
    pub payer: Signer<'info>,
    ///Configuration Account PDA
    #[account(init,payer=payer,seeds=[CONFIG_ACCOUNT_SEED],bump,space=DISCRIMINATOR_BYTES_SIZE+ConfigurationAccount::INIT_SPACE)]
    pub config_account: Account<'info, ConfigurationAccount>,
    /// Country Black List Acc PDA
  
    /// Signer of the Admin Key
    //TODO!(aaraN) use cfg_if crate for better readability

    #[cfg(feature = "testing")]
    #[account(address=INITIAL_ADMIN)]
    ///CHECK : No check is necesary as the unchecked account is only used while testing
    /// its replaced by a signer type in mainnet
    initial_admin: UncheckedAccount<'info>,
    #[cfg(not(feature = "testing"))]
    #[account(address=INITIAL_ADMIN)]
    initial_admin: Signer<'info>,
    ///CHECK : `its a pda account, checked via its seeds`
    #[account(seeds=[UNCX_LOCKER_AUTHORITY_SEED],bump)]
    pub uncx_authority_acc: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}
