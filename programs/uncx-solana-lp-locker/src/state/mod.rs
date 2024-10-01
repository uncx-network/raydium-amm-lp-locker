//! `state.rs` This module hold the Data Structures/Models of the accounts used, as well as holds
//! the instruction level accounts Data Models are as follows : `user_info`,
//! `token_lock`,`settings`,`onchain_indexed_data`

use super::*;

mod configuration;
mod events;
mod token_lock;
mod user_info;

// mod zerocopy_bigvec;
use anchor_lang::prelude::*;
pub use configuration::*;
pub use events::*;
pub use token_lock::*;
pub use user_info::*;
