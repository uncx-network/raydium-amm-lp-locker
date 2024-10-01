#![allow(dead_code)]

use anchor_lang::prelude::*;

use super::{
    client::*,
    send_tx,
    solana::SolanaCookie,
    MintCookie,
    TestKeypair,
    UserCookie,
};

#[derive(Clone)]

pub struct Token {
    pub index: u16,
    pub mint: MintCookie,
    pub mint_info: Pubkey,
}

impl Token {
    pub fn create(mints: Vec<MintCookie>) -> Vec<Token> {

        let mut tokens = vec![];

        for (index, mint) in mints.iter().enumerate() {

            let token_index = index as u16;

            tokens.push(Token {
                index: token_index,
                mint: *mint,
                mint_info: mint.pubkey,
            });
        }

        tokens
    }
}

// pub async fn create_open_orders_indexer(
//     solana: &SolanaCookie,
//     payer: &UserCookie,
//     owner: TestKeypair,
//     market: Pubkey,
// ) -> Pubkey {
//     send_tx(
//         solana,
//         CreateOpenOrdersIndexerInstruction {
//             market,
//             owner,
//             payer: payer.key,
//         },
//     )
//     .await
//     .unwrap()
//     .open_orders_indexer
// }

// pub async fn create_open_orders_account(
//     solana: &SolanaCookie,
//     owner: TestKeypair,
//     market: Pubkey,
//     account_num: u32,
//     payer: &UserCookie,
//     delegate: Option<Pubkey>,
// ) -> Pubkey {
//     send_tx(
//         solana,
//         CreateOpenOrdersAccountInstruction {
//             account_num,
//             market,
//             owner,
//             payer: payer.key,
//             delegate,
//         },
//     )
//     .await
//     .unwrap()
//     .open_orders_account
// }
