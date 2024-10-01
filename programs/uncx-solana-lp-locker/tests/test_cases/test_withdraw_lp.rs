use super::*;

use test_create_and_lock_lp::{
    handle_create_and_lock_lp,
    TestCreateLockerFixture,
};

#[tokio::test]

pub async fn test_withdraw_lp() -> Result<(), TransportError> {

    //fee payment method

    println!("Starting Create and Lock Lp Ix");

    //initialize
    let initial_admin_settings =
        TestContext::with_initialize_locker_program(TestInitializeSettings::default()).await?;

    // config account
    let config_acc = initial_admin_settings
        .get_account::<ConfigurationAccount>(initial_admin_settings.config_account)
        .await;

    //add whitelist
    let whitelist_address = TestKeypair::new();

    test_add_whitelist::handle_test_add_whitelist(
        &initial_admin_settings,
        whitelist_address.pubkey(),
    )
    .await?;

    //build raydium accounts

    //lp creation related info
    let next_locker_id = config_acc.config.next_locker_unique_id;

    let next_lp_tracker_acc_index = 0;

    //token accounts (referral,secondary,lp)
    let fixture = TestCreateLockerFixture::create_fixture_non_native(
        &initial_admin_settings,
        1000_000.into(),
    );

    //lock _amount
    let lock_amount = 1_000_001.into();

    // println!("going into handle");

    handle_create_and_lock_lp(
        &initial_admin_settings,
        next_locker_id,
        next_lp_tracker_acc_index,
        fixture,
        config_acc,
        lock_amount,
        None
    )
    .await?;

    let locker_account = initial_admin_settings
        .get_account::<uncx_solana_lp_locker::state::TokenLock>(get_locker_pda_acc(next_locker_id))
        .await;

    let withdraw_amount = locker_account.current_locked_amount - 1000;

    let mut withdraw_lp_ix = WithdrawLpFromLockerInstruction::new_no_user_info_tracker(
        next_locker_id,
        fixture.locker_wallet.user_wallet,
        fixture.raydium_amm_accounts.amm_id,
        0,
        withdraw_amount,
        initial_admin_settings.payer,
        fixture.raydium_amm_accounts.amm_lp_mint,
    );

    println!("Tx 1");

    let Err(_) = send_tx(&initial_admin_settings.context.solana, withdraw_lp_ix).await else {

        panic!("lp unlocked without unlock date being reached")
    };

    let current_time = initial_admin_settings
        .context
        .solana
        .get_clock()
        .await
        .unix_timestamp;

    println!("unlock timestamp is {}", locker_account.unlock_date);

    println!(
        "current time stamp before warping is {}",
        initial_admin_settings
            .context
            .solana
            .get_clock()
            .await
            .unix_timestamp
    );

    initial_admin_settings
        .context
        .solana
        .set_exact_time((locker_account.unlock_date + 1) - current_time)
        .await;

    println!(
        "current time stamp after warping is {}",
        initial_admin_settings
            .context
            .solana
            .get_clock()
            .await
            .unix_timestamp
    );

    //to ensure tx hash differs, as advance slot is broken in solana program test resulting in
    // "calculate_accounts_hash_with_verify mismatch" errors
    withdraw_lp_ix.withdraw_amount += 1;

    send_tx(&initial_admin_settings.context.solana, withdraw_lp_ix).await?;

    Ok(())
}
