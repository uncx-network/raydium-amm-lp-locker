use super::*;
use crate::constants::MIGRATOR_SEED;

//use inital declared pubkey instead of programdata pattern to safeguard initial configuration

#[derive(Accounts)]
#[instruction(migrator_pda_acc : Pubkey)]

pub struct RemoveMigrator<'info> {
    ///Configuration Account PDA
    #[account(seeds=[CONFIG_ACCOUNT_SEED],bump=config_account.bump)]
    pub config_account: Account<'info, ConfigurationAccount>,

    /// Signer of the Admin Key
    #[account(constraint =admin_sign.key()==config_account.config.admin_key)]
    admin_sign: Signer<'info>,

    ///user to whitelist
    #[account(mut,seeds=[MIGRATOR_SEED,migrator_pda_acc.as_ref()],bump=migrator_marker_acc.bump,close=receiver)]
    migrator_marker_acc: Account<'info, Migrator>,
    /// receiver account to to send lamports to
    #[account(mut)]
    receiver: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}
