use super::*;

#[tokio::test]

pub async fn test_admin_ix() -> Result<(), TransportError> {
    println!("Starting Admin Ix Tests");

    let initial_admin_settings =
        TestContext::with_initialize_locker_program(TestInitializeSettings::default()).await?;

    let new_dev_addr = Pubkey::new_unique();

    let admin_ix_set_dev_address = AdminIxSetDevAddr {
        admin: initial_admin_settings.admin,
        new_dev_addr,
    };

    let mut new_fee_config = FeesConfig::default();

    new_fee_config.referral_discount_bps = 500;

    let admin_ix_set_fees = AdminIxSetFeeConfig {
        admin: initial_admin_settings.admin,
        new_config: new_fee_config,
    };

    let admin_ix_set_referral_balance = AdminIxSetReferralAndBalance {
        admin: initial_admin_settings.admin,
        new_referral_balance: 10000,
        new_referral_token_address: None,
    };

    let admin_ix_set_secondary_token = AdminIxSetSecondaryToken {
        new_secondary_token_address: None,
        admin: initial_admin_settings.admin,
    };

    let admin_ix_set_new_admin = AdminIxChangeAdmin {
        admin: initial_admin_settings.admin,
        new_admin_key: Pubkey::new_unique(),
    };
    let admin_ix_add_country_code_to_blacklist: AdminIxAddCountryToBlacklist =
        AdminIxAddCountryToBlacklist {
            admin: initial_admin_settings.admin,
            country_to_add_to_black_list: 100,
        };

    let admin_ix_remove_country_code_to_blacklist: AdminIxRemoveFromBlackList =
        AdminIxRemoveFromBlackList {
            admin: initial_admin_settings.admin,
            country_to_remove_from_blacklist: 100,
        };

    let config_acc = initial_admin_settings
        .get_account::<ConfigurationAccount>(initial_admin_settings.config_account)
        .await;

    println!("config_acc is {:?}", config_acc);

    assert_ne!(config_acc.config.dev_addr, new_dev_addr);

    handle_admin_ix(admin_ix_set_dev_address, &initial_admin_settings).await?;

    let config_acc = initial_admin_settings
        .get_account::<ConfigurationAccount>(initial_admin_settings.config_account)
        .await;

    println!("config_acc is {:?}", config_acc);

    assert_eq!(config_acc.config.dev_addr, new_dev_addr);

    handle_admin_ix(admin_ix_set_fees, &initial_admin_settings).await?;

    let config_acc = initial_admin_settings
        .get_account::<ConfigurationAccount>(initial_admin_settings.config_account)
        .await;

    println!("config_acc is {:?}", config_acc);

    assert_eq!(config_acc.config.fee_config, new_fee_config);

    handle_admin_ix(admin_ix_set_referral_balance, &initial_admin_settings).await?;

    let config_acc = initial_admin_settings
        .get_account::<ConfigurationAccount>(initial_admin_settings.config_account)
        .await;

    assert_eq!(
        config_acc.config.referral_token_address,
        admin_ix_set_referral_balance.new_referral_token_address
    );

    println!("config_acc is {:?}", config_acc);

    handle_admin_ix(admin_ix_set_secondary_token, &initial_admin_settings).await?;

    let config_acc = initial_admin_settings
        .get_account::<ConfigurationAccount>(initial_admin_settings.config_account)
        .await;

    assert_eq!(
        config_acc.config.secondary_token_address,
        admin_ix_set_secondary_token.new_secondary_token_address
    );

    println!("config_acc is {:?}", config_acc);
    let config_acc = initial_admin_settings
        .get_account::<ConfigurationAccount>(initial_admin_settings.config_account)
        .await;

    assert_eq!(
        config_acc.blacklisted_countries,
        vec![92, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    handle_admin_ix(
        admin_ix_add_country_code_to_blacklist,
        &initial_admin_settings,
    )
    .await?;
    let config_acc = initial_admin_settings
        .get_account::<ConfigurationAccount>(initial_admin_settings.config_account)
        .await;
    assert_eq!(
        config_acc.blacklisted_countries,
        vec![92, 0, 0, 0, 0, 0, 0, 0, 0, 0, 100]
    );
    handle_admin_ix(
        admin_ix_remove_country_code_to_blacklist,
        &initial_admin_settings,
    )
    .await?;
    let config_acc = initial_admin_settings
        .get_account::<ConfigurationAccount>(initial_admin_settings.config_account)
        .await;
    assert_eq!(
        config_acc.blacklisted_countries,
        vec![92, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );

    handle_admin_ix(admin_ix_set_new_admin, &initial_admin_settings).await?;

    let config_acc = initial_admin_settings
        .get_account::<ConfigurationAccount>(initial_admin_settings.config_account)
        .await;

    assert_eq!(
        config_acc.config.admin_key,
        admin_ix_set_new_admin.new_admin_key
    );

    Ok(())
}

pub async fn handle_admin_ix<T: ClientInstruction>(
    ix: T,
    initial_admin_settings: &TestInitialize,
) -> std::result::Result<(), TransportError> {
    send_tx(&initial_admin_settings.context.solana, ix).await?;

    Ok(())
}
