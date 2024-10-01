use super::*;

use crate::constants::{DISCRIMINATOR_BYTES_SIZE, WHITELIST_ACC_STATIC_SEED};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
/// TokenLock is modelled as follows, Seeds:["uncx_locker","locker_id(unique)"] locker_id =
/// &[locker_id.to_le_bytes()]
use raydium_port::AmmInfo;

#[constant]

pub const LP_LOCKER_SEED: &[u8] = b"uncx_locker";

#[constant]

pub const LP_MARKER_SEED: &[u8] = b"global_lp_tracker";

#[constant]

pub const USER_INFO_SEED: &[u8] = b"user_info";

#[constant]

pub const USER_LP_TRACKER_SEED: &[u8] = b"user_lp_tracker";

#[constant]

pub const UNCX_LOCKER_AUTHORITY_SEED: &[u8] = b"uncx_authority";

#[constant]

pub const UNCX_LP_VAULT_ACCOUNT: &[u8] = b"uncx_lp_vault";

/// UserInfo is modelled as follows, the Seed :[`user's wallet address`,`ammid or the lp mint`] are
/// the core seeds of all user info specific accounts // Account#1 - Seeds[(core seeds),b"user"]
#[derive(Accounts, Clone)]

//locker and program lp token acc related instruction
pub struct CoreLockerAccounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub amm_info_acc: UncheckedAccount<'info>,

    #[account(init_if_needed,payer=payer,seeds=[LP_MARKER_SEED,amm_info_acc.key().as_ref()],bump,space=DISCRIMINATOR_BYTES_SIZE+GlobalLpMintMarker::INIT_SPACE)]
    pub global_lp_marker_acc: Box<Account<'info, GlobalLpMintMarker>>,

    #[account(init,payer=payer,seeds=[LP_LOCKER_SEED,config_account.config.next_locker_unique_id.to_le_bytes().as_ref()],bump,space=DISCRIMINATOR_BYTES_SIZE+TokenLock::INIT_SPACE)]
    pub lp_locker_acc: Box<Account<'info, TokenLock>>,

    #[account(mut,seeds=[CONFIG_ACCOUNT_SEED],bump=config_account.bump)]
    pub config_account: Box<Account<'info, ConfigurationAccount>>,
}
#[derive(Accounts, Clone)]
#[instruction(lock_owner : Pubkey,amm_info_acc_key: Pubkey,referral_wallet_key :Pubkey)]

//user related accounts
pub struct UserAccounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(init_if_needed,payer=payer,seeds=[USER_INFO_SEED,lock_owner.as_ref(),amm_info_acc_key.as_ref()],bump,space=DISCRIMINATOR_BYTES_SIZE+UserInfoAccount::INIT_SPACE)]
    pub user_info_acc: Box<Account<'info, UserInfoAccount>>,
    #[account(init_if_needed,payer=payer,seeds=[USER_LP_TRACKER_SEED,lock_owner.as_ref(),amm_info_acc_key.as_ref(),user_info_acc.next_user_lp_acc_index().as_ref()],bump,space=DISCRIMINATOR_BYTES_SIZE+UserLpInfoAccount::INIT_SPACE)]
    pub user_info_lp_tracker_acc: Box<Account<'info, UserLpInfoAccount>>,

    pub system_program: Program<'info, System>,

    ///IX CreateAndlockLp Acc # 13
    ///SAFETY : balance checked to ensure any acc passed as a referral , fullfils the criteria

    ///user to whitelist
    /// will unwrap only in the scenario, whitelisted account is present.
    // user_whitelist_pda_acc marker
    #[account(seeds=[WHITELIST_ACC_STATIC_SEED,whitelist_address.as_ref().unwrap().key().as_ref()],bump=user_whitelist_pda_acc.bump)]
    pub user_whitelist_pda_acc: Option<Account<'info, Whitelisted>>,
    ///IX CreateAndlockLp Acc # 21

    ///whitelisted account which signs the tx.
    // whitelist_address signer account
    pub whitelist_address: Option<Signer<'info>>,
}
#[derive(Accounts, Clone)]
pub struct LpLockerInitAccounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init_if_needed,payer=payer,seeds=[UNCX_LP_VAULT_ACCOUNT,amm_info_acc.key().as_ref()],bump,token::mint = lp_mint_acc,token::authority = uncx_authority_acc)]
    pub uncx_lock_lp_vault_acc: InterfaceAccount<'info, TokenAccount>,
    ///CHECK : `its a pda account, checked via its seeds`
    #[account(constraint=uncx_authority_acc.key()==config_account.uncx_authority_pda_address)]
    pub uncx_authority_acc: UncheckedAccount<'info>,

    #[account(constraint = lp_mint_acc.key()==AmmInfo::load_checked(amm_info_acc.as_ref())?.lp_mint)]
    pub lp_mint_acc: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        owner = raydium_amm::ID,
    )]
    pub amm_info_acc: UncheckedAccount<'info>,

    ///IX CreateAndlockLp Acc # 25
    // associated token program
    /// *** 12
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    #[account(mut,seeds=[CONFIG_ACCOUNT_SEED],bump=config_account.bump)]
    pub config_account: Box<Account<'info, ConfigurationAccount>>,
    /// USER TOKEN ACC
    #[account(init_if_needed,payer=payer,associated_token::mint = lp_mint_acc,associated_token::authority =dev_wallet )]
    pub dev_lp_token_account: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
    pub associated_token_program: Option<Program<'info, AssociatedToken>>,

    #[account(mut,constraint = dev_wallet.key()==config_account.config.dev_addr)]
    pub dev_wallet: Option<UncheckedAccount<'info>>,
}
#[derive(Accounts, Clone)]

pub struct FeeAccounts<'info> {
    /// *** #8
    /// Used just for balance check i.e referral account balance >
    /// config_account.config.referral_min_balance
    #[account(token::mint=config_account.config.referral_token_address.unwrap(),constraint =referral_token_account.amount>= config_account.config.min_referral_balance,token::authority=referral_wallet)]
    pub referral_token_account: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
    ///IX CreateAndlockLp Acc # 15

    ///*** #9
    #[account(mut,token::mint=config_account.config.secondary_token_address.unwrap(),constraint = &user_secondary_token_account.owner ==&user_secondary_token_authority_acc.as_ref().unwrap_or_else(|| &payer ).key())]
    pub user_secondary_token_account: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
    ///IX CreateAndlockLp Acc # 16

    /// *** 10
    #[account(mut,token::mint=config_account.config.secondary_token_address.unwrap())]
    pub referral_secondary_token_account: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
    ///IX CreateAndlockLp Acc # 17

    /// *** 11
    pub user_secondary_token_authority_acc: Option<Signer<'info>>,
    ///IX CreateAndlockLp Acc # 18

    ///*** 13
    #[account(mut,constraint = secondary_token_mint.key()==config_account.config.secondary_token_address.unwrap() @ UncxLpError::InvalidAccountError)]
    pub secondary_token_mint: Option<Box<InterfaceAccount<'info, Mint>>>,

    #[account(mut,seeds=[CONFIG_ACCOUNT_SEED],bump=config_account.bump)]
    pub config_account: Box<Account<'info, ConfigurationAccount>>,
    ////Tx Payer
    ///can be authority of user secondary token account and lp token account authority
    /// also pays fee if its native
    /// IX CreateAndlockLp Acc # 1

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        owner = raydium_amm::ID,
    )]
    pub amm_info_acc: UncheckedAccount<'info>,
    #[account(constraint = lp_mint_acc.key()==AmmInfo::load_checked(amm_info_acc.as_ref())?.lp_mint)]
    pub lp_mint_acc: Box<InterfaceAccount<'info, Mint>>,
    //*** #3
    //doc: this checks if the lp account authority is the user account or a seperate account
    /*TEST THIS SCENARIO SPECIFICALLY */
    //NOTE :as_ref is important as it converts &Option<Signer> to Option<&Signer> allowing us to
    // return a refrence to a signer instead of cloning in the unwrap_or_else
    #[account(mut,token::mint=lp_mint_acc,constraint = &user_lp_token_acc.owner ==&user_lp_token_authority_acc.as_ref().unwrap_or_else(|| &payer ).key())]
    pub user_lp_token_acc: Box<InterfaceAccount<'info, TokenAccount>>,
    ///IX CreateAndlockLp Acc # 9
    // *** #4
    //optional incase payer is the authority
    pub user_lp_token_authority_acc: Option<Signer<'info>>,
    #[account(mut)]
    // *** #7
    //add referral wallet check
    // #[account(address=referral_wallet_key)]
    pub referral_wallet: Option<UncheckedAccount<'info>>,
}

#[derive(Accounts, Clone)]
//user

pub struct CreateAndLockLp<'info> {
    pub core_locker_acc: CoreLockerAccounts<'info>,
    pub user_specific_acc: UserAccounts<'info>,
    pub fee_accounts: FeeAccounts<'info>,
    pub locker_init_accounts: LpLockerInitAccounts<'info>,
    /// Country Black List Acc PDA
    #[account(seeds=[BLACKLISTED_COUNTRIES_SEED],bump=blacklisted_countries.bump)]
    pub blacklisted_countries: Box<Account<'info, BlackListedCountries>>,
}

impl<'info> CreateAndLockLp<'info> {
    pub(crate) fn is_country_allowed(&self, country_code: u8) -> bool {
        self.blacklisted_countries.is_country_allowed(country_code)
    }
}

// #[derive(Accounts)]
// pub struct TestInit<'info> {
//     #[account(mut)]
//     payer: Signer<'info>,
//     #[account(init,payer=payer,seeds=[b"hellop"],bump,space=5)]
//     pub test_account: Account<'info, ConfigurationAccount>,
//     pub system_program: Program<'info, System>,
// }
