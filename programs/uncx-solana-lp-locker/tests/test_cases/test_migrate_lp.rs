use super::*;

use test_create_and_lock_lp::{
    handle_create_and_lock_lp,
    TestCreateLockerFixture,
};

#[tokio::test]

pub async fn test_migrate_lp() -> Result<(), TransportError> {

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
    let migrator_kp = TestKeypair::new();

    test_add_migrator::handle_test_add_migrator(&initial_admin_settings, migrator_kp.pubkey())
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
        None,
    )
    .await?;

    let locker_account = initial_admin_settings
        .get_account::<uncx_solana_lp_locker::state::TokenLock>(get_locker_pda_acc(next_locker_id))
        .await;

    let withdraw_amount = locker_account.current_locked_amount - 1000;

    let migrate_lp_ix = MigrateLpInstruction::new_no_user_info_tracker(
        next_locker_id,
        fixture.locker_wallet.user_wallet,
        fixture.raydium_amm_accounts.amm_id,
        0,
        withdraw_amount,
        initial_admin_settings.payer,
        fixture.raydium_amm_accounts.amm_lp_mint,
        migrator_kp,
    );

    //create migrator lp token acc
    initial_admin_settings.create_token_account(
        migrator_kp.pubkey(),
        get_ata_account(
            &migrator_kp.pubkey(),
            &fixture.raydium_amm_accounts.amm_lp_mint,
        ),
        fixture.raydium_amm_accounts.amm_lp_mint,
        1_000_000,
    );

    send_tx(&initial_admin_settings.context.solana, migrate_lp_ix).await?;

    Ok(())
}
