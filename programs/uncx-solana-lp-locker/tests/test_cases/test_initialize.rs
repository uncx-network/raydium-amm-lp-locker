use super::*;

#[tokio::test]

pub async fn test_locker_initialize() -> Result<(), TransportError> {
    //println!("Starting locker initialization");
    let initial_admin_settings =
        TestContext::with_initialize_locker_program(TestInitializeSettings::default()).await?;
    let config_acc = initial_admin_settings
        .get_account::<ConfigurationAccount>(initial_admin_settings.config_account)
        .await;
    println!("config {:?}", config_acc);

    //println!(" locker initialization successful");

    Ok(())
}
