use super::*;

#[tokio::test]

pub async fn test_add_whitelist() -> Result<(), TransportError> {

    println!("Starting Add Whitelist Ix");

    let initial_admin_settings =
        TestContext::with_initialize_locker_program(TestInitializeSettings::default()).await?;

    let whitelist_address = Pubkey::new_unique();

    handle_test_add_whitelist(&initial_admin_settings, whitelist_address).await?;

    Ok(())
}

pub async fn handle_test_add_whitelist(
    initial_admin_settings: &TestInitialize,
    whitelist_address: Pubkey,
) -> std::result::Result<(), TransportError> {

    let add_whitelist = AddWhiteListInstruction {
        address_to_whitelist: whitelist_address,
        payer: initial_admin_settings.payer,
        admin: initial_admin_settings.admin,
    };

    let offchain_bump = add_whitelist.get_whitelisted_pda_acc().1;

    let pda_acc = add_whitelist.get_whitelisted_pda_acc().0;

    let Ok(uncx_solana_lp_locker::accounts::AddWhitelistAcc { .. }) =
        send_tx(&initial_admin_settings.context.solana, add_whitelist).await
    else {

        println!("{:?}", &initial_admin_settings.context.solana.program_log());

        return Err(TransportError::Custom(
            "adding whitelist failed".to_string(),
        ));
    };

    // println!("WHITELISTED PDA ACC ADDRESS {}", pda_acc);

    let whitelisted_acc = initial_admin_settings
        .context
        .solana
        .get_account::<uncx_solana_lp_locker::state::Whitelisted>(pda_acc)
        .await;

    // println!(
    //     "onchain bump {}, offchain bump {}",
    //     whitelisted_acc.bump, offchain_bump
    // );

    assert_eq!(whitelisted_acc.bump, offchain_bump);

    Ok(())
}
