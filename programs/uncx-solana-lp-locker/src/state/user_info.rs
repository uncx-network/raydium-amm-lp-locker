//! UserInfo is modelled as follows, the Seed :[`user's wallet address`,`ammid or the lp mint`] are
//! the core seeds of all user info specific accounts

//! UserInfo will comprise of multiple accounts to track user-data onchain

//! -each unique lp will be its own account containing a counter(Z), tracking the number of lockers
//! associated with the uniquelp

//! a second account derived by the core-seeds above with the addition of the `Z` will track the
//! token ids associated with the mint

//! the value of Z will not be used in its raw form but the `Z/MaxLpTrackPerAccountLimit` will be
//! the value used, such as for each specific LpMint per user once the user has associated lockers
//! greater than `MaxLpTrackPerAccountLimit` a new account will be used to store the newly
//! associated token ids. so every MaxLpTrackPerAccountLimit a new account is created which a high
//! enough number ensuring the scenario for there to exist more than one account storing all
//! associated tokenids with a mint infrequent.

//!We map the set of all unique lp addresses by having an onchain pda for each unique lp account
//! for each user and using gpa rpc call to retrieve all lp accounts specific to each user by using
//! the 2nd and 3rd fields as input to the memcpy filter in the gpa rpc call.

use super::*;

#[constant]

pub const MAX_LP_TRACK_PER_ACCOUNT: u16 = 15;

///Accounts
// Account#1 - Seeds[(core seeds),b"user_info"]
#[account]
#[derive(InitSpace)]

pub struct UserInfoAccount {
    pub(crate) bump: u8,
    //field used for offchain indexing via gpa
    pub(crate) user: Pubkey,
    //Address of the associated lp mint
    //used to aid in offchain filtering of accounts via GPA rpc.
    pub(crate) lp_mint: Pubkey,
    //stores total count of lp lockers associated with a specific amm id
    pub lp_locker_count: u64,
}

impl UserInfoAccount {
    pub(crate) fn next_user_lp_acc_index(&self) -> [u8; std::mem::size_of::<u64>()] {
        (self.lp_locker_count / MAX_LP_TRACK_PER_ACCOUNT as u64).to_le_bytes()
    }
}

//Account#2 - Seeds[(core seeds),b"user_lp_tracker","count(number of lockers with lpmint) (divided
// by) MaxLpTrackPerAccountLimit"] Stores Vec of locker id's associated with the mint
#[account]
#[derive(InitSpace)]

pub struct UserLpInfoAccount {
    pub(crate) bump: u8,
    //Growable List of locker ids associated with a specific lp mint/ amm id used to derive the
    // address of this pda
    #[max_len(MAX_LP_TRACK_PER_ACCOUNT)]
    pub(crate) associated_locker_ids: Vec<u64>,
}

impl UserLpInfoAccount {
    pub fn space(len: usize) -> usize {
        //8 Discriminator + 1 (Bump) + 4 (borsh serializes size of the dynamic list as a u32 and
        // prefixes it.)
        8 + 1 + (4 + (len * std::mem::size_of::<u64>()))
    }

    pub fn find_and_remove_locker_from_tracking(&mut self, locker_id_to_remove: u64) -> Result<()> {
        let locker_id_remove_index = self
            .associated_locker_ids
            .iter()
            .position(|locker_id| *locker_id == locker_id_to_remove)
            .ok_or(UncxLpError::MissingLockerId)?;

        self.associated_locker_ids.remove(locker_id_remove_index);

        Ok(())
    }
}
