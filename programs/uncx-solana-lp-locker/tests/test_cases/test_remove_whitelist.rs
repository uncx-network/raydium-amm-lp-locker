use std::{
    borrow::BorrowMut,
    ops::DerefMut,
};

use uncx_solana_lp_locker::instructions::handle_add_whitelist;

use super::{
    test_add_whitelist::handle_test_add_whitelist,
    *,
};

#[tokio::test]

pub async fn test_remove_whitelist() -> Result<(), TransportError> {

    println!("Starting Remove Whitelist Ix");

    let initial_admin_settings =
        TestContext::with_initialize_locker_program(TestInitializeSettings::default()).await?;

    let whitelist_address = Pubkey::new_unique();

    let receiver_balance_before_add_whitelist = initial_admin_settings
        .context
        .solana
        .get_account_native_balance(initial_admin_settings.payer.pubkey())
        .await?;

    handle_test_add_whitelist(&initial_admin_settings, whitelist_address).await?;

    let remove_whitelist = RemoveWhitelistInstruction {
        whitelisted_address_to_remove: whitelist_address,
        payer: initial_admin_settings.payer,
        admin: initial_admin_settings.admin,
    };

    println!("Balance before adding a whitelist acc {receiver_balance_before_add_whitelist}");

    let receiver_balance_before_remove_whitelist = initial_admin_settings
        .context
        .solana
        .get_account_native_balance(initial_admin_settings.payer.pubkey())
        .await?;

    println!("Balance before removing and closing a whitelist acc {receiver_balance_before_remove_whitelist}");

    let Ok(uncx_solana_lp_locker::accounts::RemoveWhitelistAcc { .. }) =
        send_tx(&initial_admin_settings.context.solana, remove_whitelist).await
    else {

        println!("{:?}", &initial_admin_settings.context.solana.program_log());

        return Err(TransportError::Custom(
            "Removing whitelist failed".to_string(),
        ));
    };

    let receiver_balance_after_remove_whitelist = initial_admin_settings
        .context
        .solana
        .get_account_native_balance(initial_admin_settings.payer.pubkey())
        .await?;

    println!("Balance before removing and closing a whitelist acc {receiver_balance_after_remove_whitelist}");

    assert_eq!(
        receiver_balance_after_remove_whitelist, receiver_balance_before_add_whitelist,
        "receiver balance mismatch"
    );

    Ok(())
}
