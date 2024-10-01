use super::*;

use test_create_and_lock_lp::{
    handle_create_and_lock_lp,
    TestCreateLockerFixture,
};

#[tokio::test]

pub async fn test_transfer_lock_ownership() -> Result<(), TransportError> {

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

    // println!("next_locker_id {},old_locker_id {}",next_locker_id,old_locker_id);
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

    let old_user_info_acc_address = CreateAndLockLpInstructionDerivedAccounts::user_info_address(
        fixture.lock_owner,
        fixture.raydium_amm_accounts.amm_id,
    );

    let old_user_info_acc = initial_admin_settings
        .get_account::<uncx_solana_lp_locker::state::UserInfoAccount>(old_user_info_acc_address)
        .await;

    let new_owner = Pubkey::new_unique();

    let new_user_info_acc_address = CreateAndLockLpInstructionDerivedAccounts::user_info_address(
        new_owner,
        fixture.raydium_amm_accounts.amm_id,
    );

    //getting new user locker count
    println!("getting new user locker count pending");

    let new_user_locker_count = initial_admin_settings
        .get_optional_account::<uncx_solana_lp_locker::state::UserInfoAccount>(
            new_user_info_acc_address,
        )
        .await
        .map(|info_acc| info_acc.lp_locker_count)
        .unwrap_or(0);

    println!("getting new user locker count successful");

    let transfer_lock_ix = TransferOwnerShipLockerInstruction::new(
        current_locker_id,
        fixture.locker_wallet.user_wallet,
        new_owner,
        fixture.raydium_amm_accounts.amm_id,
        old_user_info_acc.lp_locker_count,
        initial_admin_settings.payer,
        fixture.raydium_amm_accounts.amm_lp_mint,
        new_user_locker_count,
    );

    send_tx(&initial_admin_settings.context.solana, transfer_lock_ix).await?;

    let locker_account = initial_admin_settings
        .get_account::<uncx_solana_lp_locker::state::TokenLock>(get_locker_pda_acc(
            current_locker_id,
        ))
        .await;

    assert_eq!(locker_account.lock_owner, new_owner);

    Ok(())
}
