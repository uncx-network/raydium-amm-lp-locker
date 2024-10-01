use anchor_lang::prelude::*;

pub const MAX_BASIS_POINTS: u16 = 10000; //100 %
pub const DISCRIMINATOR_BYTES_SIZE: usize = 8;

#[constant]

pub const WHITELIST_ACC_STATIC_SEED: &[u8] = b"uncx-whitelist";

pub const MIGRATOR_SEED: &[u8] = b"MIGRATOR";

//place holder change it to some initial pubkey while deploying
pub const INITIAL_ADMIN: Pubkey =
    solana_program::pubkey!("Fa8RNztZiKS1u17N5vtZ4goMBH6jHkRWVEC7ifP7aN6Y");
