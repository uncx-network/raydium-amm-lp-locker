#![allow(dead_code)]

use super::{load_raydium_amm_acc::RaydiumTestFixture, OptionalKey};
use anchor_lang::{prelude::*, system_program};
use anchor_spl::{associated_token, token::Token};

// use itertools::Itertools;
use once_cell::sync::Lazy;
use solana_program_test::BanksClientError;
use solana_sdk::{instruction, transport::TransportError};
use std::sync::Arc;
use uncx_solana_lp_locker::{
    accounts_ix::{
        BLACKLISTED_COUNTRIES_SEED, LP_LOCKER_SEED, LP_MARKER_SEED, UNCX_LOCKER_AUTHORITY_SEED,
        UNCX_LP_VAULT_ACCOUNT, USER_INFO_SEED, USER_LP_TRACKER_SEED,
    },
    constants::{MIGRATOR_SEED, WHITELIST_ACC_STATIC_SEED},
    instructions::FeePaymentMethod,
    Config, FeesConfig,
};

pub static UNCX_PROGRAM: Pubkey = uncx_solana_lp_locker::ID;
pub static CONFIG_ACCOUNT_ADDRESS: Lazy<Pubkey> = Lazy::new(|| get_config_account_address());

use super::{solana::SolanaCookie, utils::TestKeypair, TestInitialize};

pub fn get_config_account_address() -> Pubkey {
    let (config_account_address, _) =
        Pubkey::find_program_address(&[uncx_solana_lp_locker::CONFIG_ACCOUNT_SEED], &UNCX_PROGRAM);

    config_account_address
}

#[async_trait::async_trait(?Send)]

pub trait ClientAccountLoader {
    async fn load_bytes(&self, pubkey: &Pubkey) -> Option<Vec<u8>>;

    async fn load<T: AccountDeserialize>(&self, pubkey: &Pubkey) -> Option<T> {
        let bytes = self.load_bytes(pubkey).await?;

        AccountDeserialize::try_deserialize(&mut &bytes[..]).ok()
    }
}

#[async_trait::async_trait(?Send)]

impl ClientAccountLoader for &SolanaCookie {
    async fn load_bytes(&self, pubkey: &Pubkey) -> Option<Vec<u8>> {
        self.get_account_data(*pubkey).await
    }
}

// TODO: report error outwards etc
pub async fn send_tx<CI: ClientInstruction>(
    solana: &SolanaCookie,
    ix: CI,
) -> std::result::Result<CI::Accounts, TransportError> {
    let (accounts, instruction) = ix.to_instruction(solana).await;

    let signers = ix.signers();

    let instructions = vec![instruction];

    solana
        .process_transaction(&instructions, Some(&signers[..]))
        .await?;

    Ok(accounts)
}

pub async fn send_tx_and_get_ix_custom_error<CI: ClientInstruction>(
    solana: &SolanaCookie,
    ix: CI,
) -> Option<u32> {
    let tx_result = send_tx(solana, ix).await;

    if let Err(TransportError::TransactionError(
        solana_sdk::transaction::TransactionError::InstructionError(
            _,
            solana_sdk::instruction::InstructionError::Custom(err_num),
        ),
    )) = tx_result
    {
        Some(err_num)
    } else {
        None
    }
}

/// Build a transaction from multiple instructions

pub struct ClientTransaction {
    solana: Arc<SolanaCookie>,
    instructions: Vec<instruction::Instruction>,
    signers: Vec<TestKeypair>,
}

impl<'a> ClientTransaction {
    pub fn new(solana: &Arc<SolanaCookie>) -> Self {
        Self {
            solana: solana.clone(),
            instructions: vec![],
            signers: vec![],
        }
    }

    pub async fn add_instruction<CI: ClientInstruction>(&mut self, ix: CI) -> CI::Accounts {
        let solana: &SolanaCookie = &self.solana;

        let (accounts, instruction) = ix.to_instruction(solana).await;

        self.instructions.push(instruction);

        self.signers.extend(ix.signers());

        accounts
    }

    pub fn add_instruction_direct(&mut self, ix: instruction::Instruction) {
        self.instructions.push(ix);
    }

    pub fn add_signer(&mut self, keypair: TestKeypair) {
        self.signers.push(keypair);
    }

    pub async fn send(&self) -> std::result::Result<(), BanksClientError> {
        self.solana
            .process_transaction(&self.instructions, Some(&self.signers))
            .await
    }
}

#[async_trait::async_trait(?Send)]

pub trait ClientInstruction {
    type Accounts: anchor_lang::ToAccountMetas;

    type Instruction: anchor_lang::InstructionData;

    async fn to_instruction(
        &self,
        loader: impl ClientAccountLoader,
    ) -> (Self::Accounts, instruction::Instruction);

    fn signers(&self) -> Vec<TestKeypair>;
}

fn make_instruction(
    program_id: Pubkey,
    accounts: &impl anchor_lang::ToAccountMetas,
    data: impl anchor_lang::InstructionData,
) -> instruction::Instruction {
    instruction::Instruction {
        program_id,
        accounts: anchor_lang::ToAccountMetas::to_account_metas(accounts, None),
        data: anchor_lang::InstructionData::data(&data),
    }
}

// pub fn get_market_address(market: TestKeypair) -> Pubkey {
//     Pubkey::find_program_address(
//         &[b"Market".as_ref(), market.pubkey().to_bytes().as_ref()],
//         &openbook_v2::id(),
//     )
//     .0
// }
pub struct InitializeLockerProgramInstruction {
    pub payer: TestKeypair,
    pub initial_config: Config,
    pub initial_black_listed_countries: Option<[u8; 10]>,
}

#[async_trait::async_trait(?Send)]

impl ClientInstruction for InitializeLockerProgramInstruction {
    type Accounts = uncx_solana_lp_locker::accounts::InitializeConfig;

    type Instruction = uncx_solana_lp_locker::instruction::Initialize;

    async fn to_instruction(
        &self,
        _account_loader: impl ClientAccountLoader,
    ) -> (Self::Accounts, instruction::Instruction) {
        let instruction = uncx_solana_lp_locker::instruction::Initialize {
            initial_config: self.initial_config.clone(),
            initial_black_listed_countries: self.initial_black_listed_countries,
        };

        let config_account_address = UncxCoreProgramAccounts::get_config_address();

        let (uncx_authority_acc_key, _) = Pubkey::find_program_address(
            &[uncx_solana_lp_locker::accounts_ix::UNCX_LOCKER_AUTHORITY_SEED],
            &UNCX_PROGRAM,
        );

        let accounts = uncx_solana_lp_locker::accounts::InitializeConfig {
            payer: self.payer.pubkey(),
            config_account: config_account_address,
            initial_admin: uncx_solana_lp_locker::constants::INITIAL_ADMIN,
            system_program: System::id(),
            uncx_authority_acc: uncx_authority_acc_key,
        };

        let instruction = make_instruction(UNCX_PROGRAM, &accounts, instruction);

        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.payer]
    }
}

pub struct AddMigratorInstruction {
    pub migrator_address: Pubkey,
    pub payer: TestKeypair,
    pub admin: TestKeypair,
}

impl AddMigratorInstruction {
    pub fn get_migrator_pda_acc(&self) -> (Pubkey, u8) {
        let (migrator_pda_acc, bump) = Pubkey::find_program_address(
            &[
                uncx_solana_lp_locker::constants::MIGRATOR_SEED,
                self.migrator_address.as_ref(),
            ],
            &UNCX_PROGRAM,
        );

        (migrator_pda_acc, bump)
    }
}

#[async_trait::async_trait(?Send)]

impl ClientInstruction for AddMigratorInstruction {
    type Accounts = uncx_solana_lp_locker::accounts::AddMigrator;

    type Instruction = uncx_solana_lp_locker::instruction::AddMigrator;

    async fn to_instruction(
        &self,
        _account_loader: impl ClientAccountLoader,
    ) -> (Self::Accounts, instruction::Instruction) {
        let instruction = uncx_solana_lp_locker::instruction::AddMigrator {
            new_migrator_address_pda: self.migrator_address,
        };

        let (migrator_pda_acc, _) = Pubkey::find_program_address(
            &[
                uncx_solana_lp_locker::constants::MIGRATOR_SEED,
                self.migrator_address.as_ref(),
            ],
            &UNCX_PROGRAM,
        );

        let config_account_address = UncxCoreProgramAccounts::get_config_address();

        let accounts = uncx_solana_lp_locker::accounts::AddMigrator {
            payer: self.payer.pubkey(),
            config_account: config_account_address,
            system_program: System::id(),
            admin_sign: self.admin.pubkey(),
            migrator_marker_acc: migrator_pda_acc,
        };

        let instruction = make_instruction(UNCX_PROGRAM, &accounts, instruction);

        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.payer, self.admin]
    }
}

pub struct AddWhiteListInstruction {
    pub address_to_whitelist: Pubkey,
    pub payer: TestKeypair,
    pub admin: TestKeypair,
}

impl AddWhiteListInstruction {
    pub fn get_whitelisted_pda_acc(&self) -> (Pubkey, u8) {
        let (user_whitelist_pda_acc, bump) = Pubkey::find_program_address(
            &[
                uncx_solana_lp_locker::constants::WHITELIST_ACC_STATIC_SEED,
                self.address_to_whitelist.as_ref(),
            ],
            &UNCX_PROGRAM,
        );

        (user_whitelist_pda_acc, bump)
    }
}

#[async_trait::async_trait(?Send)]

impl ClientInstruction for AddWhiteListInstruction {
    type Accounts = uncx_solana_lp_locker::accounts::AddWhitelistAcc;

    type Instruction = uncx_solana_lp_locker::instruction::AddWhitelist;

    async fn to_instruction(
        &self,
        _account_loader: impl ClientAccountLoader,
    ) -> (Self::Accounts, instruction::Instruction) {
        let instruction = uncx_solana_lp_locker::instruction::AddWhitelist {
            whitelist_address: self.address_to_whitelist,
        };

        let (user_whitelist_pda_acc, _) = Pubkey::find_program_address(
            &[
                uncx_solana_lp_locker::constants::WHITELIST_ACC_STATIC_SEED,
                self.address_to_whitelist.as_ref(),
            ],
            &UNCX_PROGRAM,
        );

        let config_account_address = UncxCoreProgramAccounts::get_config_address();

        let accounts = uncx_solana_lp_locker::accounts::AddWhitelistAcc {
            payer: self.payer.pubkey(),
            config_account: config_account_address,
            system_program: System::id(),
            admin_sign: self.admin.pubkey(),
            user_whitelist_pda_acc,
        };

        let instruction = make_instruction(UNCX_PROGRAM, &accounts, instruction);

        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.payer, self.admin]
    }
}

pub struct RemoveWhitelistInstruction {
    pub whitelisted_address_to_remove: Pubkey,
    pub payer: TestKeypair,
    pub admin: TestKeypair,
}

#[async_trait::async_trait(?Send)]

impl ClientInstruction for RemoveWhitelistInstruction {
    type Accounts = uncx_solana_lp_locker::accounts::RemoveWhitelistAcc;

    type Instruction = uncx_solana_lp_locker::instruction::RemoveWhitelist;

    async fn to_instruction(
        &self,
        _account_loader: impl ClientAccountLoader,
    ) -> (Self::Accounts, instruction::Instruction) {
        let instruction = uncx_solana_lp_locker::instruction::RemoveWhitelist {
            whitelist_address: self.whitelisted_address_to_remove,
        };

        let (user_whitelist_pda_acc, _) = Pubkey::find_program_address(
            &[
                uncx_solana_lp_locker::constants::WHITELIST_ACC_STATIC_SEED,
                self.whitelisted_address_to_remove.as_ref(),
            ],
            &UNCX_PROGRAM,
        );

        let config_account_address = UncxCoreProgramAccounts::get_config_address();

        let accounts = uncx_solana_lp_locker::accounts::RemoveWhitelistAcc {
            receiver: self.payer.pubkey(),
            config_account: config_account_address,
            system_program: System::id(),
            admin_sign: self.admin.pubkey(),
            user_whitelist_pda_acc,
        };

        let instruction = make_instruction(UNCX_PROGRAM, &accounts, instruction);

        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.admin]
    }
}

pub struct CreateAndLockLpInstruction {
    pub locker_pda_acc: Pubkey,
    pub payer: TestKeypair,
    pub admin: TestKeypair,
    pub lock_owner: Pubkey,
    pub lock_amount: u64,
    pub unlock_date: i64,
    pub country_code: u8,

    fee_payment_method: FeePaymentMethod,
    pub whitelisted_user_kp: Option<TestKeypair>,
    pub whitelist_user: OptionalKey,
    pub amm_info_acc: Pubkey,
    pub user_lp_token_acc: Pubkey,
    pub derived_accounts: CreateAndLockLpInstructionDerivedAccounts,
    pub optional_accounts: CreateAndLockLpInstructionOptionalAccounts,
    pub user_whitelist_pda_acc: OptionalKey,

    pub lp_mint_acc: Pubkey,
    pub blacklisted_countries_acc: Pubkey,
    pub dev_wallet: Option<Pubkey>,
    pub amm_target_order_acc: Pubkey,
    pub coin_metadata_acc: Pubkey,
    pub pc_metadata_acc: Pubkey,
    pub pc_vault_acc: Pubkey,
    pub coin_vault_acc: Pubkey,
}

pub fn get_locker_pda_acc(next_lp_locker_nonce: u64) -> Pubkey {
    Pubkey::find_program_address(
        &[LP_LOCKER_SEED, next_lp_locker_nonce.to_le_bytes().as_ref()],
        &UNCX_PROGRAM,
    )
    .0
}

pub fn get_user_whitelist_pda_acc(whitelist_user: Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[WHITELIST_ACC_STATIC_SEED, whitelist_user.key().as_ref()],
        &UNCX_PROGRAM,
    )
    .0
}

pub fn get_ata_account(user: &Pubkey, mint: &Pubkey) -> Pubkey {
    associated_token::get_associated_token_address(user, mint)
}

pub struct UncxCoreProgramAccounts;

impl UncxCoreProgramAccounts {
    pub fn get_uncx_authority_acc() -> Pubkey {
        Pubkey::find_program_address(&[UNCX_LOCKER_AUTHORITY_SEED], &UNCX_PROGRAM).0
    }

    pub fn get_blacklisted_countries() -> Pubkey {
        let blacklisted_address =
            Pubkey::find_program_address(&[BLACKLISTED_COUNTRIES_SEED], &UNCX_PROGRAM).0;

        // println!("blacklisted countries address {}",blacklisted_address);
        blacklisted_address
    }

    pub fn get_locker_pda_address(next_locker_id: u64) -> Pubkey {
        get_locker_pda_acc(next_locker_id)
    }

    pub fn get_locker_token_vault(amm_id: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[UNCX_LP_VAULT_ACCOUNT, amm_id.key().as_ref()],
            &UNCX_PROGRAM,
        )
        .0
    }

    pub fn get_lp_marker_acc(amm_info_acc: Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[LP_MARKER_SEED, amm_info_acc.key().as_ref()],
            &UNCX_PROGRAM,
        )
        .0
    }

    pub fn user_info_address(user_address: Pubkey, amm_info_acc: Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[
                USER_INFO_SEED,
                user_address.key().as_ref(),
                amm_info_acc.as_ref(),
            ],
            &UNCX_PROGRAM,
        )
        .0
    }

    pub fn get_config_address() -> Pubkey {
        get_config_account_address()
    }

    pub fn user_lp_tracker_address(
        lock_owner: Pubkey,
        amm_info_acc: Pubkey,
        //total associated lockers of a particular amm id /255
        locker_count: u64,
    ) -> Pubkey {
        let actual_normalized_locker_count = locker_count / u8::MAX as u64;

        Pubkey::find_program_address(
            &[
                USER_LP_TRACKER_SEED,
                lock_owner.key().as_ref(),
                amm_info_acc.key().as_ref(),
                actual_normalized_locker_count.to_le_bytes().as_ref(),
            ],
            &UNCX_PROGRAM,
        )
        .0
    }
}

impl CreateAndLockLpInstruction {
    pub fn new(
        lp_mint: Pubkey,
        next_locker_nonce: u64,
        initial_admin_settings: &TestInitialize,
        lock_amount: u64,
        unlock_date: i64,
        country_code: u8,
        fee_payment_method: FeePaymentMethod,
        whitelist_user: Option<TestKeypair>,
        amm_info_acc: Pubkey,
        next_lp_acc_counter: u64,
        referral_wallet: OptionalKey,
        referral_token_wallet: OptionalKey,
        referral_secondary_token_account: OptionalKey,
        user_lp_token_account: Pubkey,
        user_secondary_token_burn_acc: OptionalKey,
        user_secondary_token_authority_acc: Option<TestKeypair>,
        secondary_token_mint: OptionalKey,
        associated_token_program: OptionalKey,
        dev_wallet: OptionalKey,
        dev_lp_token_acc: OptionalKey,
        user_lp_token_authority: Option<TestKeypair>,
        locker_owner: Pubkey,
        raydium_text_fixture: RaydiumTestFixture,
    ) -> Self {
        let derived_accounts = CreateAndLockLpInstructionDerivedAccounts {
            lp_locker_acc: get_locker_pda_acc(next_locker_nonce),
            global_lp_marker_acc: CreateAndLockLpInstructionDerivedAccounts::get_lp_marker_acc(
                amm_info_acc,
            ),
            user_whitelist_pda_acc: whitelist_user
                .map(|kp| get_user_whitelist_pda_acc(kp.pubkey())),
            user_info_acc: CreateAndLockLpInstructionDerivedAccounts::user_info_address(
                locker_owner,
                amm_info_acc,
            ),
            user_info_lp_tracker_acc:
                CreateAndLockLpInstructionDerivedAccounts::user_lp_tracker_address(
                    locker_owner,
                    amm_info_acc.key(),
                    next_lp_acc_counter,
                ),
            uncx_lock_lp_vault_acc:
                CreateAndLockLpInstructionDerivedAccounts::get_lp_locker_token_acc(amm_info_acc),
            uncx_authority: UncxCoreProgramAccounts::get_uncx_authority_acc(),
        };

        let optional_accounts = CreateAndLockLpInstructionOptionalAccounts {
            referral_wallet,
            referral_token_account: referral_token_wallet,
            referral_secondary_token_account,
            dev_lp_token_acc,
            user_lp_token_acc_authority: user_lp_token_authority,
            user_secondary_token_authority_acc,
            user_secondary_token_account: user_secondary_token_burn_acc,
            secondary_token_mint,
            associated_token_program,
        };

        let locker_acc = Self {
            locker_pda_acc: get_locker_pda_acc(next_locker_nonce),
            payer: initial_admin_settings.payer,
            admin: initial_admin_settings.admin,
            lock_owner: locker_owner,
            unlock_date,
            country_code,
            lock_amount,
            fee_payment_method,
            whitelist_user: whitelist_user.map(|kp| kp.pubkey()),
            dev_wallet,

            whitelisted_user_kp: whitelist_user,
            amm_info_acc,
            user_lp_token_acc: user_lp_token_account,
            derived_accounts,
            optional_accounts,
            user_whitelist_pda_acc: whitelist_user
                .map(|whitelist_kp| get_user_whitelist_pda_acc(whitelist_kp.pubkey())),
            lp_mint_acc: lp_mint,
            blacklisted_countries_acc: UncxCoreProgramAccounts::get_blacklisted_countries(),
            amm_target_order_acc: raydium_text_fixture.amm_info.target_orders,
            coin_metadata_acc: raydium_text_fixture.coin_mint_metadata,
            pc_metadata_acc: raydium_text_fixture.pc_mint_metadata,
            pc_vault_acc: raydium_text_fixture.amm_info.pc_vault,
            coin_vault_acc: raydium_text_fixture.amm_info.coin_vault,
        };

        locker_acc
    }
}

pub struct CreateAndLockLpInstructionDerivedAccounts {
    global_lp_marker_acc: Pubkey,
    lp_locker_acc: Pubkey,
    user_info_acc: Pubkey,
    user_info_lp_tracker_acc: Pubkey,
    user_whitelist_pda_acc: OptionalKey,
    uncx_lock_lp_vault_acc: Pubkey,
    uncx_authority: Pubkey,
}

impl CreateAndLockLpInstructionDerivedAccounts {
    //locker count = total number of locker count /255
    pub fn new(
        amm_id: Pubkey,
        user_address: Pubkey,
        lock_owner: Pubkey,
        locker_count: u64,
        locker_id: u64,
    ) -> Self {
        Self {
            lp_locker_acc: get_locker_pda_acc(locker_id),
            global_lp_marker_acc: Self::get_lp_marker_acc(amm_id),
            user_info_acc: Self::user_info_address(user_address, amm_id),
            user_info_lp_tracker_acc: Self::user_lp_tracker_address(
                lock_owner,
                amm_id,
                locker_count,
            ),
            user_whitelist_pda_acc: None,
            uncx_authority: UncxCoreProgramAccounts::get_uncx_authority_acc(),
            uncx_lock_lp_vault_acc: Self::get_lp_locker_token_acc(amm_id),
        }
    }

    pub fn get_lp_locker_token_acc(amm_info_acc: Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[UNCX_LP_VAULT_ACCOUNT, amm_info_acc.key().as_ref()],
            &UNCX_PROGRAM,
        )
        .0
    }

    pub fn get_lp_marker_acc(amm_info_acc: Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[LP_MARKER_SEED, amm_info_acc.key().as_ref()],
            &UNCX_PROGRAM,
        )
        .0
    }

    pub fn user_info_address(user_address: Pubkey, amm_info_acc: Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[
                USER_INFO_SEED,
                user_address.key().as_ref(),
                amm_info_acc.as_ref(),
            ],
            &UNCX_PROGRAM,
        )
        .0
    }

    pub fn user_lp_tracker_address(
        lock_owner: Pubkey,
        amm_info_acc: Pubkey,
        //total associated lockers of a particular amm id /255
        locker_count: u64,
    ) -> Pubkey {
        let actual_normalized_locker_count = locker_count / u8::MAX as u64;

        Pubkey::find_program_address(
            &[
                USER_LP_TRACKER_SEED,
                lock_owner.key().as_ref(),
                amm_info_acc.key().as_ref(),
                actual_normalized_locker_count.to_le_bytes().as_ref(),
            ],
            &UNCX_PROGRAM,
        )
        .0
    }
}

pub struct CreateAndLockLpInstructionOptionalAccounts {
    referral_token_account: OptionalKey,
    pub user_secondary_token_account: OptionalKey,
    pub referral_secondary_token_account: OptionalKey,
    pub user_secondary_token_authority_acc: Option<TestKeypair>,

    pub secondary_token_mint: OptionalKey,
    pub referral_wallet: OptionalKey,
    pub associated_token_program: OptionalKey,
    pub dev_lp_token_acc: OptionalKey,
    pub user_lp_token_acc_authority: Option<TestKeypair>,
}

impl From<&CreateAndLockLpInstruction> for uncx_solana_lp_locker::accounts::CreateAndLockLp {
    fn from(lp_ix: &CreateAndLockLpInstruction) -> Self {
        let config_account_address = get_config_account_address();

        let accounts = uncx_solana_lp_locker::accounts::CreateAndLockLp {
            payer: lp_ix.payer.pubkey(),
            global_lp_marker_acc: lp_ix.derived_accounts.global_lp_marker_acc,
            system_program: system_program::ID,
            lp_locker_acc: lp_ix.derived_accounts.lp_locker_acc,
            config_account: config_account_address,
            amm_info_acc: lp_ix.amm_info_acc,

            user_info_acc: lp_ix.derived_accounts.user_info_acc,
            user_info_lp_tracker_acc: lp_ix.derived_accounts.user_info_lp_tracker_acc,

            user_whitelist_pda_acc: lp_ix.user_whitelist_pda_acc,
            whitelist_address: lp_ix.whitelist_user,

            referral_secondary_token_account: lp_ix
                .optional_accounts
                .referral_secondary_token_account,
            referral_token_account: lp_ix.optional_accounts.referral_token_account,
            user_lp_token_acc: lp_ix.user_lp_token_acc,
            user_secondary_token_account: lp_ix.optional_accounts.user_secondary_token_account,
            user_secondary_token_authority_acc: lp_ix
                .optional_accounts
                .user_secondary_token_authority_acc
                .map(|kp| kp.pubkey()),
            secondary_token_mint: lp_ix.optional_accounts.secondary_token_mint,

            user_lp_token_authority_acc: lp_ix
                .optional_accounts
                .user_lp_token_acc_authority
                .map(|kp| kp.pubkey()),
            referral_wallet: lp_ix.optional_accounts.referral_wallet,

            lp_mint_acc: lp_ix.lp_mint_acc,

            uncx_lock_lp_vault_acc: lp_ix.derived_accounts.uncx_lock_lp_vault_acc,
            uncx_authority_acc: lp_ix.derived_accounts.uncx_authority,
            token_program: Token::id(),
            dev_lp_token_account: lp_ix.optional_accounts.dev_lp_token_acc,
            associated_token_program: lp_ix.optional_accounts.associated_token_program,
            dev_wallet: lp_ix.dev_wallet,
            coin_metadata_account: lp_ix.coin_metadata_acc,
            pc_metadata_account: lp_ix.pc_metadata_acc,
            amm_target_orders_info_acc: lp_ix.amm_target_order_acc,
            pc_vault_token_acc: lp_ix.pc_vault_acc,
            coin_vault_token_acc: lp_ix.coin_vault_acc,
        };

        accounts
    }
}

#[async_trait::async_trait(?Send)]

impl ClientInstruction for CreateAndLockLpInstruction {
    type Accounts = uncx_solana_lp_locker::accounts::CreateAndLockLp;

    type Instruction = uncx_solana_lp_locker::instruction::CreateAndLockLp;

    async fn to_instruction(
        &self,
        _account_loader: impl ClientAccountLoader,
    ) -> (Self::Accounts, instruction::Instruction) {
        let instruction = uncx_solana_lp_locker::instruction::CreateAndLockLp {
            lock_amount: self.lock_amount,
            lock_owner: self.lock_owner,
            unlock_date: self.unlock_date,
            country_code: self.country_code,
            referral_wallet_key: self.optional_accounts.referral_wallet,
            fee_payment_method: self.fee_payment_method.clone(),
            amm_info_acc_key: self.amm_info_acc,
        };
        println!("passed in lock owner is {}", instruction.lock_owner);
        println!(
            "passed in referral wallet is {:?}",
            self.optional_accounts.referral_wallet
        );

        let accounts = uncx_solana_lp_locker::accounts::CreateAndLockLp::from(self);
        println!("referral wallet account is {:?}", accounts.referral_wallet);

        let instruction = make_instruction(UNCX_PROGRAM, &accounts, instruction);

        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        let mut signer_vec: Vec<TestKeypair> = Vec::with_capacity(10);

        signer_vec.push(self.payer);

        self.whitelisted_user_kp.and_then(|kp| {
            //  println!("whitelisted user is {}", kp.pubkey());

            signer_vec.push(kp);

            self.whitelisted_user_kp
        });

        self.optional_accounts
            .user_lp_token_acc_authority
            .map(|user_lp_authority| {
                //   println!("user lp authority  is {}", user_lp_authority.pubkey());

                signer_vec.push(user_lp_authority)
            });

        self.optional_accounts
            .user_secondary_token_authority_acc
            .and_then(|kp| {
                //   println!("secondary token acc authority is {}", kp.pubkey());

                signer_vec.push(kp);

                self.optional_accounts.user_secondary_token_authority_acc
            });

        signer_vec
    }
}

#[derive(Clone, Copy)]

pub struct WithdrawLpFromLockerInstruction {
    pub lock_owner: TestKeypair,
    pub withdraw_amount: u64,
    pub locker_id: u64,
    pub user_lp_token_acc: Pubkey,
    pub amm_id: Pubkey,
    pub global_lp_marker_acc: Pubkey,
    pub lp_locker_acc: Pubkey,
    pub user_info_acc: OptionalKey,
    pub user_info_lp_tracker_acc: OptionalKey,
    pub uncx_lock_lp_vault_acc: Pubkey,
    pub uncx_authority: Pubkey,
    pub payer: TestKeypair,
}

impl WithdrawLpFromLockerInstruction {
    pub fn new(
        locker_id: u64,
        lock_owner: TestKeypair,
        amm_id: Pubkey,
        total_locker_count_for_user: u64,
        withdraw_amount: u64,
        payer: TestKeypair,
        lp_mint: Pubkey,
    ) -> Self {
        let derived_accounts = CreateAndLockLpInstructionDerivedAccounts::new(
            amm_id,
            lock_owner.pubkey(),
            lock_owner.pubkey(),
            total_locker_count_for_user,
            locker_id,
        );

        let user_lp_token_acc = get_ata_account(&lock_owner.pubkey(), &lp_mint);

        Self {
            lock_owner,
            locker_id,
            withdraw_amount,
            amm_id,
            payer,
            user_lp_token_acc,
            global_lp_marker_acc: derived_accounts.global_lp_marker_acc,
            lp_locker_acc: derived_accounts.lp_locker_acc,
            user_info_acc: derived_accounts.user_info_acc.into(),
            user_info_lp_tracker_acc: derived_accounts.user_info_lp_tracker_acc.into(),
            uncx_lock_lp_vault_acc: derived_accounts.uncx_lock_lp_vault_acc,
            uncx_authority: derived_accounts.uncx_authority,
        }
    }

    pub fn new_no_user_info_tracker(
        locker_id: u64,
        lock_owner: TestKeypair,
        amm_id: Pubkey,
        total_locker_count_for_user: u64,
        withdraw_amount: u64,
        payer: TestKeypair,
        lp_mint: Pubkey,
    ) -> Self {
        let derived_accounts = CreateAndLockLpInstructionDerivedAccounts::new(
            amm_id,
            lock_owner.pubkey(),
            lock_owner.pubkey(),
            total_locker_count_for_user,
            locker_id,
        );

        let user_lp_token_acc = get_ata_account(&lock_owner.pubkey(), &lp_mint);

        Self {
            lock_owner,
            locker_id,
            withdraw_amount,
            amm_id,
            payer,
            user_lp_token_acc,
            global_lp_marker_acc: derived_accounts.global_lp_marker_acc,
            lp_locker_acc: derived_accounts.lp_locker_acc,
            user_info_acc: None,
            user_info_lp_tracker_acc: None,
            uncx_lock_lp_vault_acc: derived_accounts.uncx_lock_lp_vault_acc,
            uncx_authority: derived_accounts.uncx_authority,
        }
    }
}

#[async_trait::async_trait(?Send)]

impl ClientInstruction for WithdrawLpFromLockerInstruction {
    type Accounts = uncx_solana_lp_locker::accounts::WithdrawLp;

    type Instruction = uncx_solana_lp_locker::instruction::WithdrawLp;

    async fn to_instruction(
        &self,
        _account_loader: impl ClientAccountLoader,
    ) -> (Self::Accounts, instruction::Instruction) {
        let instruction = uncx_solana_lp_locker::instruction::WithdrawLp {
            withdraw_amount: self.withdraw_amount,
            locker_id: self.locker_id,
        };

        let accounts = uncx_solana_lp_locker::accounts::WithdrawLp {
            lp_locker_acc: self.lp_locker_acc,
            lock_owner: self.lock_owner.pubkey(),
            uncx_authority_acc: UncxCoreProgramAccounts::get_uncx_authority_acc(),
            uncx_lock_lp_vault_acc: self.uncx_lock_lp_vault_acc,
            config_account: get_config_account_address(),
            user_lp_token_acc: self.user_lp_token_acc,
            token_program: anchor_spl::token::ID,
            user_info_acc: self.user_info_acc.into(),
            user_info_lp_tracker_acc: self.user_info_lp_tracker_acc.into(),
        };

        let instruction = make_instruction(UNCX_PROGRAM, &accounts, instruction);

        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        let mut signer_vec: Vec<TestKeypair> = Vec::with_capacity(10);

        signer_vec.push(self.lock_owner);

        signer_vec
    }
}

#[derive(Clone, Copy)]

pub struct MigrateLpInstruction {
    pub lock_owner: TestKeypair,
    pub migrate_amount: u64,
    pub locker_id: u64,
    pub migrator_lp_token_acc: Pubkey,
    pub amm_id: Pubkey,
    pub global_lp_marker_acc: Pubkey,
    pub lp_locker_acc: Pubkey,
    pub user_info_acc: OptionalKey,
    pub user_info_lp_tracker_acc: OptionalKey,
    pub uncx_lock_lp_vault_acc: Pubkey,
    pub uncx_authority: Pubkey,
    pub migrator_authority: TestKeypair,
    pub payer: TestKeypair,
}

impl MigrateLpInstruction {
    pub fn new(
        locker_id: u64,
        lock_owner: TestKeypair,
        amm_id: Pubkey,
        total_locker_count_for_user: u64,
        migrate_amount: u64,
        payer: TestKeypair,
        lp_mint: Pubkey,
        migrator_authority: TestKeypair,
    ) -> Self {
        let derived_accounts = CreateAndLockLpInstructionDerivedAccounts::new(
            amm_id,
            lock_owner.pubkey(),
            lock_owner.pubkey(),
            total_locker_count_for_user,
            locker_id,
        );

        let migrator_lp_token_acc = get_ata_account(&migrator_authority.pubkey(), &lp_mint);

        Self {
            lock_owner,
            locker_id,
            migrate_amount,
            amm_id,
            payer,
            migrator_lp_token_acc,
            global_lp_marker_acc: derived_accounts.global_lp_marker_acc,
            lp_locker_acc: derived_accounts.lp_locker_acc,
            user_info_acc: derived_accounts.user_info_acc.into(),
            user_info_lp_tracker_acc: derived_accounts.user_info_lp_tracker_acc.into(),
            uncx_lock_lp_vault_acc: derived_accounts.uncx_lock_lp_vault_acc,
            uncx_authority: derived_accounts.uncx_authority,
            migrator_authority,
        }
    }

    //function to check for cases where user info is not going to be modified.
    pub fn new_no_user_info_tracker(
        locker_id: u64,
        lock_owner: TestKeypair,
        amm_id: Pubkey,
        total_locker_count_for_user: u64,
        migrate_amount: u64,
        payer: TestKeypair,
        lp_mint: Pubkey,
        migrator_authority: TestKeypair,
    ) -> Self {
        let derived_accounts = CreateAndLockLpInstructionDerivedAccounts::new(
            amm_id,
            lock_owner.pubkey(),
            lock_owner.pubkey(),
            total_locker_count_for_user,
            locker_id,
        );

        let migrator_lp_token_acc = get_ata_account(&migrator_authority.pubkey(), &lp_mint);

        Self {
            lock_owner,
            locker_id,
            migrate_amount,
            amm_id,
            payer,
            migrator_lp_token_acc,
            global_lp_marker_acc: derived_accounts.global_lp_marker_acc,
            lp_locker_acc: derived_accounts.lp_locker_acc,
            user_info_acc: None,
            user_info_lp_tracker_acc: None,
            uncx_lock_lp_vault_acc: derived_accounts.uncx_lock_lp_vault_acc,
            uncx_authority: derived_accounts.uncx_authority,
            migrator_authority,
        }
    }
}

#[async_trait::async_trait(?Send)]

impl ClientInstruction for MigrateLpInstruction {
    type Accounts = uncx_solana_lp_locker::accounts::MigrateLp;

    type Instruction = uncx_solana_lp_locker::instruction::MigrateLp;

    async fn to_instruction(
        &self,
        _account_loader: impl ClientAccountLoader,
    ) -> (Self::Accounts, instruction::Instruction) {
        let instruction = uncx_solana_lp_locker::instruction::MigrateLp {
            locker_id: self.locker_id,
            migrate_amount: self.migrate_amount,
            migration_option: 0,
        };

        let migrator_marker_acc = Pubkey::find_program_address(
            &[
                MIGRATOR_SEED,
                self.migrator_authority.pubkey().key().as_ref(),
            ],
            &UNCX_PROGRAM,
        )
        .0;

        let accounts = uncx_solana_lp_locker::accounts::MigrateLp {
            lp_locker_acc: self.lp_locker_acc,
            lock_owner: self.lock_owner.pubkey(),
            uncx_authority_acc: UncxCoreProgramAccounts::get_uncx_authority_acc(),
            uncx_lock_lp_vault_acc: self.uncx_lock_lp_vault_acc,
            config_account: get_config_account_address(),
            migrator_token_lp_account: self.migrator_lp_token_acc,
            token_program: anchor_spl::token::ID,
            user_info_acc: self.user_info_acc.into(),
            user_info_lp_tracker_acc: self.user_info_lp_tracker_acc.into(),
            whitelisted_migrator_authority: self.migrator_authority.pubkey(),
            migrator_marker_acc,
        };

        let instruction = make_instruction(UNCX_PROGRAM, &accounts, instruction);

        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        let mut signer_vec: Vec<TestKeypair> = Vec::with_capacity(10);

        signer_vec.push(self.lock_owner);

        signer_vec.push(self.migrator_authority);

        signer_vec
    }
}

pub struct TransferOwnerShipLockerInstruction {
    pub locker_pda_acc: Pubkey,
    pub lock_owner: TestKeypair,

    pub locker_id: u64,
    pub user_lp_token_acc: Pubkey,

    pub new_owner: Pubkey,
    pub amm_id: Pubkey,
    pub old_user_locker_count: u64,
    pub new_user_locker_count: u64,

    pub payer: TestKeypair,
}

impl TransferOwnerShipLockerInstruction {
    pub fn new(
        locker_id: u64,
        lock_owner: TestKeypair,
        new_owner: Pubkey,
        amm_id: Pubkey,
        old_owner_locker_count: u64,
        payer: TestKeypair,
        lp_mint: Pubkey,
        new_user_locker_count: u64,
    ) -> Self {
        let derived_accounts = CreateAndLockLpInstructionDerivedAccounts::new(
            amm_id,
            lock_owner.pubkey(),
            lock_owner.pubkey(),
            old_owner_locker_count,
            locker_id,
        );

        let user_lp_token_acc = get_ata_account(&lock_owner.pubkey(), &lp_mint);

        Self {
            lock_owner,
            locker_id,
            amm_id,
            payer,
            user_lp_token_acc,
            old_user_locker_count: old_owner_locker_count,
            locker_pda_acc: derived_accounts.lp_locker_acc,
            new_owner,
            new_user_locker_count,
        }
    }

    pub fn new_no_user_info_tracker(
        locker_id: u64,
        lock_owner: TestKeypair,
        new_owner: Pubkey,
        amm_id: Pubkey,
        old_owner_locker_count: u64,
        payer: TestKeypair,
        lp_mint: Pubkey,
        new_user_locker_count: u64,
    ) -> Self {
        let derived_accounts = CreateAndLockLpInstructionDerivedAccounts::new(
            amm_id,
            lock_owner.pubkey(),
            lock_owner.pubkey(),
            old_owner_locker_count,
            locker_id,
        );

        let user_lp_token_acc = get_ata_account(&lock_owner.pubkey(), &lp_mint);

        Self {
            lock_owner,
            locker_id,
            amm_id,
            payer,
            user_lp_token_acc,
            old_user_locker_count: old_owner_locker_count,
            locker_pda_acc: derived_accounts.lp_locker_acc,
            new_owner,
            new_user_locker_count,
        }
    }
}

#[async_trait::async_trait(?Send)]

impl ClientInstruction for TransferOwnerShipLockerInstruction {
    type Accounts = uncx_solana_lp_locker::accounts::TransferLockOwnership;

    type Instruction = uncx_solana_lp_locker::instruction::TransferLockOwnership;

    async fn to_instruction(
        &self,
        _account_loader: impl ClientAccountLoader,
    ) -> (Self::Accounts, instruction::Instruction) {
        let instruction = uncx_solana_lp_locker::instruction::TransferLockOwnership {
            new_owner: self.new_owner,
            locker_id: self.locker_id,
        };

        let new_derived_accounts = CreateAndLockLpInstructionDerivedAccounts::new(
            self.amm_id,
            self.new_owner,
            self.new_owner,
            self.new_user_locker_count,
            self.locker_id + 1,
        );

        let old_derived_accounts = CreateAndLockLpInstructionDerivedAccounts::new(
            self.amm_id,
            self.lock_owner.pubkey(),
            self.lock_owner.pubkey(),
            self.old_user_locker_count,
            self.locker_id,
        );

        let accounts = uncx_solana_lp_locker::accounts::TransferLockOwnership {
            lp_locker_acc: old_derived_accounts.lp_locker_acc,
            lock_owner: self.lock_owner.pubkey(),
            payer: self.payer.pubkey(),
            system_program: anchor_lang::system_program::ID,
            old_user_info_acc: old_derived_accounts.user_info_acc,
            old_user_info_lp_tracker_acc: old_derived_accounts.user_info_lp_tracker_acc,
            new_user_info_acc: new_derived_accounts.user_info_acc,
            new_user_info_lp_tracker_acc: new_derived_accounts.user_info_lp_tracker_acc,
        };

        let instruction = make_instruction(UNCX_PROGRAM, &accounts, instruction);

        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        let mut signer_vec: Vec<TestKeypair> = Vec::with_capacity(10);

        signer_vec.push(self.payer);

        signer_vec.push(self.lock_owner);

        signer_vec
    }
}

#[derive(Clone, Copy)]

pub struct SplitLockInstruction {
    pub locker_pda_acc: Pubkey,
    pub lock_owner: TestKeypair,
    pub new_lock_amount: u64,

    pub old_locker_id: u64,
    pub new_locker_id: u64,
    pub user_lp_token_acc: Pubkey,
    pub user_info_acc: OptionalKey,
    pub user_lp_tracker_acc: OptionalKey,
    pub amm_id: Pubkey,
    pub user_locker_count: u64,
    pub dev_wallet: Pubkey,
    pub payer: TestKeypair,
}

impl SplitLockInstruction {
    pub fn new(
        locker_id: u64,
        lock_owner: TestKeypair,
        new_lock_amount: u64,
        amm_id: Pubkey,
        user_locker_count: u64,
        payer: TestKeypair,
        lp_mint: Pubkey,
        dev_wallet: Pubkey,
    ) -> Self {
        let derived_accounts = CreateAndLockLpInstructionDerivedAccounts::new(
            amm_id,
            lock_owner.pubkey(),
            lock_owner.pubkey(),
            user_locker_count,
            locker_id,
        );

        let user_lp_token_acc = get_ata_account(&lock_owner.pubkey(), &lp_mint);

        Self {
            lock_owner,
            old_locker_id: locker_id,
            new_locker_id: locker_id + 1,
            amm_id,
            payer,
            user_lp_token_acc,
            locker_pda_acc: derived_accounts.lp_locker_acc,
            new_lock_amount,
            user_info_acc: derived_accounts.user_info_acc.into(),
            user_lp_tracker_acc: derived_accounts.user_info_lp_tracker_acc.into(),
            user_locker_count,
            dev_wallet,
        }
    }
}

#[async_trait::async_trait(?Send)]

impl ClientInstruction for SplitLockInstruction {
    type Accounts = uncx_solana_lp_locker::accounts::SplitLock;

    type Instruction = uncx_solana_lp_locker::instruction::SplitRelockLp;

    async fn to_instruction(
        &self,
        _account_loader: impl ClientAccountLoader,
    ) -> (Self::Accounts, instruction::Instruction) {
        let instruction = uncx_solana_lp_locker::instruction::SplitRelockLp {
            new_locker_locked_amount: self.new_lock_amount,
            old_locker_id: self.old_locker_id,
        };

        let derived_accounts = CreateAndLockLpInstructionDerivedAccounts::new(
            self.amm_id,
            self.lock_owner.pubkey(),
            self.lock_owner.pubkey(),
            self.user_locker_count,
            self.old_locker_id,
        );

        let accounts = uncx_solana_lp_locker::accounts::SplitLock {
            lp_locker_acc: derived_accounts.lp_locker_acc,
            lock_owner: self.lock_owner.pubkey(),
            payer: self.payer.pubkey(),
            system_program: anchor_lang::system_program::ID,
            new_lp_locker_acc: get_locker_pda_acc(self.new_locker_id),
            config_account: get_config_account_address(),
            dev_wallet: self.dev_wallet,
            user_info_acc: derived_accounts.user_info_acc,
            user_info_lp_tracker_acc: derived_accounts.user_info_lp_tracker_acc,
        };

        let instruction = make_instruction(UNCX_PROGRAM, &accounts, instruction);

        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        let mut signer_vec: Vec<TestKeypair> = Vec::with_capacity(10);

        signer_vec.push(self.payer);

        signer_vec.push(self.lock_owner);

        signer_vec
    }
}

pub struct RelockInstruction {
    pub locker_pda_acc: Pubkey,
    pub lock_owner: TestKeypair,
    pub lp_vault_account: Pubkey,

    pub locker_id: u64,
    pub new_unlock_date: i64,

    pub amm_id: Pubkey,

    pub dev_wallet: Pubkey,
    pub dev_lp_token_acc: Pubkey,
    pub payer: TestKeypair,
}

impl RelockInstruction {
    pub fn new(
        locker_id: u64,
        lock_owner: TestKeypair,

        amm_id: Pubkey,
        payer: TestKeypair,
        lp_mint: Pubkey,
        dev_wallet: Pubkey,
        new_unlock_date: i64,
    ) -> Self {
        let uncx_lp_vault_acc =
            CreateAndLockLpInstructionDerivedAccounts::get_lp_locker_token_acc(amm_id);

        let dev_lp_token_acc = get_ata_account(&dev_wallet, &lp_mint);

        Self {
            lock_owner,
            locker_id,
            amm_id,
            payer,
            locker_pda_acc: get_locker_pda_acc(locker_id),
            dev_wallet,
            dev_lp_token_acc,
            new_unlock_date,
            lp_vault_account: uncx_lp_vault_acc,
        }
    }
}

#[async_trait::async_trait(?Send)]

impl ClientInstruction for RelockInstruction {
    type Accounts = uncx_solana_lp_locker::accounts::RelockLp;

    type Instruction = uncx_solana_lp_locker::instruction::RelockLp;

    async fn to_instruction(
        &self,
        _account_loader: impl ClientAccountLoader,
    ) -> (Self::Accounts, instruction::Instruction) {
        let instruction = uncx_solana_lp_locker::instruction::RelockLp {
            lock_id: self.locker_id,
            new_unlock_date: self.new_unlock_date,
        };

        let accounts = uncx_solana_lp_locker::accounts::RelockLp {
            lp_locker_acc: self.locker_pda_acc,
            lock_owner: self.lock_owner.pubkey(),
            uncx_authority_acc: UncxCoreProgramAccounts::get_uncx_authority_acc(),
            uncx_lock_lp_vault_acc: self.lp_vault_account,
            dev_lp_token_account: self.dev_lp_token_acc,
            config_account: get_config_account_address(),
            token_program: anchor_spl::token::ID,
        };

        let instruction = make_instruction(UNCX_PROGRAM, &accounts, instruction);

        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        let mut signer_vec: Vec<TestKeypair> = Vec::with_capacity(10);

        signer_vec.push(self.lock_owner);

        signer_vec
    }
}

#[derive(Clone, Copy)]

pub struct AdminIxChangeAdmin {
    pub new_admin_key: Pubkey,

    pub admin: TestKeypair,
}

#[async_trait::async_trait(?Send)]

impl ClientInstruction for AdminIxChangeAdmin {
    type Accounts = uncx_solana_lp_locker::accounts::AdminIx;

    type Instruction = uncx_solana_lp_locker::instruction::SetDev;

    async fn to_instruction(
        &self,
        _account_loader: impl ClientAccountLoader,
    ) -> (Self::Accounts, instruction::Instruction) {
        let instruction = uncx_solana_lp_locker::instruction::SetNewAdmin {
            new_admin_addr: self.new_admin_key,
        };

        let config_account_address = UncxCoreProgramAccounts::get_config_address();

        let accounts = uncx_solana_lp_locker::accounts::AdminIx {
            config_account: config_account_address,

            admin_sign: self.admin.pubkey(),
        };

        let instruction = make_instruction(UNCX_PROGRAM, &accounts, instruction);

        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.admin]
    }
}

#[derive(Clone, Copy)]

pub struct AdminIxSetDevAddr {
    pub new_dev_addr: Pubkey,

    pub admin: TestKeypair,
}

#[async_trait::async_trait(?Send)]

impl ClientInstruction for AdminIxSetDevAddr {
    type Accounts = uncx_solana_lp_locker::accounts::AdminIx;

    type Instruction = uncx_solana_lp_locker::instruction::SetDev;

    async fn to_instruction(
        &self,
        _account_loader: impl ClientAccountLoader,
    ) -> (Self::Accounts, instruction::Instruction) {
        let instruction = uncx_solana_lp_locker::instruction::SetDev {
            new_dev_addr: self.new_dev_addr,
        };

        let config_account_address = UncxCoreProgramAccounts::get_config_address();

        let accounts = uncx_solana_lp_locker::accounts::AdminIx {
            config_account: config_account_address,

            admin_sign: self.admin.pubkey(),
        };

        let instruction = make_instruction(UNCX_PROGRAM, &accounts, instruction);

        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.admin]
    }
}

#[derive(Clone, Copy)]

pub struct AdminIxSetFeeConfig {
    pub new_config: FeesConfig,

    pub admin: TestKeypair,
}

#[async_trait::async_trait(?Send)]

impl ClientInstruction for AdminIxSetFeeConfig {
    type Accounts = uncx_solana_lp_locker::accounts::AdminIx;

    type Instruction = uncx_solana_lp_locker::instruction::SetNewFeesConfig;

    async fn to_instruction(
        &self,
        _account_loader: impl ClientAccountLoader,
    ) -> (Self::Accounts, instruction::Instruction) {
        let instruction = uncx_solana_lp_locker::instruction::SetNewFeesConfig {
            new_fees_config: self.new_config,
        };

        let config_account_address = UncxCoreProgramAccounts::get_config_address();

        let accounts = uncx_solana_lp_locker::accounts::AdminIx {
            config_account: config_account_address,

            admin_sign: self.admin.pubkey(),
        };

        let instruction = make_instruction(UNCX_PROGRAM, &accounts, instruction);

        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.admin]
    }
}

#[derive(Clone, Copy)]

pub struct AdminIxSetReferralAndBalance {
    pub new_referral_token_address: Option<Pubkey>,
    pub new_referral_balance: u64,

    pub admin: TestKeypair,
}

#[async_trait::async_trait(?Send)]

impl ClientInstruction for AdminIxSetReferralAndBalance {
    type Accounts = uncx_solana_lp_locker::accounts::AdminIx;

    type Instruction = uncx_solana_lp_locker::instruction::SetReferralTokenHoldBalance;

    async fn to_instruction(
        &self,
        _account_loader: impl ClientAccountLoader,
    ) -> (Self::Accounts, instruction::Instruction) {
        let instruction = uncx_solana_lp_locker::instruction::SetReferralTokenHoldBalance {
            new_referral_token: self.new_referral_token_address,
            new_referral_token_hold_balance: self.new_referral_balance,
        };

        let config_account_address = UncxCoreProgramAccounts::get_config_address();

        let accounts = uncx_solana_lp_locker::accounts::AdminIx {
            config_account: config_account_address,

            admin_sign: self.admin.pubkey(),
        };

        let instruction = make_instruction(UNCX_PROGRAM, &accounts, instruction);

        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.admin]
    }
}

#[derive(Clone, Copy)]

pub struct AdminIxSetSecondaryToken {
    pub new_secondary_token_address: OptionalKey,

    pub admin: TestKeypair,
}

#[async_trait::async_trait(?Send)]

impl ClientInstruction for AdminIxSetSecondaryToken {
    type Accounts = uncx_solana_lp_locker::accounts::AdminIx;

    type Instruction = uncx_solana_lp_locker::instruction::SetSecondaryToken;

    async fn to_instruction(
        &self,
        _account_loader: impl ClientAccountLoader,
    ) -> (Self::Accounts, instruction::Instruction) {
        let instruction = uncx_solana_lp_locker::instruction::SetSecondaryToken {
            new_secondary_token: self.new_secondary_token_address,
        };

        let config_account_address = UncxCoreProgramAccounts::get_config_address();

        let accounts = uncx_solana_lp_locker::accounts::AdminIx {
            config_account: config_account_address,

            admin_sign: self.admin.pubkey(),
        };

        let instruction = make_instruction(UNCX_PROGRAM, &accounts, instruction);

        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.admin]
    }
}

#[derive(Clone, Copy)]

pub struct AdminIxAddCountryToBlacklist {
    pub country_to_add_to_black_list: u8,

    pub admin: TestKeypair,
}

#[async_trait::async_trait(?Send)]

impl ClientInstruction for AdminIxAddCountryToBlacklist {
    type Accounts = uncx_solana_lp_locker::accounts::AdminIx;

    type Instruction = uncx_solana_lp_locker::instruction::SetNewFeesConfig;

    async fn to_instruction(
        &self,
        _account_loader: impl ClientAccountLoader,
    ) -> (Self::Accounts, instruction::Instruction) {
        let instruction = uncx_solana_lp_locker::instruction::AddCountryToBlacklist {
            country_code_to_add: self.country_to_add_to_black_list,
        };

        let config_account_address = UncxCoreProgramAccounts::get_config_address();

        let accounts = uncx_solana_lp_locker::accounts::AdminIx {
            config_account: config_account_address,

            admin_sign: self.admin.pubkey(),
        };

        let instruction = make_instruction(UNCX_PROGRAM, &accounts, instruction);

        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.admin]
    }
}

#[derive(Clone, Copy)]

pub struct AdminIxRemoveFromBlackList {
    pub country_to_remove_from_blacklist: u8,

    pub admin: TestKeypair,
}

#[async_trait::async_trait(?Send)]

impl ClientInstruction for AdminIxRemoveFromBlackList {
    type Accounts = uncx_solana_lp_locker::accounts::AdminIx;

    type Instruction = uncx_solana_lp_locker::instruction::SetNewFeesConfig;

    async fn to_instruction(
        &self,
        _account_loader: impl ClientAccountLoader,
    ) -> (Self::Accounts, instruction::Instruction) {
        let instruction = uncx_solana_lp_locker::instruction::RemoveCountryFromBlacklist {
            country_code_to_remove: self.country_to_remove_from_blacklist,
        };

        let config_account_address = UncxCoreProgramAccounts::get_config_address();

        let accounts = uncx_solana_lp_locker::accounts::AdminIx {
            config_account: config_account_address,

            admin_sign: self.admin.pubkey(),
        };

        let instruction = make_instruction(UNCX_PROGRAM, &accounts, instruction);

        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.admin]
    }
}
