use super::*;
use crate::constants::WHITELIST_ACC_STATIC_SEED;

//use inital declared pubkey instead of programdata pattern to safeguard initial configuration
#[derive(Accounts)]
#[instruction(whitelist_address : Pubkey)]

pub struct RemoveWhitelistAcc<'info> {
    ///Configuration Account PDA
    #[account(seeds=[CONFIG_ACCOUNT_SEED],bump=config_account.bump)]
    pub config_account: Account<'info, ConfigurationAccount>,

    /// Signer of the Admin Key
    #[account(constraint =admin_sign.key()==config_account.config.admin_key)]
    admin_sign: Signer<'info>,

    ///user to whitelist
    #[account(mut,seeds=[WHITELIST_ACC_STATIC_SEED,whitelist_address.as_ref()],bump=user_whitelist_pda_acc.bump,close=receiver)]
    user_whitelist_pda_acc: Account<'info, Whitelisted>,
    /// receiver account to to send lamports to
    #[account(mut)]
    receiver: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}
