use super::*;
use crate::constants::{DISCRIMINATOR_BYTES_SIZE, WHITELIST_ACC_STATIC_SEED};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use mpl_token_metadata::accounts::Metadata as MPLMETADATA;
/// TokenLock is modelled as follows, Seeds:["uncx_locker","locker_id(unique)"] locker_id =
/// &[locker_id.to_le_bytes()]
use raydium_port::{calc_actual_reserves, AmmInfo, Loadable};

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

#[cfg_attr(feature = "cpi-event", event_cpi)]
#[derive(Accounts, Clone)]
//user
#[instruction(lock_owner : Pubkey,amm_info_acc_key: Pubkey,referral_wallet_key :Option<Pubkey>)]

pub struct CreateAndLockLp<'info> {
    ///Payer
    #[account(mut)]
    pub payer: Signer<'info>,
    ///Core Accounts
    // Country Black List Acc PDA

    //uncx lp vault where users lp will be locked into
    #[account(init_if_needed,payer=payer,seeds=[UNCX_LP_VAULT_ACCOUNT,amm_info_acc.key().as_ref()],bump,token::mint = lp_mint_acc,token::authority = uncx_authority_acc)]
    pub uncx_lock_lp_vault_acc: Account<'info, TokenAccount>,

    //pda of the UNCX Locker Protocol Program
    ///CHECK : `its a pda account, checked via its seeds`
    #[account(constraint=uncx_authority_acc.key()==config_account.uncx_authority_pda_address)]
    pub uncx_authority_acc: UncheckedAccount<'info>,

    //Lp marker acc, to help us track all the unique amm's we have locked lp for
    //Tracks Amm id not the mint of the derived lp account
    #[account(init_if_needed,payer=payer,seeds=[LP_MARKER_SEED,amm_info_acc.key().as_ref()],bump,space=DISCRIMINATOR_BYTES_SIZE+GlobalLpMintMarker::INIT_SPACE)]
    pub global_lp_marker_acc: Box<Account<'info, GlobalLpMintMarker>>,
    //The Locker account holding important information such as lock owner,withdrawal date, start
    // and current locked amounts etc
    #[account(init,payer=payer,seeds=[LP_LOCKER_SEED,config_account.config.next_locker_unique_id.to_le_bytes().as_ref()],bump,space=DISCRIMINATOR_BYTES_SIZE+TokenLock::INIT_SPACE)]
    pub lp_locker_acc: Box<Account<'info, TokenLock>>,

    // Config Account storing UNCX Locker Protocol Configuration
    #[account(mut,seeds=[CONFIG_ACCOUNT_SEED],bump=config_account.bump)]
    pub config_account: Box<Account<'info, ConfigurationAccount>>,
    // Lp mint acc specified in the raydiumn amm info account
    #[account(constraint = lp_mint_acc.key()==AmmInfo::load_checked(amm_info_acc.as_ref())?.lp_mint)]
    pub lp_mint_acc: Box<Account<'info, Mint>>,

    ///User Specific Accounts

    //doc: this checks if the lp account authority is the user account or a seperate account
    /*TEST THIS SCENARIO SPECIFICALLY */
    //NOTE :as_ref is important as it converts &Option<Signer> to Option<&Signer> allowing us to
    // return a refrence to a signer instead of cloning in the unwrap_or_else
    //User lp token account which holds the lock lp tokens
    #[account(mut,token::mint=lp_mint_acc,token::authority =user_lp_token_authority_acc.as_ref().unwrap_or(&payer).key())]
    pub user_lp_token_acc: Box<Account<'info, TokenAccount>>,
    ///IX CreateAndlockLp Acc # 9
    // *** #4
    //optional user specified authority incase payer is not  the authority
    pub user_lp_token_authority_acc: Option<Signer<'info>>,

    // *** #7
    //add referral wallet check if a referral wallet account was added
    // #[account(constraint=referral_wallet.as_ref().map(|acc|
    // acc.key)==&referral_wallet_key.as_ref())] referral
    #[account(mut)]
    pub referral_wallet: Option<SystemAccount<'info>>,

    /// *** #8
    /// Used just for balance check i.e referral account balance >
    /// config_account.config.referral_min_balance

    #[account(token::mint=config_account.config.referral_token_address.unwrap(),constraint =referral_token_account.amount>= config_account.config.min_referral_balance @ UncxLpError::InsufficientReferralBalance,token::authority=referral_wallet)]
    pub referral_token_account: Option<Box<Account<'info, TokenAccount>>>,
    ///IX CreateAndlockLp Acc # 15

    ///*** #9
    #[account(mut,token::mint=config_account.config.secondary_token_address.unwrap(), token::authority = user_secondary_token_authority_acc.as_ref().unwrap_or(&payer).key())]
    pub user_secondary_token_account: Option<Box<Account<'info, TokenAccount>>>,
    ///IX CreateAndlockLp Acc # 16

    /// referral secondary token account to credit the referral fees in
    #[account(mut,token::mint=config_account.config.secondary_token_address.unwrap(),token::authority=referral_wallet)]
    pub referral_secondary_token_account: Option<Box<Account<'info, TokenAccount>>>,
    ///IX CreateAndlockLp Acc # 17

    /// *** 11
    /// user secondary token acc authority
    pub user_secondary_token_authority_acc: Option<Signer<'info>>,
    ///IX CreateAndlockLp Acc # 18

    ///*** 13
    /// if the user opts for burning secondary specified token in the config acc, the mint for that
    /// token
    #[account(mut,constraint = secondary_token_mint.key()==config_account.config.secondary_token_address.unwrap() @ UncxLpError::InvalidAccountError)]
    pub secondary_token_mint: Option<Box<Account<'info, Mint>>>,
    //dev lp token account to get the liquidity fees
    #[account(init_if_needed,payer=payer,associated_token::mint = lp_mint_acc,associated_token::authority =dev_wallet )]
    pub dev_lp_token_account: Option<Account<'info, TokenAccount>>,

    //dev wallet to transfer fees to incase feepayment mode is native
    #[account(mut,constraint = dev_wallet.key()==config_account.config.dev_addr)]
    pub dev_wallet: Option<UncheckedAccount<'info>>,

    ///if the locker is a whitelisted entity, this accounts existence is his proof of whitelist
    // user_whitelist_pda_acc marker
    #[account(seeds=[WHITELIST_ACC_STATIC_SEED,whitelist_address.as_ref().unwrap().key().as_ref()],bump=user_whitelist_pda_acc.bump)]
    pub user_whitelist_pda_acc: Option<Account<'info, Whitelisted>>,
    ///IX CreateAndlockLp Acc # 21

    ///whitelisted account which signs the tx.
    // whitelist entity passed as a signer
    pub whitelist_address: Option<Signer<'info>>,

    #[account(init_if_needed,payer=payer,seeds=[USER_INFO_SEED,lock_owner.as_ref(),amm_info_acc_key.as_ref()],bump,space=DISCRIMINATOR_BYTES_SIZE+UserInfoAccount::INIT_SPACE)]
    pub user_info_acc: Box<Account<'info, UserInfoAccount>>,
    #[account(init_if_needed,payer=payer,seeds=[USER_LP_TRACKER_SEED,lock_owner.as_ref(),amm_info_acc_key.as_ref(),user_info_acc.next_user_lp_acc_index().as_ref()],bump,space=DISCRIMINATOR_BYTES_SIZE+UserLpInfoAccount::INIT_SPACE)]
    pub user_info_lp_tracker_acc: Box<Account<'info, UserLpInfoAccount>>,
    /// Utility Programs
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Option<Program<'info, AssociatedToken>>,

    /// Indexer Utility Accounts
    // Token metadata account of coin
    ///CHECK: See ["validate_metadata"]
    pub coin_metadata_account: UncheckedAccount<'info>,
    //
    // // Token metadata account of pc
    ///CHECK: See ["validate_metadata"]
    pub pc_metadata_account: UncheckedAccount<'info>,

    // Raydium accounts to calculate true pc and coin reserves.
    ///List of Accounts if openbook enabled , but we assume to only support pools which do not
    ///share liquidity with openbook, as per the raydium protocol team, apart from 6 pools rest
    ///were changed to the status of not sharing liquidity with openbook
    ///msg1) https://discord.com/channels/813741812598439958/813750197423308820/1265105647746289818
    ///msg2) https://discord.com/channels/813741812598439958/813750197423308820/1265097426105139242
    //1) Amm info acc
    //2) Pc Vault acc
    //3) Coin Vault acc
    //4) TargetOrders acc
    //5) OpenOrders acc
    //6) market event queue acc

    ///List of Accounts if open book disabled
    //1) AmmInfo Acc
    //2) Pc Vault Acc
    //3) Coin Vault Acc
    //4) Target Orders Acc

    //
    //Required Acc in either case ,OpenBook Enabled or disabled
    #[account(
        owner = raydium_amm::ID,
    )]
    ///CHECK: ownership check is done, also its zero-copy deserialized via bytemuck and in other
    /// areas its values are checked if the length wasnt sufficient or wrong the zero-copy
    /// deserialization in other places its used would error out.
    pub amm_info_acc: UncheckedAccount<'info>,
    ///CHECK: See [" validate_raydium_reserve_calc_accounts"]
    pub amm_target_orders_info_acc: UncheckedAccount<'info>,
    ///CHECK: See [" validate_raydium_reserve_calc_accounts"]
    pub pc_vault_token_acc: Box<Account<'info, TokenAccount>>,
    ///CHECK: See [" validate_raydium_reserve_calc_accounts"]
    pub coin_vault_token_acc: Box<Account<'info, TokenAccount>>,
    // //Optional Account incase OpenBook enabled. - We dont support openbook enabled amms
    // pub amm_open_orders_acc: UncheckedAccount<'info>,
    // pub market_event_queue_acc: UncheckedAccount<'info>,
}

impl<'info> CreateAndLockLp<'info> {
    pub(crate) fn is_country_allowed(&self, country_code: u8) -> bool {
        self.config_account.is_country_allowed(country_code)
    }

    pub(crate) fn validate_metadata(
        &self,
    ) -> anchor_lang::prelude::Result<(MPLMETADATA, MPLMETADATA)> {
        let amm_info_acc = AmmInfo::load(self.amm_info_acc.as_ref())?;
        debug!(
            "coin metadata  key {},pc metadta key {}",
            self.coin_metadata_account.key(),
            self.pc_metadata_account.key(),
        );
        let coin_metadata_acc_data = mpl_token_metadata::accounts::Metadata::from_bytes(
            &self
                .coin_metadata_account
                .data
                .try_borrow()
                .map_err(|_| UncxLpError::InvalidTokenMetadata)?[..],
        )?;
        let pc_metadata_acc_data = mpl_token_metadata::accounts::Metadata::from_bytes(
            self.pc_metadata_account
                .data
                .try_borrow()
                .map_err(|_| UncxLpError::InvalidTokenMetadata)?
                .as_ref(),
        )?;
        let true = ((coin_metadata_acc_data.mint == amm_info_acc.coin_vault_mint)
            && (pc_metadata_acc_data.mint == amm_info_acc.pc_vault_mint)
        // Add Token Metadata Account Ownership checks
            && (self.coin_metadata_account.owner == &mpl_token_metadata::ID)
            && (self.pc_metadata_account.owner == &mpl_token_metadata::ID))
        else {
            return err!(UncxLpError::InvalidTokenMetadata);
        };

        Ok((coin_metadata_acc_data, pc_metadata_acc_data))
    }
    //Validates passed in pc_vault token acc, coin_vault_token_acc and associated target orders acc.

    pub(crate) fn validate_raydium_reserve_calc_accounts(&self, amm_info: &AmmInfo) -> Result<()> {
        // Assert amm  : pc|vault| target orders, acc keys instead of matching respective mints.
        require_keys_eq!(
            amm_info.pc_vault,
            self.pc_vault_token_acc.key(),
            UncxLpError::InvalidRaydiumV4Accounts
        );
        require_keys_eq!(
            amm_info.coin_vault,
            self.coin_vault_token_acc.key(),
            UncxLpError::InvalidRaydiumV4Accounts
        );
        require_keys_eq!(
            amm_info.target_orders,
            self.amm_target_orders_info_acc.key(),
            UncxLpError::InvalidRaydiumV4Accounts
        );
        Ok(())
    }
    pub(crate) fn checked_calc_raydium_reserves(
        &self,
        amm_info: &AmmInfo, //return amm lp mint amount,amm true pc amount, amm true coin amount
    ) -> anchor_lang::prelude::Result<(u64, u64, u64)> {
        self.validate_raydium_reserve_calc_accounts(amm_info)?;
        calc_actual_reserves(
            amm_info,
            &self.amm_target_orders_info_acc,
            &self.pc_vault_token_acc,
            &self.coin_vault_token_acc,
        )
    }
}
