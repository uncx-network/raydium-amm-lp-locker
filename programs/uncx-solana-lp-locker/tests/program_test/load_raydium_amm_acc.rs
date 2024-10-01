// use super::market_keys_utils::get_keys_for_market;
use super::*;
use anyhow::Result;
// use market_keys_utils::MarketPubkeys;
use raydium_port::AmmInfo;
use raydium_port::Loadable;
use solana_account_decoder::UiAccountData;
use solana_account_decoder::UiAccountEncoding;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::account::ReadableAccount;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[derive(Debug, Clone)]

pub struct RaydiumTestFixtureBuilder {
    amm_id: Pubkey,
}
#[derive(Default, Clone, Copy)]
pub struct RaydiumTestFixture {
    pub(crate) amm_info: raydium_port::AmmInfo,
    // pub(crate) market_keys: MarketPubkeys,
    pub(crate) pc_mint_metadata: Pubkey,
    pub(crate) coin_mint_metadata: Pubkey,
}
impl RaydiumTestFixtureBuilder {
    pub const fn new(amm_id: Pubkey) -> Self {
        Self { amm_id }
    }
    pub async fn build(
        self,
        client: &RpcClient,
        test_env: &mut ProgramTest,
    ) -> Result<RaydiumTestFixture> {
        let amm_info_acc = client.get_account(&self.amm_id).await?;
        let amm_info_ret = AmmInfo::load_from_bytes(amm_info_acc.data())?;
        let pc_mint_metadata = find_metadata_v3_acc(amm_info_ret.pc_vault_mint);
        let coin_mint_metadata = find_metadata_v3_acc(amm_info_ret.coin_vault_mint);

        //     println!("coin token metadata acc {}, mint {},pc metadata_acc {}, mint {}",
        //     coin_mint_metadata,
        //     amm_info_ret.coin_vault_mint,
        //     pc_mint_metadata,
        //     amm_info_ret.pc_vault_mint
        // );
        let raydium_addresses = [
            self.amm_id,
            amm_info_ret.coin_vault_mint,
            amm_info_ret.pc_vault_mint,
            amm_info_ret.pc_vault,
            amm_info_ret.coin_vault,
            amm_info_ret.lp_mint,
            amm_info_ret.market,
            amm_info_ret.target_orders,
            amm_info_ret.open_orders,
            // // market_addresses.coin_vault,
            // // market_addresses.pc_vault,
            // // market_addresses.event_q,
            // // market_addresses.bids,
            // // market_addresses.asks,
            // // market_addresses.req_q,
            pc_mint_metadata,
            coin_mint_metadata,
        ];
        add_multiple_acc_to_env(&client, test_env, raydium_addresses.as_ref()).await?;
        Ok(RaydiumTestFixture {
            amm_info: *amm_info_ret,
            // market_keys: market_addresses,
            pc_mint_metadata,
            coin_mint_metadata,
        })
    }
}

pub async fn add_multiple_acc_to_env(
    client: &RpcClient,
    test_env: &mut ProgramTest,
    account_addresses: &[Pubkey],
) -> Result<()> {
    for account_address in account_addresses {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let acc = client.get_account(&account_address).await?;

        test_env.add_account(*account_address, acc);
    }
    Ok(())
}
pub fn write_account_to_file(
    acc: &solana_sdk::account::Account,
    key: Pubkey,
) -> anyhow::Result<()> {
    let ui_account = solana_account_decoder::UiAccount {
        lamports: acc.lamports,
        owner: acc.owner.to_string(),
        executable: acc.executable,
        space: Some(acc.data.len() as u64),
        rent_epoch: acc.rent_epoch,
        data: UiAccountData::Binary(base64::encode(acc.data()), UiAccountEncoding::Base64),
    };
    println!("writing acc to file");
    let serialized_data = serde_json::to_string_pretty(&ui_account)?;
    let file_name = key.to_string();
    let fixture_path = std::env::current_dir()
        .expect("fixture path current dir failed")
        .join(format!("tests/fixtures/{}", file_name));
    println!("path is {}", fixture_path.display());
    std::fs::write(fixture_path.clone(), serialized_data)
        .expect(format!("writing data failed for pubkey {}", key).as_str());
    println!("writing acc to file successful");
    let json_content = std::fs::read_to_string(&fixture_path)?;

    let account: solana_account_decoder::UiAccount =
        serde_json::from_str(&json_content).expect("Failed to deserialize JSON");
    let test_serialized_data = account.data.decode().ok_or(anyhow::anyhow!("failed"))?;
    assert_eq!(
        acc.data, test_serialized_data,
        "failed in checking written and read data is same"
    );
    Ok(())
}
pub fn parse_and_get_sdk_acc_from_ui_acc(
    file_name: &str,
) -> anyhow::Result<solana_sdk::account::Account> {
    // println!("reading from fixtures");
    let fixture_path = std::env::current_dir()
        .expect("fixture path current dir failed")
        .join(format!("tests/fixtures/{}", file_name));
    // println!("read json of acc");
    let json_content = std::fs::read_to_string(&fixture_path)?;

    let account: solana_account_decoder::UiAccount =
        serde_json::from_str(&json_content).expect("Failed to deserialize JSON");
    Ok(solana_sdk::account::Account::new_data(
        account.lamports,
        &account.data.decode().expect("shoudnt return nothing"),
        &Pubkey::from_str(account.owner.as_str()).expect("should not fail"),
    )?)
}
