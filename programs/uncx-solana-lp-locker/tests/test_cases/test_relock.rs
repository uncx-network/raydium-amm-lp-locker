use super::*;

use test_create_and_lock_lp::{
    handle_create_and_lock_lp,
    TestCreateLockerFixture,
};

#[tokio::test]

pub async fn test_relock_lp() -> Result<(), TransportError> {

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

    let config_acc = initial_admin_settings
        .get_account::<ConfigurationAccount>(initial_admin_settings.config_account)
        .await;

    let next_locker_id = config_acc.config.next_locker_unique_id;

    let current_locker_id = next_locker_id - 1;

    println!("getting locker account");

    let locker_account = initial_admin_settings
        .get_account::<uncx_solana_lp_locker::state::TokenLock>(get_locker_pda_acc(
            current_locker_id,
        ))
        .await;

    println!("got locker account");

    assert_eq!(locker_account.lock_owner, fixture.lock_owner);

    let new_unlock_date = locker_account.unlock_date + 86400;

    let relock_lp_ix = RelockInstruction::new(
        current_locker_id,
        fixture.locker_wallet.user_wallet,
        fixture.raydium_amm_accounts.amm_id,
        initial_admin_settings.payer,
        fixture.raydium_amm_accounts.amm_lp_mint,
        config_acc.config.dev_addr,
        new_unlock_date,
    );

    send_tx(&initial_admin_settings.context.solana, relock_lp_ix).await?;

    //new locker acc
    let locker_with_updated_date = initial_admin_settings
        .get_account::<uncx_solana_lp_locker::state::TokenLock>(get_locker_pda_acc(
            current_locker_id,
        ))
        .await;

    assert_eq!(locker_with_updated_date.unlock_date, new_unlock_date);

    Ok(())
}
