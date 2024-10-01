use super::*;

#[tokio::test]

pub async fn test_add_migrator() -> Result<(), TransportError> {

    println!("Starting Add Migrator Ix");

    let initial_admin_settings =
        TestContext::with_initialize_locker_program(TestInitializeSettings::default()).await?;

    let migrator_address = Pubkey::new_unique();

    handle_test_add_migrator(&initial_admin_settings, migrator_address).await?;

    Ok(())
}

pub async fn handle_test_add_migrator(
    initial_admin_settings: &TestInitialize,
    migrator_address: Pubkey,
) -> std::result::Result<(), TransportError> {

    let add_migrator = AddMigratorInstruction {
        migrator_address: migrator_address,
        payer: initial_admin_settings.payer,
        admin: initial_admin_settings.admin,
    };

    let offchain_bump = add_migrator.get_migrator_pda_acc().1;

    let pda_acc = add_migrator.get_migrator_pda_acc().0;

    let Ok(uncx_solana_lp_locker::accounts::AddMigrator { .. }) =
        send_tx(&initial_admin_settings.context.solana, add_migrator).await
    else {

        println!("{:?}", &initial_admin_settings.context.solana.program_log());

        return Err(TransportError::Custom("adding migrator failed".to_string()));
    };

    //println!("MIGRATORED PDA ACC ADDRESS {}", pda_acc);
    let migratored_acc = initial_admin_settings
        .context
        .solana
        .get_account::<uncx_solana_lp_locker::state::Migrator>(pda_acc)
        .await;

    assert_eq!(migratored_acc.bump, offchain_bump);

    Ok(())
}
