#![allow(dead_code)]

use std::{
    cell::{RefCell, RefMut},
    sync::{Arc, RwLock},
};

use crate::program_test::setup::{
    //create_open_orders_account,
    // create_open_orders_indexer,
    Token,
};
use anchor_lang::{AccountDeserialize, Key};
pub use client::*;
pub use cookies::*;
use load_raydium_amm_acc::{RaydiumTestFixture, RaydiumTestFixtureBuilder};
use log::*;
pub use solana::*;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::{program_option::COption, program_pack::Pack};
use solana_program_test::*;
pub use solana_sdk::transport::TransportError;
use solana_sdk::{account::AccountSharedData, pubkey::Pubkey, signer::Signer};
use spl_token::{state::*, *};
use uncx_solana_lp_locker::raydium_port;
use uncx_solana_lp_locker::{Config, FeesConfig};
// u
// use uncx_solana_lp_locker::{
//     raydium_port::{self, AmmStatus},
//     Config, FeesConfig,
// };
pub use utils::*;
pub mod client;
pub mod cookies;
pub mod load_raydium_amm_acc;
// pub mod market_keys_utils;
pub mod setup;
pub mod solana;
pub mod utils;

pub const WAD: u64 = 1_000_000_000;

pub type OptionalKey = Option<Pubkey>;

pub struct TestInitialize {
    pub context: TestContext,
    pub config_account: Pubkey,
    pub uncx_authority_acc: Pubkey,
    pub mints: Vec<MintCookie>,
    //coin token account
    pub owner_token_0: Pubkey,
    //mint token
    pub owner_token_1: Pubkey,

    pub raydium_amm_info_acc: Pubkey,
    pub raydium_lp_mint: Pubkey,
    //the mints which owner_toekn and token1 refer to
    pub tokens: Vec<Token>,
    pub payer: TestKeypair,
    pub admin: TestKeypair,
    pub dev_wallet_keypair: TestKeypair,
    pub secondary_mint: Pubkey,
    pub referral_mint: Pubkey,
}

impl TestInitialize {
    pub fn add_system_account(&self, address: Pubkey, lamports: u64) -> Result<(), TransportError> {
        self.get_account_adder().set_account(
            &address,
            &AccountSharedData::new(lamports, 0, &anchor_lang::system_program::ID),
        );

        Ok(())
    }
    pub fn total_lp_mint_balance(&self) -> u64 {
        self.context.raydium_test_fixture.amm_info.lp_amount
    }

    pub async fn get_account<T: AccountDeserialize>(&self, address: Pubkey) -> T {
        self.context.solana.get_account::<T>(address).await
    }

    pub async fn get_optional_account<T: AccountDeserialize + anchor_lang::prelude::Space>(
        &self,
        address: Pubkey,
    ) -> Option<T> {
        let acc_data = self.context.solana.get_account_data(address).await;

        let Some(data) = acc_data else {
            return None;
        };

        if data.len() >= 8 + T::INIT_SPACE {
            return self.context.solana.get_account::<T>(address).await.into();
        } else {
            return None;
        }
    }

    pub fn get_lock_owner(&self) -> TestKeypair {
        self.context.users.get(0).expect("failed to get user").key
    }

    pub fn get_user_lp_token_acc(&self) -> Pubkey {
        self.context
            .users
            .get(0)
            .expect("")
            .token_accounts
            .get(0)
            .expect("failed to find token account")
            .key()
    }

    pub fn get_lp_locker_authority_acc(&self) -> TestKeypair {
        self.context.users.get(0).expect("").key
    }

    pub fn get_lp_mint(&self) -> Pubkey {
        self.context.mints.get(0).expect("mint not found").pubkey
    }
}

//use to create amm info accounts
//create token accounts directly
pub trait AddPacked {
    fn add_packable_account<T: Pack>(
        &mut self,
        pubkey: Pubkey,
        amount: u64,
        data: &T,
        owner: &Pubkey,
    );
}

impl AddPacked for ProgramTest {
    fn add_packable_account<T: Pack>(
        &mut self,
        pubkey: Pubkey,
        amount: u64,
        data: &T,
        owner: &Pubkey,
    ) {
        let mut account = solana_sdk::account::Account::new(amount, T::get_packed_len(), owner);

        data.pack_into_slice(&mut account.data);

        self.add_account(pubkey, account);
    }
}

impl AddPacked for ProgramTestContext {
    fn add_packable_account<T: Pack>(
        &mut self,
        pubkey: Pubkey,
        amount: u64,
        data: &T,
        owner: &Pubkey,
    ) {
        let mut account = solana_sdk::account::Account::new(amount, T::get_packed_len(), owner);

        //   shared_account.set_data_from_slice(new_data)
        data.pack_into_slice(&mut account.data);

        let shared_account = AccountSharedData::from(account);

        self.set_account(&pubkey, &shared_account)
        // self. (pubkey, account);
    }
}

struct LoggerWrapper {
    inner: env_logger::Logger,
    capture: Arc<RwLock<Vec<String>>>,
}

impl Log for LoggerWrapper {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.inner.enabled(metadata)
    }

    fn log(&self, record: &log::Record) {
        if record
            .target()
            .starts_with("solana_runtime::message_processor")
        {
            let msg = record.args().to_string();

            if let Some(data) = msg.strip_prefix("Program log: ") {
                self.capture.write().unwrap().push(data.into());
            } else if let Some(data) = msg.strip_prefix("Program data: ") {
                self.capture.write().unwrap().push(data.into());
            }
        }

        self.inner.log(record);
    }

    fn flush(&self) {}
}

#[derive(Default)]

pub struct TestContextBuilder {
    test: ProgramTest,
    logger_capture: Arc<RwLock<Vec<String>>>,
    raydium_test_fixture: RaydiumTestFixture,
}

//TODO!(aaraN) - replace with `matklad`oncecell crate
lazy_static::lazy_static! {
    static ref LOGGER_CAPTURE: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(vec![]));
    static ref LOGGER_LOCK: Arc<RwLock<()>> = Arc::new(RwLock::new(()));
}

impl TestContextBuilder {
    pub async fn new() -> anyhow::Result<Self> {
        // We need to intercept logs to capture program log output
        let log_filter = "solana_rbpf=trace,\
                    solana_runtime::message_processor=debug,\
                    solana_runtime::system_instruction_processor=trace,\
                    solana_program_test=info";

        let env_logger =
            env_logger::Builder::from_env(env_logger::Env::new().default_filter_or(log_filter))
                .format_timestamp_nanos()
                .build();

        let _ = log::set_boxed_logger(Box::new(LoggerWrapper {
            inner: env_logger,
            capture: LOGGER_CAPTURE.clone(),
        }));

        // hack to fix https://github.com/coral-xyz/anchor/issues/2738
        pub fn fixed_entry(
            program_id: &Pubkey,
            accounts: &[anchor_lang::prelude::AccountInfo],
            data: &[u8],
        ) -> anchor_lang::solana_program::entrypoint::ProgramResult {
            let extended_lifetime_accs = unsafe {
                core::mem::transmute::<_, &[anchor_lang::prelude::AccountInfo<'_>]>(accounts)
            };

            uncx_solana_lp_locker::entry(program_id, extended_lifetime_accs, data)
        }

        let mut test = ProgramTest::new(
            "uncx_solana_lp_locker",
            uncx_solana_lp_locker::id(),
            None,
            // processor!(fixed_entry),
        );
        test.add_program(
            "raydium_program",
            uncx_solana_lp_locker::raydium_amm::id(),
            None,
        );
        // use std::str::FromStr;
        // let acc_1 = Pubkey::from_str("6dM4TqWyWJsbx7obrdLcviBkTafD5E8av61zfU6jq57X")?;
        // let acc_2 = Pubkey::from_str("5M55nHUb5i5rDSmXMKApbQ3mZFKujwtboZLCkTYBQhuG")?;
        //
        // test.add_account_with_file_data(acc_1, 100000000, MPL_TOKEN_METADATA_ID, "coin_mint.so");
        // test.add_account_with_file_data(acc_2, 100000000, MPL_TOKEN_METADATA_ID, "pc_mint.so");
        #[allow(non_snake_case)]
        let TEST_AMM_ID: Pubkey =
            solana_sdk::pubkey!("3bYAfRecZnzYrDtGU7JhmcGkZgvH2FuyaZn2fCZ4NZkF");
        let raydium_text_fixture_builder = RaydiumTestFixtureBuilder::new(TEST_AMM_ID);
        let rpc_client = RpcClient::new(
            std::env::var("RPC").expect("please specify a valid mainnet solana rpc"),
        );
        let raydium_fixture = raydium_text_fixture_builder
            .build(&rpc_client, &mut test)
            .await?;
        // intentionally set to as tight as possible, to catch potential problems early
        #[cfg(feature = "anchor-debug")]
        test.set_compute_max_units(250000);
        #[cfg(not(feature = "anchor-debug"))]
        test.set_compute_max_units(215000);
        Ok(Self {
            test,
            logger_capture: LOGGER_CAPTURE.clone(),
            raydium_test_fixture: raydium_fixture,
        })
    }

    pub fn test(&mut self) -> &mut ProgramTest {
        &mut self.test
    }

    pub fn create_mints(&mut self) -> Vec<MintCookie> {
        let mut mints: Vec<MintCookie> = vec![
            MintCookie {
                index: 0,
                decimals: 6,
                unit: 10u64.pow(6) as f64,
                base_lot: 100_f64,
                quote_lot: 10_f64,
                pubkey: Pubkey::new_unique(),
                authority: TestKeypair::new(),
            }, // symbol: "MNGO".to_string()
        ];

        //create 10 mints
        for i in 1..10 {
            mints.push(MintCookie {
                index: i,
                decimals: 6,
                unit: 10u64.pow(6) as f64,
                base_lot: 100_f64,
                quote_lot: 10_f64,
                pubkey: Pubkey::default(),
                authority: TestKeypair::new(),
            });
        }

        // Add mints in loop
        //set each mint keypair to a unique one from default
        //TODO!(aaraN) - this loop seems redundant, just add Pubkey::new_unique() at creation
        for mint in &mut mints {
            let mint_pk = if mint.pubkey == Pubkey::default() {
                Pubkey::new_unique()
            } else {
                mint.pubkey
            };

            mint.pubkey = mint_pk;

            self.test.add_packable_account(
                mint_pk,
                u32::MAX as u64,
                &Mint {
                    is_initialized: true,
                    mint_authority: COption::Some(mint.authority.pubkey()),
                    decimals: mint.decimals,
                    ..Mint::default()
                },
                &spl_token::id(),
            );
        }

        mints
    }

    pub fn create_amm_lp_mint_acc(&mut self) -> Vec<MintCookie> {
        let amm_lp_mint = self.raydium_test_fixture.amm_info.lp_mint;

        let mut mints: Vec<MintCookie> = vec![
            MintCookie {
                index: 0,
                decimals: 6,
                unit: 10u64.pow(6) as f64,
                base_lot: 100_f64,
                quote_lot: 10_f64,
                pubkey: amm_lp_mint,
                authority: TestKeypair::new(),
            }, // symbol: "MNGO".to_string()
        ];

        // Add mints in loop
        //set each mint keypair to a unique one from default
        //TODO!(aaraN) - this loop seems redundant, just add Pubkey::new_unique() at creation
        for mint in &mut mints {
            let mint_pk = if mint.pubkey == Pubkey::default() {
                Pubkey::new_unique()
            } else {
                mint.pubkey
            };

            mint.pubkey = mint_pk;

            self.test.add_packable_account(
                mint_pk,
                u32::MAX as u64,
                &Mint {
                    is_initialized: true,
                    mint_authority: COption::Some(mint.authority.pubkey()),
                    decimals: mint.decimals,
                    ..Mint::default()
                },
                &spl_token::id(),
            );
        }

        mints
    }

    pub fn create_mint(&mut self, supply: Option<u64>) -> MintCookie {
        let mint: MintCookie = MintCookie {
            index: 0,
            decimals: 6,
            unit: 10u64.pow(6) as f64,
            base_lot: 100_f64,
            quote_lot: 10_f64,
            pubkey: Pubkey::new_unique(),
            authority: TestKeypair::new(),
        };

        self.test.add_packable_account(
            mint.pubkey,
            u32::MAX as u64,
            &Mint {
                is_initialized: true,
                mint_authority: COption::Some(mint.authority.pubkey()),
                decimals: mint.decimals,
                supply: supply.unwrap_or(0),
                ..Mint::default()
            },
            &spl_token::id(),
        );

        mint
    }

    // pub fn create_raydium_amm_info_acc(&mut self) {
    //     let mut raydium_amm_info = raydium_port::AmmInfo::default();
    //
    //     raydium_amm_info.lp_mint = self.raydium_test_fixture.amm_info.lp_mint;
    //
    //     self.test.add_packable_account(
    //         *&(raydium_port::get_raydium_amm_info_key(&self.serum_market_account_key)),
    //         u32::MAX as u64,
    //         &raydium_amm_info,
    //         &raydium_amm::id(),
    //     )
    // }

    //create users and their respective token account for each mint specified
    pub fn create_users(&mut self, mints: &[MintCookie]) -> Vec<UserCookie> {
        let num_users = 4;

        let mut users = Vec::new();

        for _ in 0..num_users {
            let user_key = TestKeypair::new();

            self.test.add_account(
                user_key.pubkey(),
                solana_sdk::account::Account::new(
                    u32::MAX as u64,
                    0,
                    &solana_sdk::system_program::id(),
                ),
            );

            // give every user 10^18 (< 2^60) of every token
            // ~~ 1 trillion in case of 6 decimals
            let mut token_accounts = Vec::new();

            for mint in mints {
                let token_key = Pubkey::new_unique();

                // let ata_token_key =
                self.test.add_packable_account(
                    token_key,
                    u32::MAX as u64,
                    &spl_token::state::Account {
                        mint: mint.pubkey,
                        owner: user_key.pubkey(),
                        amount: 1_000_000_000_000_000_000,
                        state: spl_token::state::AccountState::Initialized,
                        ..spl_token::state::Account::default()
                    },
                    &spl_token::id(),
                );

                token_accounts.push(token_key);
            }

            users.push(UserCookie {
                key: user_key,
                token_accounts,
            });
        }

        users
    }

    pub async fn start_default(mut self) -> TestContext {
        //creates random mints
        //creates amm lp mint account
        let mut mints = self.create_amm_lp_mint_acc();

        let secondary_token_mint = self.create_mint(WAD.into());

        let referral_token_mint = self.create_mint(None);

        mints.push(secondary_token_mint);

        mints.push(referral_token_mint);

        //creates user and token accounts for each mint specified above.
        let users = self.create_users(&mints);
        let raydium_text_fixture = self.raydium_test_fixture;
        let solana = self.start().await;

        TestContext {
            solana,
            mints,
            users,
            admin: TestKeypair::new(),
            raydium_test_fixture: raydium_text_fixture,
        }
    }

    pub async fn start(self) -> Arc<SolanaCookie> {
        // self.test.set_compute_max_units(1_400_000);

        let mut context = self.test.start_with_context().await;

        let rent = context.banks_client.get_rent().await.unwrap();

        Arc::new(SolanaCookie {
            context: RefCell::new(context),
            rent,
            logger_capture: self.logger_capture.clone(),
            logger_lock: LOGGER_LOCK.clone(),
            last_transaction_log: RefCell::new(vec![]),
        })
    }
}

pub struct TestContext {
    pub solana: Arc<SolanaCookie>,
    pub mints: Vec<MintCookie>,
    pub users: Vec<UserCookie>,
    pub admin: TestKeypair,
    pub raydium_test_fixture: RaydiumTestFixture,
}

pub struct TestInitializeSettings {
    initial_config: uncx_solana_lp_locker::Config,
    initial_blacklisted_countries: Option<[u8; 10]>,
}

impl Default for TestInitializeSettings {
    fn default() -> TestInitializeSettings {
        let fee_config: FeesConfig = FeesConfig {
            native_fee: WAD,
            secondary_token_fee: 100 * WAD,
            secondary_token_discount_bps: 2000, // 20%
            liquidity_fee_bps: 100,             //1%
            referral_discount_bps: 1000,        //10%
            referral_share_bps: 2500,           //25%
        };

        let black_listed_countries = [92, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        let config = Config {
            fee_config,
            min_referral_balance: 10 * WAD,
            referral_token_address: None,
            secondary_token_address: None,
            admin_key: Pubkey::default(),
            dev_addr: Pubkey::default(),
            next_locker_unique_id: 0,
        };

        TestInitializeSettings {
            initial_blacklisted_countries: Some(black_listed_countries),
            initial_config: config,
        }
    }
}

impl TestContext {
    pub async fn new() -> Self {
        TestContextBuilder::new()
            .await
            .expect("env setup failed")
            .start_default()
            .await
    }

    pub async fn with_initialize_locker_program(
        mut args: TestInitializeSettings,
    ) -> Result<TestInitialize, TransportError> {
        let context = TestContextBuilder::new()
            .await
            .map_err(|e| {
                TransportError::Custom(format!(
                    "test initialization failed due to {}",
                    e.to_string()
                ))
            })?
            .start_default()
            .await;

        let solana = &context.solana.clone();

        let dev_kp = TestKeypair::new();

        //args settings
        args.initial_config.admin_key = context.admin.pubkey();

        args.initial_config.dev_addr = dev_kp.pubkey();

        args.initial_config.min_referral_balance = 1_000;

        args.initial_config.fee_config.liquidity_fee_bps = 1000; //10%
        args.initial_config.fee_config.native_fee = 1_000_000;

        args.initial_config.fee_config.secondary_token_fee = 100000;

        args.initial_config.fee_config.secondary_token_discount_bps = 1000; //10%

        args.initial_config.fee_config.referral_share_bps = 500; //5 %
        args.initial_config.fee_config.referral_discount_bps = 200; //2 %

        let payer = context.users[1].key;

        //has 2 mints now
        let mints = &context.mints[0..];

        // let amm_lp_mint: MintCookie = context.mints.get(0).unwrap().clone();
        let secondary_mint = context.mints.get(1).unwrap().clone();

        args.initial_config.secondary_token_address = Some(secondary_mint.pubkey);

        let referral_mint = context.mints.get(2).unwrap().clone();

        args.initial_config.referral_token_address = Some(referral_mint.pubkey);

        //gets first two token accounts for the first two mints specified above
        let owner_token_0 = context.users[0].token_accounts[0];

        let owner_token_1 = context.users[0].token_accounts[0];

        let tokens = Token::create(mints.to_vec());

        // Create a market

        let Ok(uncx_solana_lp_locker::accounts::InitializeConfig {
            config_account,

            // initial_admin,
            uncx_authority_acc,
            ..
        }) = send_tx(
            solana,
            InitializeLockerProgramInstruction {
                payer: payer,
                initial_config: args.initial_config,
                initial_black_listed_countries: args.initial_blacklisted_countries,
            },
        )
        .await
        else {
            println!("{:?}", solana.program_log());

            return Err(TransportError::Custom("something went wrong".to_string()));
        };

        let mints = mints.to_vec();

        let raydium_amm_info_acc_token0_token1_acc =
            raydium_port::get_raydium_amm_info_key(&context.raydium_test_fixture.amm_info.market);

        let amm_lp_mint = raydium_port::get_raydium_amm_lp_mint_key(
            &&context.raydium_test_fixture.amm_info.market,
        );

        Ok(TestInitialize {
            admin: context.admin.clone(),
            context,
            config_account: config_account.into(),
            uncx_authority_acc,
            mints,
            owner_token_0,
            owner_token_1,
            payer,
            tokens,
            raydium_amm_info_acc: raydium_amm_info_acc_token0_token1_acc,
            raydium_lp_mint: amm_lp_mint,
            dev_wallet_keypair: dev_kp,
            secondary_mint: secondary_mint.pubkey,
            referral_mint: referral_mint.pubkey,
        })
    }
}

impl TestInitialize {
    pub fn create_token_account(
        &self,
        owner: Pubkey,
        token_account_address: Pubkey,
        mint: Pubkey,
        token_amount: u64,
    ) -> Pubkey {
        self.context
            .solana
            .context
            .borrow_mut()
            .add_packable_account(
                token_account_address,
                u32::MAX as u64,
                &spl_token::state::Account {
                    mint: mint,
                    owner: owner,
                    amount: token_amount,
                    state: spl_token::state::AccountState::Initialized,
                    ..spl_token::state::Account::default()
                },
                &spl_token::id(),
            );

        token_account_address
    }

    pub fn get_account_adder(&self) -> RefMut<ProgramTestContext> {
        self.context.solana.context.borrow_mut()
    }
}
//
// pub struct RaydiumAccountBuilder {
//     pub serum_market_id: Pubkey,
// }
//
#[derive(Clone, Copy)]

pub struct RaydiumAccount {
    pub serum_market_id: Pubkey,
    pub amm_id: Pubkey,
    pub amm_lp_mint: Pubkey,
}
//
// impl RaydiumAccountBuilder {
//     pub fn build_raydium_accounts<T: AddPacked>(self, account_adder: &mut T) -> RaydiumAccount {
//         let mut raydium_amm_info: raydium_port::AmmInfo = raydium_port::AmmInfo::default();
//
//         raydium_amm_info.status = AmmStatus::Initialized as u64;
//
//         let serum_market_id = Pubkey::new_unique();
//
//         let raydium_amm_lp_mint = raydium_port::get_raydium_amm_lp_mint_key(&self.serum_market_id);
//
//         raydium_amm_info.lp_mint = raydium_amm_lp_mint;
//
//         let raydium_amm_id = raydium_port::get_raydium_amm_info_key(&serum_market_id);
//
//         account_adder.add_packable_account(
//             raydium_amm_id,
//             u32::MAX as u64,
//             &raydium_amm_info,
//             &raydium_amm::id(),
//         );
//
//         //create mint
//
//         let mint: MintCookie = MintCookie {
//             index: 0,
//             decimals: 6,
//             unit: 10u64.pow(6) as f64,
//             base_lot: 100_f64,
//             quote_lot: 10_f64,
//             pubkey: raydium_amm_lp_mint,
//             authority: TestKeypair::new(),
//         };
//
//         account_adder.add_packable_account(
//             mint.pubkey,
//             u32::MAX as u64,
//             &Mint {
//                 is_initialized: true,
//                 mint_authority: COption::Some(mint.authority.pubkey()),
//                 decimals: mint.decimals,
//                 supply: 1_000_000_000_0000,
//                 ..Mint::default()
//             },
//             &spl_token::id(),
//         );
//
//         RaydiumAccount {
//             serum_market_id: self.serum_market_id,
//             amm_id: raydium_amm_id,
//             amm_lp_mint: raydium_amm_lp_mint,
//         }
//     }
// }
