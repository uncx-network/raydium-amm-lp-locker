use super::*;

mod add_migrator;
mod add_whitelist;
mod change_owner;
mod remove_migrator;
mod remove_whitelist;
mod set_dev;
mod set_fees;
mod set_referral_token_hold;
mod set_secondary_token;
mod blacklist_management;
pub use add_migrator::*;
pub use add_whitelist::*;
pub use change_owner::*;
pub use remove_migrator::*;
pub use remove_whitelist::*;
pub use set_dev::*;
pub use set_fees::*;
pub use set_referral_token_hold::*;
pub use set_secondary_token::*;
pub use blacklist_management::*;
