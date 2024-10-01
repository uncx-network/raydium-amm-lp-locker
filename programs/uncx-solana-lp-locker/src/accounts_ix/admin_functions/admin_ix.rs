use super::*;

//use inital declared pubkey instead of programdata pattern to safeguard initial configuration
#[derive(Accounts)]

pub struct AdminIx<'info> {
    /// Signer of the Admin Key
    #[account(constraint =admin_sign.key()==config_account.config.admin_key)]
    admin_sign: Signer<'info>,

    ///Configuration Account PDA
    #[account(mut,seeds=[CONFIG_ACCOUNT_SEED],bump=config_account.bump)]
    pub config_account: Account<'info, ConfigurationAccount>,
}
