use std::ops::{Deref, DerefMut};

use self::accounts_ix::CreateAndLockLp;
use crate::constants::MAX_BASIS_POINTS;
use lock_lp_ix_helper::{
    calc_fee, calc_fee_ceil, calc_total_after_fee_cuts, calc_total_after_fee_cuts_ceil,
    PreciseSettlement, Settlement,
};
use raydium_port::{AmmInfo, Loadable};
pub const INVALID_TIME_BOUND: i64 = 10000000000;

use super::*;

#[cfg_attr(feature = "testing", derive(Copy))]
#[derive(AnchorDeserialize, AnchorSerialize, Clone, Debug)]

pub enum FeePaymentMethod {
    Native,
    NonNative,
    Whitelisted,
}

pub fn handle_create_and_lock_lp(
    ctx: Context<CreateAndLockLp>,
    lock_owner: Pubkey,
    lock_amount: u64,
    unlock_date: i64,
    country_code: u8,
    referral: Option<Pubkey>,
    fee_payment_method: FeePaymentMethod,
) -> Result<()> {
    //Validate pc, coin mints match their token metadata equivalents

    require!(
        unlock_date < INVALID_TIME_BOUND,
        UncxLpError::InvalidTimeBoundError
    );

    //Fix : Locks Can be Created to Unlock in The Past
    require_gt!(
        unlock_date,
        utils::clock_now().0,
        UncxLpError::InvalidUnlockDateError
    );

    //Preaudit Check : Should be gt! -done : status : done
    require_gt!(lock_amount, 0, UncxLpError::ZeroAmountError);

    require!(
        ctx.accounts.is_country_allowed(country_code),
        UncxLpError::BlackListedCountryError
    );

    //if referral is not none, ensure the account is passed where Account IX, checks the
    // constraints related to balance
    if referral.is_some() {
        let Some(_) = ctx.accounts.referral_token_account else {
            return err!(UncxLpError::MissingReferralAccount);
        };
    }
    //ensure a whitelisted acc was passed if the fee payment method was input as whitelisted
    if let FeePaymentMethod::Whitelisted = fee_payment_method {
        require!(
            &ctx.accounts.user_whitelist_pda_acc.is_some(),
            UncxLpError::MissingWhitelistAccount
        );
    };

    let fee_config = &ctx.accounts.config_account.deref().config.fee_config;

    debug!("liq fee is {}", fee_config.liquidity_fee_bps);

    debug!(
        "secondary token  burn amount is {}",
        fee_config.secondary_token_fee
    );

    debug!(
        "referral share percent  is {}",
        fee_config.referral_share_bps
    );

    debug!(
        "discount due to referral is  is {}",
        fee_config.referral_discount_bps
    );
    let mut precise_settlement = PreciseSettlement::default();

    //MAIN FEE LOGIC BODY -START
    match (referral, &fee_payment_method) {
        //no referral ,native,not whitelisted
        (None, FeePaymentMethod::Native) => {
            precise_settlement.dev_fee = fee_config.native_fee.into();

            // precise_settlement.lp_fee_amount = I80F48::from(lock_amount)
            //     .checked_mul(fee_config.liquidity_fee.into())
            //     .ok_or_else(math_error!())?
            //     .checked_div(1000.into())
            //     .ok_or_else(math_error!())?
            //     .checked_ceil()
            //     .ok_or_else(math_error!())?;

            //refactored above to this
            precise_settlement.lp_fee_amount =
                calc_fee_ceil(lock_amount, fee_config.liquidity_fee_bps)?;
        }

        //some referral ,native,not whitelisted
        (Some(_), FeePaymentMethod::Native) => {
            debug!("native with referral");
            //dev fee calc
            precise_settlement.dev_fee = {
                // let dev_fee_after_referral_cut = I80F48::from(fee_config.native_fee)
                //     .checked_mul(I80F48::from(1000) - (I80F48::from(fee_config.referral_discount)))
                //     .ok_or_else(math_error!())?
                //     .checked_div(1000.into())
                //     .ok_or_else(math_error!())?;
                let dev_fee_after_referral_cut = calc_fee(
                    fee_config.native_fee,
                    MAX_BASIS_POINTS - fee_config.referral_discount_bps,
                )?;

                debug!(
                    "intermediate dev fee after referral cut {}",
                    dev_fee_after_referral_cut
                );
                //referral share fee calc
                // precise_settlement.referral_share_fee = dev_fee_after_referral_cut
                //     .checked_mul(I80F48::from(fee_config.referral_share_percent))
                //     .ok_or_else(math_error!())?
                //     .checked_div(1000.into())
                //     .ok_or_else(math_error!())?;
                //referral share fee calc
                precise_settlement.referral_share_fee =
                    calc_fee(dev_fee_after_referral_cut, fee_config.referral_share_bps)?;
                //dev fee post referral share cut
                // dev_fee_after_referral_cut
                //     .checked_sub(precise_settlement.referral_share_fee)
                //     .ok_or_else(math_error!())?
                //     .checked_ceil()
                //     .ok_or_else(math_error!())?
                calc_total_after_fee_cuts_ceil(
                    dev_fee_after_referral_cut,
                    precise_settlement.referral_share_fee,
                )?
            };

            debug!(
                "intermediate referal share  fee {}",
                precise_settlement.referral_share_fee
            );

            // precise_settlement.lp_fee_amount = I80F48::from(lock_amount)
            //     .checked_mul(I80F48::from(fee_config.liquidity_fee))
            //     .ok_or_else(math_error!())?
            //     .checked_div(1000.into())
            //     .ok_or_else(math_error!())?
            //     .checked_ceil()
            //     .ok_or_else(math_error!())?;
            precise_settlement.lp_fee_amount =
                calc_fee_ceil(lock_amount, fee_config.liquidity_fee_bps)?;
        }
        //no referral, nonnative,not whitelisted
        (None, FeePaymentMethod::NonNative) => {
            debug!("nonnative with no referral");

            // precise_settlement.lp_fee_amount = I80F48::from(lock_amount)
            //     .checked_div(1000.into())
            //     .ok_or_else(math_error!())?
            //     .checked_mul(I80F48::from(fee_config.liquidity_fee))
            //     .ok_or_else(math_error!())?
            //     .checked_div(1000.into())
            //     .ok_or_else(math_error!())?
            //     .checked_mul(I80F48::from(1000) - I80F48::from(fee_config.secondary_token_discount))
            //     .ok_or_else(math_error!())?
            //     .checked_ceil()
            //     .ok_or_else(math_error!())?;
            precise_settlement.lp_fee_amount = {
                let lp_fee = calc_fee(lock_amount, fee_config.liquidity_fee_bps)?;

                calc_fee_ceil(
                    lp_fee,
                    MAX_BASIS_POINTS - fee_config.secondary_token_discount_bps,
                )?
            };

            precise_settlement.secondary_token_burn_amount = fee_config.secondary_token_fee.into();
        }
        //referral non-native,not-whitelisted
        (Some(_), FeePaymentMethod::NonNative) => {
            // precise_settlement.lp_fee_amount = I80F48::from(lock_amount)
            //     .checked_mul(I80F48::from(fee_config.liquidity_fee))
            //     .ok_or_else(math_error!())?
            //     .checked_div(1000.into())
            //     .ok_or_else(math_error!())?
            //     .checked_mul(I80F48::from(1000) - I80F48::from(fee_config.secondary_token_discount))
            //     .ok_or_else(math_error!())?
            //     .checked_div(1000.into())
            //     .ok_or_else(math_error!())?
            //     .checked_ceil()
            //     .ok_or_else(math_error!())?;
            precise_settlement.lp_fee_amount = {
                let lp_fee = calc_fee(lock_amount, fee_config.liquidity_fee_bps)?;

                calc_fee_ceil(
                    lp_fee,
                    MAX_BASIS_POINTS - fee_config.secondary_token_discount_bps,
                )?
            };
            //referral hence there is a discount on the secondary token burn amount

            // let precise_burn_amount_with_referral = I80F48::from(fee_config.secondary_token_fee)
            //     .checked_mul(I80F48::from(1000) - I80F48::from(fee_config.referral_discount))
            //     .ok_or_else(math_error!())?
            //     .checked_div(1000.into())
            //     .ok_or_else(math_error!())?
            //     .checked_ceil()
            //     .ok_or_else(math_error!())?;
            let precise_burn_amount_with_referral = calc_fee(
                fee_config.secondary_token_fee,
                MAX_BASIS_POINTS - fee_config.referral_discount_bps,
            )?;

            // precise_settlement.referral_share_fee = precise_burn_amount_with_referral
            //     .checked_mul((fee_config.referral_share_percent).into())
            //     .ok_or_else(math_error!())?
            //     .checked_div(1000.into())
            //     .ok_or_else(math_error!())?
            //     .checked_ceil() // dont ceil it hence we use calc_fee
            //     .ok_or_else(math_error!())?;
            precise_settlement.referral_share_fee = calc_fee(
                precise_burn_amount_with_referral,
                fee_config.referral_share_bps,
            )?;

            // precise_settlement.secondary_token_burn_amount = precise_burn_amount_with_referral
            //     .checked_sub(precise_settlement.referral_share_fee)
            //     .ok_or_else(math_error!())?
            //     .checked_ceil()
            //     .ok_or_else(math_error!())?;
            precise_settlement.secondary_token_burn_amount = calc_total_after_fee_cuts_ceil(
                precise_burn_amount_with_referral,
                precise_settlement.referral_share_fee,
            )?;
        }
        //if its whitelisted
        //check is done at the start of the handle function to ensure a whitelisted acc was
        // input, in the scenario fee payment is whitelisted.
        (_, FeePaymentMethod::Whitelisted) => {
            // precise_settlement.lp_fee_amount = I80F48::from(lock_amount)
            //     .checked_mul(fee_config.liquidity_fee.into())
            //     .ok_or_else(math_error!())?
            //     .checked_div(1000.into())
            //     .ok_or_else(math_error!())?
            //     .checked_ceil()
            //     .ok_or_else(math_error!())?;
            precise_settlement.lp_fee_amount =
                calc_fee_ceil(lock_amount, fee_config.liquidity_fee_bps)?;
        }
    }
    //lock amount after deducing lp fee charged by the protocol
    // precise_settlement.lock_amount = I80F48::from(lock_amount)
    //     - precise_settlement
    //         .lp_fee_amount
    //         .checked_to_num::<u64>()
    //         .ok_or_else(math_error!())?;
    //optimization : there is no need to use ceil version of fee cuts as lp fee amount is ceiled in all possible scenarios hence another checked_ceil is inefficient.
    precise_settlement.lock_amount =
        calc_total_after_fee_cuts(lock_amount.into(), precise_settlement.lp_fee_amount)?;

    //MAIN FEE LOGIC BODY -END

    //All calculations done and settled.
    // Start State Transitions
    //transfer dev fee native or non native + transfer referral fee native or non native +
    // //burn secondary token amount
    let settlement = Settlement::try_from(precise_settlement)?;
    debug!(
        "fee settlement is is {}, {},{},{},{}",
        settlement.dev_fee,
        settlement.lp_fee_amount,
        settlement.referral_share_fee,
        settlement.secondary_token_burn_amount,
        settlement.lock_amount
    );

    //this checks if  dev fee,referral fee, secondary token burn was to be done all fees excluding lp fee is handled here
    match &fee_payment_method {
        FeePaymentMethod::Whitelisted => {
            //no fee except liq fee is charged for whitelisted users.
        }
        FeePaymentMethod::Native => {
            if settlement.dev_fee > 0 {
                let Some(dev_wallet) = &ctx.accounts.dev_wallet else {
                    return err!(UncxLpError::MissingRequiredAccount);
                };

                debug!(
                    "transfering dev fee, from {} to {}",
                    &ctx.accounts.payer.key(),
                    &dev_wallet.key
                );

                //transfer native fee to dev wallet
                utils::system_program_transfer(
                    settlement
                        .dev_fee
                        .try_into()
                        .map_err(|_| UncxLpError::ConversionError)?,
                    &ctx.accounts.system_program,
                    &ctx.accounts.payer,
                    &dev_wallet,
                )?;
            }

            debug!("succeeded in transferring dev fee in native case");

            if settlement.referral_share_fee > 0 {
                let Some(referral_wallet) = &ctx.accounts.referral_wallet else {
                    return err!(UncxLpError::MissingRequiredAccount);
                };

                require_keys_eq!(
                    referral.ok_or_else(math_error!())?,
                    *referral_wallet.key,
                    UncxLpError::MissingRequiredAccount
                );

                //transfer native fee to dev wallet
                utils::system_program_transfer(
                    settlement
                        .referral_share_fee
                        .try_into()
                        .map_err(|_| UncxLpError::ConversionError)?,
                    &ctx.accounts.system_program,
                    &ctx.accounts.payer,
                    &referral_wallet,
                )?;
            }
        }
        FeePaymentMethod::NonNative => {
            //if non-native fee, secondary token account and mint is mandatory
            let (Some(user_secondary_token_account), Some(secondary_token_mint)) = (
                &ctx.accounts.user_secondary_token_account,
                &ctx.accounts.secondary_token_mint,
            ) else {
                return err!(UncxLpError::MissingRequiredAccount);
            };

            //no dev fees burn token is burnt
            //if referral then there is a discount on the burnt token
            //referral account is conditional
            if settlement.referral_share_fee > 0 {
                let Some(referral_secondary_token_account) =
                    &ctx.accounts.referral_secondary_token_account
                else {
                    return err!(UncxLpError::MissingRequiredAccount);
                };

                //transfer native fee to dev wallet
                utils::token_transfer(
                    settlement
                        .referral_share_fee
                        .try_into()
                        .map_err(|_| UncxLpError::ConversionError)?,
                    &ctx.accounts.token_program,
                    user_secondary_token_account.deref(),
                    referral_secondary_token_account,
                    &ctx.accounts
                        .user_secondary_token_authority_acc
                        .as_ref()
                        .unwrap_or(&ctx.accounts.payer),
                )?;
            }

            if settlement.secondary_token_burn_amount > 0 {
                debug!("burn amount is {}", settlement.secondary_token_burn_amount);

                utils::burn_token(
                    settlement
                        .secondary_token_burn_amount
                        .try_into()
                        .map_err(|_| UncxLpError::ConversionError)?,
                    ctx.accounts.token_program.as_ref(),
                    user_secondary_token_account.deref(),
                    secondary_token_mint.deref(),
                    ctx.accounts
                        .user_secondary_token_authority_acc
                        .as_ref()
                        .unwrap_or(&ctx.accounts.payer),
                )?;
            }
        }
    }

    //transfer liq fee to dev wallet

    if settlement.lp_fee_amount > 0 {
        let Some(dev_lp_token_acc) = &ctx.accounts.dev_lp_token_account else {
            return err!(UncxLpError::MissingRequiredAccount);
        };

        utils::token_transfer(
            settlement
                .lp_fee_amount
                .try_into()
                .map_err(|_| UncxLpError::ConversionError)?,
            &ctx.accounts.token_program,
            ctx.accounts.user_lp_token_acc.deref(),
            dev_lp_token_acc,
            &ctx.accounts
                .user_lp_token_authority_acc
                .as_ref()
                .unwrap_or(&ctx.accounts.payer),
        )?;
    }

    //transfer locked amount post liqfee to token vault account
    utils::token_transfer(
        settlement
            .lock_amount
            .try_into()
            .map_err(|_| UncxLpError::ConversionError)?,
        &ctx.accounts.token_program,
        ctx.accounts.user_lp_token_acc.deref(),
        &ctx.accounts.uncx_lock_lp_vault_acc,
        &ctx.accounts
            .user_lp_token_authority_acc
            .as_ref()
            .unwrap_or(&ctx.accounts.payer),
    )?;

    //transfer lock amount
    //create token locker
    let locker = TokenLock {
        amm_id: ctx.accounts.amm_info_acc.key(),
        lp_mint: ctx.accounts.lp_mint_acc.key(),
        lock_global_id: ctx.accounts.config_account.config.next_locker_unique_id,
        //current chain timestamp
        lock_date: utils::clock_now().0,
        initial_lock_amount: settlement
            .lock_amount
            .try_into()
            .map_err(|_| UncxLpError::ConversionError)?,
        current_locked_amount: settlement
            .lock_amount
            .try_into()
            .map_err(|_| UncxLpError::ConversionError)?,
        country_code,
        unlock_date,
        lock_owner,
        //Add bump to tocker locker acc
        bump: ctx.bumps.lp_locker_acc,
    };

    // Security : load_checked is used in atleast one account hence subsequent accounts can use
    // load instead
    let amm_info_acc = AmmInfo::load(&ctx.accounts.amm_info_acc)?;
    let (amm_liquidity, true_pc_vault_reserve, true_coin_vault_reserve) =
        ctx.accounts.checked_calc_raydium_reserves(&amm_info_acc)?;
    debug!("True Raydium Reserves,
    amount in pc vault {}, true pc vault reserve {},amount in coin vault {}, true coin reserve {}, total liquidity {}
",ctx.accounts.pc_vault_token_acc.amount,true_pc_vault_reserve,ctx.accounts.coin_vault_token_acc.amount,true_coin_vault_reserve,amm_liquidity);
    let (coin_metadata_account, pc_metadata_account) = ctx.accounts.validate_metadata()?;
    debug!(
        "coin name and symbok {} {}, pc name and symbol {} {}",
        coin_metadata_account.name,
        coin_metadata_account.symbol,
        pc_metadata_account.name,
        pc_metadata_account.symbol
    );

    let on_new_lock_event = OnNewLock {
        lock_id: locker.lock_global_id,
        amm_id: ctx.accounts.amm_info_acc.key(),
        owner: locker.lock_owner,
        amount: settlement
            .lock_amount
            .try_into()
            .map_err(|_| UncxLpError::ConversionError)?,
        lock_date: utils::clock_now().0,
        unlock_date,
        country_code,
        coin_token_name: coin_metadata_account.name.as_str().into(),
        coin_token_symbol: coin_metadata_account.symbol.as_str().into(),
        coin_token_decimals: amm_info_acc.coin_decimals as u8,
        pc_token_name: pc_metadata_account.name.as_str().into(),
        pc_token_symbol: pc_metadata_account.symbol.as_str().into(),
        pc_token_decimals: amm_info_acc.pc_decimals as u8,
        amm_real_liquidity: amm_liquidity,
        amm_real_pc_reserve: true_pc_vault_reserve,
        amm_real_coin_reserve: true_coin_vault_reserve,
        pc_mint: amm_info_acc.pc_vault_mint,
        coin_mint: amm_info_acc.coin_vault_mint,
    };

    //Add bump to user info acc
    let user_specific_acc_struct = UserInfoAccount {
        bump: ctx.bumps.user_info_acc,
        lp_mint: ctx.accounts.amm_info_acc.key(),
        //new memory is zero initialized
        lp_locker_count: ctx.accounts.user_info_acc.lp_locker_count + 1,
        user: lock_owner,
    };

    ctx.accounts
        .user_info_acc
        .set_inner(user_specific_acc_struct);

    let lp_tracker_acc = ctx.accounts.user_info_lp_tracker_acc.deref_mut();

    //Add bump to  user lp tracker acc
    lp_tracker_acc.bump = ctx.bumps.user_info_lp_tracker_acc;

    lp_tracker_acc
        .associated_locker_ids
        .push(locker.lock_global_id);

    ctx.accounts.lp_locker_acc.set_inner(locker);

    let config_acc = ctx.accounts.config_account.deref_mut();

    config_acc.config.next_locker_unique_id += 1;

    //Add bump to global lp marker acc
    ctx.accounts
        .global_lp_marker_acc
        .set_inner(GlobalLpMintMarker {
            bump: ctx.bumps.global_lp_marker_acc,
        });

    //emit event
    #[cfg(not(feature = "cpi-event"))]
    emit_stack(on_new_lock_event);

    #[cfg(feature = "cpi-event")]
    {
        emit_cpi!(on_new_lock_event)
    };

    Ok(())
}
