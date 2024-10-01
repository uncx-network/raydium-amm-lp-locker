//!This module represents all possible events and their structure emitted in repsonse to the
//! different set of actions invoked on the contract.
use anchor_lang::prelude::*;
use borsh::BorshSerialize;
use std::borrow::Cow;
#[inline(never)] // ensure fresh stack frame
pub fn emit_stack<T: anchor_lang::Event>(e: T) {
    use std::io::{Cursor, Write};

    // stack buffer, stack frames are 4kb
    let mut buffer = [0u8; 3000];

    let mut cursor = Cursor::new(&mut buffer[..]);

    cursor.write_all(&T::DISCRIMINATOR).unwrap();

    e.serialize(&mut cursor)
        .expect("event must fit into stack buffer");

    //get the index that stores the last byte relating to the event
    let pos = cursor.position() as usize;

    anchor_lang::solana_program::log::sol_log_data(&[&buffer[..pos]]);
}
//Used CoW instead of &str due to the latter not impl Anchor(Se)Deserialize traits.
// #[derive(anchor_lang::__private::EventIndex, AnchorSerialize, AnchorDeserialize)]
//NOTE: we avoid AnchorSerialize as it fails to compile with 'CoW' we use 'BorshSerialize' instead.
#[derive(anchor_lang::__private::EventIndex, BorshSerialize, AnchorDeserialize)]
pub struct OnNewLock<'a> {
    pub lock_id: u64,
    pub amm_id: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub lock_date: i64,
    pub unlock_date: i64,
    pub country_code: u8,
    pub pc_token_name: Cow<'a, str>,
    pub pc_token_symbol: Cow<'a, str>,
    pub pc_token_decimals: u8,
    pub pc_mint: Pubkey,
    pub coin_token_name: Cow<'a, str>,
    pub coin_token_symbol: Cow<'a, str>,
    pub coin_token_decimals: u8,
    pub coin_mint: Pubkey,
    pub amm_real_liquidity: u64,
    pub amm_real_pc_reserve: u64,
    pub amm_real_coin_reserve: u64,
}
impl<'a> anchor_lang::Event for OnNewLock<'a> {
    fn data(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(256);
        data.extend_from_slice(&[98, 218, 121, 247, 127, 230, 232, 168]);
        self.serialize(&mut data).unwrap();
        data
    }
}
impl<'a> anchor_lang::Discriminator for OnNewLock<'a> {
    const DISCRIMINATOR: [u8; 8] = [98, 218, 121, 247, 127, 230, 232, 168];
}

#[event]

pub struct OnRelock {
    pub lock_id: u64,
    pub lp_token: Pubkey,
    pub owner: Pubkey,
    pub amount_remaining_in_lock: u64,
    pub liquidity_fee: u64,
    pub unlock_date: i64,
}

#[event]

pub struct OnWithdraw {
    pub lock_id: u64,
    pub lp_token: Pubkey,
    pub owner: Pubkey,
    pub amount_remaining_in_lock: u64,
    pub amount_removed: u64,
}

#[event]

pub struct OnIncrementLock {
    pub lock_id: u64,
    pub lp_token: Pubkey,
    pub owner: Pubkey,
    pub entity: Pubkey,
    pub amount_remaining_in_lock: u64,
    pub amount_added: u64,
    pub liquidity_fee: u64,
}

#[event]

pub struct OnSplitLock {
    pub lock_id: u64,
    pub old_lock_id: u64,
    pub lp_token: Pubkey,
    pub owner: Pubkey,
    pub amount_remaining_in_old_lock: u64,
    pub amount_removed: u64,
    pub unlock_date: i64,
}

#[event]

pub struct OnTransferLockOwnership {
    pub lock_id: u64,
    pub lp_token: Pubkey,
    pub old_owner: Pubkey,
    pub new_owner: Pubkey,
}

#[event]

pub struct OnMigrate {
    pub lock_id: u64,
    pub lp_token: Pubkey,
    pub owner: Pubkey,
    pub amount_remaining_in_lock: u64,
    pub amount_migrated: u64,
    pub migration_option: u16,
}
