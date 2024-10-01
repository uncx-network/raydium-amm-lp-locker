use super::*;

use test_create_and_lock_lp::{
    handle_create_and_lock_lp,
    TestCreateLockerFixture,
};

#[tokio::test]

pub async fn test_split_lock() -> Result<(), TransportError> {

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

    let new_lock_amount = locker_account.current_locked_amount - 1000;

    assert_eq!(locker_account.lock_owner, fixture.lock_owner);

    let old_user_info_acc_address = CreateAndLockLpInstructionDerivedAccounts::user_info_address(
        fixture.lock_owner,
        fixture.raydium_amm_accounts.amm_id,
    );

    let old_user_info_acc = initial_admin_settings
        .get_account::<uncx_solana_lp_locker::state::UserInfoAccount>(old_user_info_acc_address)
        .await;

    let mut split_lock_ix = SplitLockInstruction::new(
        current_locker_id,
        fixture.locker_wallet.user_wallet,
        new_lock_amount,
        fixture.raydium_amm_accounts.amm_id,
        old_user_info_acc.lp_locker_count,
        initial_admin_settings.payer,
        fixture.raydium_amm_accounts.amm_lp_mint,
        config_acc.config.dev_addr,
    );

    //to ensure tx hash differs, as advance slot is broken in solana program test resulting in
    // "calculate_accounts_hash_with_verify mismatch" errors split_lock_ix.new_lock_amount += 1;
    send_tx(&initial_admin_settings.context.solana, split_lock_ix).await?;

    //new locker acc
    let _ = initial_admin_settings
        .get_account::<uncx_solana_lp_locker::state::TokenLock>(get_locker_pda_acc(next_locker_id))
        .await;

    Ok(())
}
