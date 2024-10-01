use std::ops::DerefMut;

use super::*;
use anchor_spl::associated_token;
use uncx_solana_lp_locker::instructions::FeePaymentMethod;
use uncx_solana_lp_locker::state::MAX_LP_TRACK_PER_ACCOUNT;

#[derive(Clone, Copy)]

pub struct TestCreateLockerFixture {
    pub fee_payment_method: FeePaymentMethod,
    pub referral_wallet: Option<ReferalSettings>,
    pub locker_wallet: UserSettings,
    pub referral_token_account: OptionalKey,
    pub referral_secondary_token_account: OptionalKey,
    pub user_secondary_token_account: OptionalKey,
    pub user_lp_token_account: Pubkey,
    pub user_lp_authority: Option<TestKeypair>,
    pub user_secondary_token_authority: Option<TestKeypair>,
    pub raydium_amm_accounts: RaydiumAccount,
    pub whitelisted_signer: Option<TestKeypair>,
    pub lock_owner: Pubkey, /*user_info_account : Pubkey,
                             *user_info_lp_tracker_acc : Pubkey, */
}

impl TestCreateLockerFixture {
    pub fn create_fixture_native(
        initial_admin_settings: &TestInitialize,
        referral_amount: Option<u64>,
    ) -> Self {
        let locker_settings = UserSettings {
            user_wallet: TestKeypair::new(),
            second_authority: TestKeypair::new().into(),
        };

        let referral_mint = initial_admin_settings.referral_mint;

        let referral_settings = ReferalSettings {
            referral_wallet: TestKeypair::new(),
        };

        //add lamports to the account to prevent rent exemption errors
        //https://github.com/solana-labs/solana/issues/23670
        initial_admin_settings
            .add_system_account(referral_settings.referral_wallet.pubkey(), WAD)
            .expect("failed to set account data in create fixture native");

        let referral_token_account_address = referral_settings.get_ata_for_mint(&referral_mint);

        let referral_token_account = initial_admin_settings.create_token_account(
            referral_settings.referral_wallet.pubkey(),
            referral_token_account_address,
            referral_mint,
            referral_amount.unwrap_or(1000_0),
        );

        let raydium_accounts = RaydiumAccount {
            serum_market_id: initial_admin_settings
                .context
                .raydium_test_fixture
                .amm_info
                .market,
            amm_id: initial_admin_settings.raydium_amm_info_acc,
            amm_lp_mint: initial_admin_settings.raydium_lp_mint,
        };

        //locker/ user setting

        let locker_lp_account_ = initial_admin_settings.create_token_account(
            locker_settings.user_wallet.pubkey(),
            locker_settings.get_ata_for_mint(&raydium_accounts.amm_lp_mint),
            raydium_accounts.amm_lp_mint,
            initial_admin_settings.total_lp_mint_balance() / 2,
        );

        TestCreateLockerFixture {
            lock_owner: locker_settings.user_wallet.pubkey(),
            referral_wallet: referral_settings.into(),
            user_lp_authority: locker_settings.user_wallet.into(),
            locker_wallet: locker_settings,
            user_secondary_token_account: None,
            user_secondary_token_authority: None,
            referral_token_account: referral_token_account.into(),
            raydium_amm_accounts: raydium_accounts,
            referral_secondary_token_account: None,
            whitelisted_signer: None,
            user_lp_token_account: locker_lp_account_,
            fee_payment_method: FeePaymentMethod::Native,
        }
    }

    pub fn create_fixture_native_no_referral(initial_admin_settings: &TestInitialize) -> Self {
        let locker_settings = UserSettings {
            user_wallet: TestKeypair::new(),
            second_authority: TestKeypair::new().into(),
        };

        let raydium_accounts = RaydiumAccount {
            serum_market_id: initial_admin_settings
                .context
                .raydium_test_fixture
                .amm_info
                .market,
            amm_id: initial_admin_settings.raydium_amm_info_acc,
            amm_lp_mint: initial_admin_settings.raydium_lp_mint,
        };
        //locker/ user setting

        let locker_lp_account_ = initial_admin_settings.create_token_account(
            locker_settings.user_wallet.pubkey(),
            locker_settings.get_ata_for_mint(&raydium_accounts.amm_lp_mint),
            raydium_accounts.amm_lp_mint,
            initial_admin_settings.total_lp_mint_balance() / 2,
        );

        TestCreateLockerFixture {
            lock_owner: locker_settings.user_wallet.pubkey(),
            referral_wallet: None.into(),
            user_lp_authority: locker_settings.user_wallet.into(),
            locker_wallet: locker_settings,
            user_secondary_token_account: None,
            user_secondary_token_authority: None,
            referral_token_account: None.into(),
            raydium_amm_accounts: raydium_accounts,
            referral_secondary_token_account: None,
            whitelisted_signer: None,
            user_lp_token_account: locker_lp_account_,
            fee_payment_method: FeePaymentMethod::Native,
        }
    }

    pub fn create_fixture_non_native(
        initial_admin_settings: &TestInitialize,
        referral_amount: Option<u64>,
    ) -> Self {
        let locker_settings = UserSettings {
            user_wallet: TestKeypair::new(),
            second_authority: TestKeypair::new().into(),
        };

        let locker_second_authority = {
            if locker_settings.second_authority.is_some() {
                locker_settings.second_authority.unwrap()
            } else {
                locker_settings.user_wallet
            }
        };

        let secondary_mint = initial_admin_settings.secondary_mint;

        // println!("secondary mint is {:?}", secondary_mint);

        let referral_mint = initial_admin_settings.referral_mint;

        let locker_secondary_token_account_address =
            locker_settings.get_ata_for_mint(&secondary_mint);

        let locker_secondary_token_account = initial_admin_settings.create_token_account(
            locker_second_authority.pubkey(),
            locker_secondary_token_account_address,
            secondary_mint,
            1_000_000,
        );

        let referral_settings = ReferalSettings {
            referral_wallet: TestKeypair::new(),
        };

        let referral_token_account_address = referral_settings.get_ata_for_mint(&referral_mint);

        let referral_secondary_token_account_address =
            referral_settings.get_ata_for_mint(&initial_admin_settings.secondary_mint);

        let referral_token_account = initial_admin_settings.create_token_account(
            referral_settings.referral_wallet.pubkey(),
            referral_token_account_address,
            referral_mint,
            referral_amount.unwrap_or(1_000_0000),
        );

        let referral_secondary_token_account = initial_admin_settings.create_token_account(
            referral_settings.referral_wallet.pubkey(),
            referral_secondary_token_account_address,
            secondary_mint,
            1_000_0,
        );

        let raydium_accounts = RaydiumAccount {
            serum_market_id: initial_admin_settings
                .context
                .raydium_test_fixture
                .amm_info
                .market,
            amm_id: initial_admin_settings.raydium_amm_info_acc,
            amm_lp_mint: initial_admin_settings.raydium_lp_mint,
        };

        //locker/ user setting

        let locker_lp_account_ = initial_admin_settings.create_token_account(
            locker_settings.user_wallet.pubkey(),
            locker_settings.get_ata_for_mint(&raydium_accounts.amm_lp_mint),
            raydium_accounts.amm_lp_mint,
            initial_admin_settings.total_lp_mint_balance() / 2,
        );

        TestCreateLockerFixture {
            lock_owner: locker_settings.user_wallet.pubkey(),
            referral_wallet: referral_settings.into(),
            user_lp_authority: locker_settings.user_wallet.into(),
            locker_wallet: locker_settings,
            user_secondary_token_account: locker_secondary_token_account.into(),
            user_secondary_token_authority: locker_second_authority.into(),
            referral_token_account: referral_token_account.into(),
            raydium_amm_accounts: raydium_accounts,
            referral_secondary_token_account: referral_secondary_token_account.into(),
            whitelisted_signer: None,
            user_lp_token_account: locker_lp_account_,
            fee_payment_method: FeePaymentMethod::NonNative,
        }
    }

    fn create_fixture_non_native_no_referral(
        initial_admin_settings: &TestInitialize,
        burn_amount: Option<u64>,
    ) -> Self {
        let locker_settings = UserSettings {
            user_wallet: TestKeypair::new(),
            second_authority: None,
        };

        let locker_second_authority = {
            if locker_settings.second_authority.is_some() {
                locker_settings.second_authority.unwrap()
            } else {
                locker_settings.user_wallet
            }
        };

        let secondary_mint = initial_admin_settings.secondary_mint;

        let locker_secondary_token_account_address =
            locker_settings.get_ata_for_mint(&secondary_mint);

        let locker_secondary_token_account = initial_admin_settings.create_token_account(
            locker_second_authority.pubkey(),
            locker_secondary_token_account_address,
            secondary_mint,
            burn_amount.unwrap_or(1_000_000),
        );

        let raydium_accounts = RaydiumAccount {
            serum_market_id: initial_admin_settings
                .context
                .raydium_test_fixture
                .amm_info
                .market,
            amm_id: initial_admin_settings.raydium_amm_info_acc,
            amm_lp_mint: initial_admin_settings.raydium_lp_mint,
        };

        //locker/ user setting

        let locker_lp_account_ = initial_admin_settings.create_token_account(
            locker_settings.user_wallet.pubkey(),
            locker_settings.get_ata_for_mint(&raydium_accounts.amm_lp_mint),
            raydium_accounts.amm_lp_mint,
            initial_admin_settings.total_lp_mint_balance() / 2,
        );

        TestCreateLockerFixture {
            lock_owner: locker_settings.user_wallet.pubkey(),
            referral_wallet: None.into(),
            user_lp_authority: locker_settings.user_wallet.into(),
            locker_wallet: locker_settings,
            user_secondary_token_account: locker_secondary_token_account.into(),
            user_secondary_token_authority: locker_second_authority.into(),
            referral_token_account: None.into(),
            raydium_amm_accounts: raydium_accounts,
            referral_secondary_token_account: None.into(),
            whitelisted_signer: None,
            user_lp_token_account: locker_lp_account_,
            fee_payment_method: FeePaymentMethod::NonNative,
        }
    }

    fn create_whitelisted_fixture(
        initial_admin_settings: &TestInitialize,
        whitelisted: Option<TestKeypair>,
    ) -> Self {
        let locker_settings = UserSettings {
            user_wallet: TestKeypair::new(),
            second_authority: None,
        };

        let raydium_accounts = RaydiumAccount {
            serum_market_id: initial_admin_settings
                .context
                .raydium_test_fixture
                .amm_info
                .market,
            amm_id: initial_admin_settings.raydium_amm_info_acc,
            amm_lp_mint: initial_admin_settings.raydium_lp_mint,
        };

        //locker/ user setting

        let locker_lp_account_ = initial_admin_settings.create_token_account(
            locker_settings.user_wallet.pubkey(),
            locker_settings.get_ata_for_mint(&raydium_accounts.amm_lp_mint),
            raydium_accounts.amm_lp_mint,
            initial_admin_settings.total_lp_mint_balance() / 2,
        );

        TestCreateLockerFixture {
            lock_owner: locker_settings.user_wallet.pubkey(),
            referral_wallet: None,
            user_secondary_token_account: None,
            user_secondary_token_authority: None,
            referral_token_account: None,
            referral_secondary_token_account: None,
            whitelisted_signer: whitelisted,
            user_lp_authority: locker_settings.user_wallet.into(),
            locker_wallet: locker_settings,
            user_lp_token_account: locker_lp_account_,
            raydium_amm_accounts: raydium_accounts,
            fee_payment_method: FeePaymentMethod::Whitelisted,
        }
    }
}

#[tokio::test]

pub async fn test_create_and_lock_lp_non_native_no_referral() -> Result<(), TransportError> {
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
    let fixture = TestCreateLockerFixture::create_fixture_non_native_no_referral(
        &initial_admin_settings,
        None,
    );

    //lock _amount
    let lock_amount = (initial_admin_settings.total_lp_mint_balance() / 2).into();

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

    Ok(())
}
#[tokio::test]
pub async fn test_create_and_lock_lp_non_native_with_blacklisted_country(
) -> Result<(), TransportError> {
    //fee payment method

    println!("Starting Create and Lock Lp Ix with a blacklisted country");

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
    let code = config_acc.blacklisted_countries.get(0).copied();

    let Err(_) = handle_create_and_lock_lp(
        &initial_admin_settings,
        next_locker_id,
        next_lp_tracker_acc_index,
        fixture,
        config_acc,
        lock_amount,
        code,
    )
    .await
    else {
        panic!("should have failed");
    };

    Ok(())
}

#[tokio::test]

pub async fn test_create_and_lock_lp_non_native() -> Result<(), TransportError> {
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
        None,
    )
    .await?;

    Ok(())
}

#[tokio::test]

pub async fn test_whitelisted_fixture() -> Result<(), TransportError> {
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
    let whitelist_kp = TestKeypair::new();

    test_add_whitelist::handle_test_add_whitelist(&initial_admin_settings, whitelist_kp.pubkey())
        .await?;

    //build raydium accounts

    //lp creation related info
    let next_locker_id = config_acc.config.next_locker_unique_id;

    let next_lp_tracker_acc_index = 0;

    //token accounts (referral,secondary,lp)
    let fixture = TestCreateLockerFixture::create_whitelisted_fixture(
        &initial_admin_settings,
        whitelist_kp.into(),
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

    Ok(())
}

#[tokio::test]

pub async fn test_create_and_lock_lp_native_without_referral() -> Result<(), TransportError> {
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
    let fixture =
        TestCreateLockerFixture::create_fixture_native_no_referral(&initial_admin_settings);

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

    Ok(())
}

#[tokio::test]

pub async fn test_create_and_lock_lp_native() -> Result<(), TransportError> {
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
    let fixture =
        TestCreateLockerFixture::create_fixture_native(&initial_admin_settings, 1000_000.into());

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

    Ok(())
}

#[tokio::test]

pub async fn test_create_and_lock_lp_native_without_sufficient_referral(
) -> Result<(), TransportError> {
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
    let fixture =
        TestCreateLockerFixture::create_fixture_native(&initial_admin_settings, 100.into());

    //lock _amount
    let lock_amount = 1_000_001.into();

    // println!("going into handle");

    let Err(_) = handle_create_and_lock_lp(
        &initial_admin_settings,
        next_locker_id,
        next_lp_tracker_acc_index,
        fixture,
        config_acc,
        lock_amount,
        None,
    )
    .await
    else {
        return Err(TransportError::Custom("should have failed but dint".into()));
    };

    Ok(())
}

/*

    initial_admin_settings: &TestInitialize,
    whitelist_kp: Option<TestKeypair>,
    next_locker_nonce: u64,
    payment_method: FeePaymentMethod,
    next_lp_tracker_acc_index: u8,
    test_fixture: TestCreateLockerFixture,
    initial_config: ConfigurationAccount,
    dev_wallet: Option<TestKeypair>,
*/

// #[allow(unused)]

// #[allow(unused)]
#[derive(Clone, Copy)]

pub struct ReferalSettings {
    referral_wallet: TestKeypair,
}

impl ReferalSettings {
    // #[allow(unused)]

    pub fn get_ata_for_mint(&self, mint: &Pubkey) -> Pubkey {
        get_ata_account(&self.referral_wallet.pubkey(), &mint)
    }
}

// #[allow(unused)]
#[derive(Clone, Copy)]

pub struct UserSettings {
    pub user_wallet: TestKeypair,
    second_authority: Option<TestKeypair>,
}

impl UserSettings {
    // #[allow(unused)]

    pub fn get_ata_for_mint(&self, mint: &Pubkey) -> Pubkey {
        get_ata_account(&self.user_wallet.pubkey(), &mint)
    }
}

// #[allow(unused)]

pub async fn handle_create_and_lock_lp(
    initial_admin_settings: &TestInitialize,

    next_locker_nonce: u64,
    next_lp_tracker_acc_index: u8,
    test_fixture: TestCreateLockerFixture,
    initial_config: ConfigurationAccount,
    input_lock_amount: Option<u64>,
    country_code: Option<u8>,
) -> std::result::Result<(), TransportError> {
    //println!("in handle");
    let lp_mint = test_fixture.raydium_amm_accounts.amm_lp_mint;

    let secondary_mint = initial_config.config.secondary_token_address;

    // println!("secondary mintttttt  in handle is {:?}", secondary_mint);
    assert_eq!(
        initial_config.config.secondary_token_address.unwrap(),
        initial_admin_settings.secondary_mint
    );

    let lock_amount = input_lock_amount.unwrap_or_else(|| WAD);

    let create_lock_ix = CreateAndLockLpInstruction::new(
        lp_mint,
        next_locker_nonce,
        initial_admin_settings,
        lock_amount,
        initial_admin_settings
            .context
            .solana
            .get_clock()
            .await
            .unix_timestamp
            + 86400,
        country_code.unwrap_or(1),
        test_fixture.fee_payment_method,
        test_fixture.whitelisted_signer,
        test_fixture.raydium_amm_accounts.amm_id,
        next_lp_tracker_acc_index as u64 / MAX_LP_TRACK_PER_ACCOUNT as u64,
        test_fixture
            .referral_wallet
            .as_ref()
            .map(|kp| kp.referral_wallet.pubkey()),
        test_fixture.referral_token_account,
        secondary_mint
            .as_ref()
            .map(|_| test_fixture.referral_secondary_token_account)
            .flatten(),
        test_fixture.user_lp_token_account,
        secondary_mint
            .map(|_| test_fixture.user_secondary_token_account)
            .flatten(),
        test_fixture.user_secondary_token_authority,
        secondary_mint,
        secondary_mint.map(|_| associated_token::ID),
        initial_config.config.dev_addr.into(),
        Some(initial_admin_settings.dev_wallet_keypair)
            .map(|dev_kp| get_ata_account(&dev_kp.pubkey(), &lp_mint)),
        test_fixture.user_lp_authority,
        test_fixture.lock_owner,
        initial_admin_settings.context.raydium_test_fixture,
    );

    send_tx(&initial_admin_settings.context.solana, create_lock_ix).await?;

    Ok(())
}
