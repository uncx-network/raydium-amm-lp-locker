//!Maps LOCKS and TokensLocked on evm contracts.
//! TokenLock is modelled as follows, Seeds:["uncx_locker",`locker_id(unique```] id will be conv to
//! u8 as u64.to_le_bytes

use super::*;

//Account#1
//account is derived via the locker id, essentially mapping to its evm equivalent of  `LOCKS`
//and implicitly `TokenLocks`(via gpa by filtering for the first field : `lp_token`) i.e mapping(lp
// address=>[lock id associated])
#[account]
#[cfg_attr(feature = "client", derive(Debug))]
#[derive(InitSpace)]

pub struct TokenLock {
    pub(crate) bump: u8,
    /// The raydium amm pool id/amm pair lp mint address
    // keeping lp-token first for easier indexing via gpa.
    pub(crate) amm_id: Pubkey,
    pub(crate) lp_mint: Pubkey,
    /// incremental global locker_count value.
    //stored earlier for easier indexing
    pub(crate) lock_global_id: u64,
    ///the data the token was locked at ,stored in unix timestamp
    pub lock_date: i64,
    ///the data the token is unlocked at ,stored in unix timestamp
    pub unlock_date: i64,
    /// the country code of the locker/business
    pub(crate) country_code: u8,
    //amount of lp locked at the time of creating the lock
    pub(crate) initial_lock_amount: u64,
    //amount of lp locked at any specific time after the initial lock
    pub current_locked_amount: u64,
    /// who can withdraw the lp from the lock after specified duration is complete aka owner
    pub lock_owner: Pubkey,
}

///`GlobalLpMintMarker` discriminator is the filter - Once per unique AMM
/// seeds : [amm_address,"global_lp_tracker"]
#[account]
#[derive(InitSpace)]

pub struct GlobalLpMintMarker {
    pub(crate) bump: u8,
}
