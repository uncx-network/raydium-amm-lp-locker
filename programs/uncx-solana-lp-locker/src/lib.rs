pub mod accounts_ix;
pub mod constants;
pub mod error;
pub mod instructions;
mod macros;
pub mod state;

mod utils;

#[cfg(feature = "zero-copy")]
mod zerocopy;

use accounts_ix::*;
use anchor_lang::prelude::*;

pub mod raydium_port;

pub use accounts_ix::CONFIG_ACCOUNT_SEED;

use error::UncxLpError;
use instructions::{
    handle_add_whitelist, handle_create_and_lock_lp, handle_increment_lock_lp,
    handle_initialize_config, handle_relock_lp, handle_remove_whitelist, handle_split_lock,
    handle_transfer_lock_ownership, handle_withdraw_lp, FeePaymentMethod,
};

pub use state::*;

declare_id!("GsSCS3vPWrtJ5Y9aEVVT65fmrex5P5RGHXdZvsdbWgfo");

#[program]

pub mod uncx_solana_lp_locker {

    use self::instructions::{
        handle_add_migrator, handle_migrate_lp, handle_remove_migrator, FeePaymentMethod,
    };

    use super::*;

    //initially only allow blacklist 10 countries.
    pub fn initialize(
        ctx: Context<InitializeConfig>,
        initial_config: Config,
        // initial_black_listed_countries: [u8; 1],
        initial_black_listed_countries: Option<[u8; 10]>,
    ) -> Result<()> {
        handle_initialize_config(ctx, initial_config, initial_black_listed_countries)?;

        Ok(())
    }

    //whitelist address is used in instruction handler,but clippy does not know that hence gives a
    // warning
    #[allow(unused_variables)]

    pub fn add_whitelist(ctx: Context<AddWhitelistAcc>, whitelist_address: Pubkey) -> Result<()> {
        handle_add_whitelist(ctx)?;

        Ok(())
    }

    //whitelist address is used in instruction handler,but clippy does not know that hence gives a
    // warning
    #[allow(unused_variables)]

    pub fn remove_whitelist(
        ctx: Context<RemoveWhitelistAcc>,
        whitelist_address: Pubkey,
    ) -> Result<()> {
        handle_remove_whitelist(ctx)?;

        Ok(())
    }

    //new_migrator_address_pda address is used in instruction handler,but clippy does not know that
    // hence gives a warning
    #[allow(unused_variables)]

    pub fn add_migrator(ctx: Context<AddMigrator>, new_migrator_address_pda: Pubkey) -> Result<()> {
        handle_add_migrator(ctx)?;

        Ok(())
    }

    //migrator_pda_acc address is used in instruction handler,but clippy does not know that hence
    // gives a warning
    #[allow(unused_variables)]

    pub fn remove_migrator(ctx: Context<RemoveMigrator>, migrator_pda_acc: Pubkey) -> Result<()> {
        handle_remove_migrator(ctx)?;

        Ok(())
    }

    //amm_info_acc_key is used in instruction handler
    #[allow(unused_variables, clippy::too_many_arguments)]

    pub fn create_and_lock_lp(
        ctx: Context<CreateAndLockLp>,
        lock_owner: Pubkey,
        amm_info_acc_key: Pubkey,
        referral_wallet_key: Option<Pubkey>,
        lock_amount: u64,
        unlock_date: i64,
        country_code: u8,

        fee_payment_method: FeePaymentMethod,
    ) -> Result<()> {
        handle_create_and_lock_lp(
            ctx,
            lock_owner,
            lock_amount,
            unlock_date,
            country_code,
            referral_wallet_key,
            fee_payment_method,
        )?;

        Ok(())
    }

    pub fn relock_lp(ctx: Context<RelockLp>, lock_id: u64, new_unlock_date: i64) -> Result<()> {
        handle_relock_lp(ctx, lock_id, new_unlock_date)?;

        Ok(())
    }

    pub fn split_relock_lp(
        ctx: Context<SplitLock>,
        old_locker_id: u64,
        new_locker_locked_amount: u64,
    ) -> Result<()> {
        handle_split_lock(ctx, old_locker_id, new_locker_locked_amount)?;

        Ok(())
    }

    pub fn transfer_lock_ownership(
        ctx: Context<TransferLockOwnership>,
        locker_id: u64,
        new_owner: Pubkey,
    ) -> Result<()> {
        handle_transfer_lock_ownership(ctx, locker_id, new_owner)?;

        Ok(())
    }

    pub fn withdraw_lp(
        ctx: Context<WithdrawLp>,
        locker_id: u64,
        withdraw_amount: u64,
    ) -> Result<()> {
        handle_withdraw_lp(ctx, locker_id, withdraw_amount)?;

        Ok(())
    }

    pub fn migrate_lp(
        ctx: Context<MigrateLp>,
        locker_id: u64,
        migrate_amount: u64,
        migration_option: u16,
    ) -> Result<()> {
        handle_migrate_lp(ctx, locker_id, migrate_amount, migration_option)?;

        Ok(())
    }

    pub fn increment_lock_lp(
        ctx: Context<IncrementLockLp>,
        locker_id: u64,
        increase_lp_amount_by: u64,
    ) -> Result<()> {
        handle_increment_lock_lp(ctx, locker_id, increase_lp_amount_by)?;

        Ok(())
    }

    pub fn set_dev(ctx: Context<AdminIx>, new_dev_addr: Pubkey) -> Result<()> {
        instructions::handle_set_dev(ctx, new_dev_addr)?;

        Ok(())
    }

    pub fn set_new_admin(ctx: Context<AdminIx>, new_admin_addr: Pubkey) -> Result<()> {
        instructions::handle_change_owner(ctx, new_admin_addr)?;

        Ok(())
    }

    pub fn set_new_fees_config(ctx: Context<AdminIx>, new_fees_config: FeesConfig) -> Result<()> {
        instructions::handle_set_fees(ctx, new_fees_config)?;

        Ok(())
    }

    pub fn set_referral_token_hold_balance(
        ctx: Context<AdminIx>,
        new_referral_token_hold_balance: u64,
        new_referral_token: Option<Pubkey>,
    ) -> Result<()> {
        instructions::handle_set_referral_token_and_min_balance(
            ctx,
            new_referral_token,
            new_referral_token_hold_balance,
        )?;

        Ok(())
    }

    pub fn set_secondary_token(
        ctx: Context<AdminIx>,
        new_secondary_token: Option<Pubkey>,
    ) -> Result<()> {
        instructions::handle_set_secondary_token(ctx, new_secondary_token)?;

        Ok(())
    }
    pub fn add_country_to_blacklist(ctx: Context<AdminIx>, country_code_to_add: u8) -> Result<()> {
        instructions::handle_add_country_to_blacklist(ctx, country_code_to_add)?;

        Ok(())
    }
    pub fn remove_country_from_blacklist(
        ctx: Context<AdminIx>,
        country_code_to_remove: u8,
    ) -> Result<()> {
        instructions::handle_remove_country_from_blacklist(ctx, country_code_to_remove)?;

        Ok(())
    }
}

pub mod raydium_amm {

    #[cfg(feature = "mainnet")]

    anchor_lang::declare_id!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");

    #[cfg(feature = "devnet")]

    anchor_lang::declare_id!("HWy1jotHpo6UqeQxx49dpYYdQB8wj9Qk9MdxwjLvDHB8");
}
