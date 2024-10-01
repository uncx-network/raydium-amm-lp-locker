mod calc_reserves;
mod error;
mod math;
mod state;
use crate::raydium_amm;

use super::Pubkey;

/// Suffix for amm authority seed

pub const AMM_ASSOCIATED_SEED: &[u8] = b"amm_associated_seed";

/// Suffix for lp mint associated seed

pub const LP_MINT_ASSOCIATED_SEED: &[u8] = b"lp_mint_associated_seed";

//for lp mint,
//program id is raydium amm id
//market_account is opendex/serum market account key,
//associated seed is - >
//program id is again raydium id
//associated token address is actually the mint

pub fn get_associated_address_and_bump_seed(
    info_id: &Pubkey,
    market_address: &Pubkey,
    associated_seed: &[u8],
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            &info_id.to_bytes(),
            &market_address.to_bytes(),
            associated_seed,
        ],
        program_id,
    )
}

pub fn get_raydium_amm_info_key(market_address: &Pubkey) -> Pubkey {
    let (amm_info_key, _) = get_associated_address_and_bump_seed(
        &raydium_amm::id(),
        market_address,
        AMM_ASSOCIATED_SEED,
        &raydium_amm::id(),
    );

    amm_info_key
}

pub fn get_raydium_amm_lp_mint_key(market_address: &Pubkey) -> Pubkey {
    let (amm_info_key, _) = get_associated_address_and_bump_seed(
        &raydium_amm::id(),
        market_address,
        LP_MINT_ASSOCIATED_SEED,
        &raydium_amm::id(),
    );

    amm_info_key
}

pub use calc_reserves::*;
pub use error::*;
pub use state::*;
