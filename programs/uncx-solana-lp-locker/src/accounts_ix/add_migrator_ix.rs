use super::*;
use crate::constants::{
    DISCRIMINATOR_BYTES_SIZE,
    MIGRATOR_SEED,
};

//use inital declared pubkey instead of programdata pattern to safeguard initial configuration
#[derive(Accounts)]
#[instruction(new_migrator_address_pda : Pubkey)]

pub struct AddMigrator<'info> {
    ///Funding Wallet
    #[account(mut)]
    pub payer: Signer<'info>,
    /// Signer of the Admin Key
    #[account(constraint =admin_sign.key()==config_account.config.admin_key)]
    admin_sign: Signer<'info>,

    ///Configuration Account PDA
    #[account(seeds=[CONFIG_ACCOUNT_SEED],bump=config_account.bump)]
    pub config_account: Account<'info, ConfigurationAccount>,

    ///new_migrator_address_pda will be a signer
    #[account(init,payer=payer,seeds=[MIGRATOR_SEED,new_migrator_address_pda.as_ref()],bump,space=DISCRIMINATOR_BYTES_SIZE+Migrator::INIT_SPACE)]
    pub migrator_marker_acc: Account<'info, Migrator>,
    /// Native System Program
    pub system_program: Program<'info, System>,
}
