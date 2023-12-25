use rand::seq::SliceRandom;
use solana_client::nonblocking::rpc_client::RpcClient;
use anchor_client::solana_client::rpc_config::RpcSendTransactionConfig;
use anchor_lang::system_program;
use bincode::serialize;

use anchor_client::solana_sdk::pubkey::Pubkey;

use anchor_client::solana_sdk::signature::{Keypair, Signer};
use anchor_client::{Cluster, Program};
use serde_json::json;
use solana_address_lookup_table_program::state::{AddressLookupTable};
use solana_client::rpc_request::RpcRequest;
use solana_program::instruction::AccountMeta;
use solana_program::message::{VersionedMessage, v0};
use solana_program::program_pack::Pack;
use solana_sdk::address_lookup_table_account::AddressLookupTableAccount;
use solana_sdk::commitment_config::{CommitmentLevel, CommitmentConfig};
use solana_sdk::signature::{read_keypair_file, Signature};
use solana_transaction_status::UiTransactionEncoding;
use std::collections::{HashMap, HashSet, BinaryHeap};
use std::pin::Pin;
use std::sync::Arc;

use solana_sdk::instruction::Instruction;
use solana_sdk::transaction::{Transaction, VersionedTransaction};

use std::borrow::{Borrow, BorrowMut};
use std::rc::Rc;
use std::str::FromStr;
use std::vec;

use log::info;

use tmp::accounts as tmp_accounts;
use tmp::instruction as tmp_ix;

use crate::constants::{ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID};
use crate::monitor::pools::pool::{PoolOperations, PoolType};

use crate::utils::{derive_token_address, PoolGraph, PoolIndex, PoolQuote, store_amount_in_redis};
#[derive(Clone)]
pub struct Arbitrager {
    pub token_mints: Vec<Pubkey>,
    pub graph_edges: Vec<HashSet<usize>>, // used for quick searching over the graph
    pub graph: PoolGraph,
    pub cluster: Cluster,
    // vv -- need to clone these explicitly -- vv
    pub connection: Arc<RpcClient>,
}
impl Arbitrager {
    pub async fn optimized_search(
        &mut self,
        start_mint_idx: usize,
        init_balance: u128,
        path: Vec<usize>,
    ) -> Result<Option<(u128, Vec<usize>, Vec<PoolQuote>)>, anyhow::Error> {
        loop {
    let mut best_profit = 0;
    let mut current_amount = init_balance;
    let mut best_path = Vec::new();
    let mut best_pool_path = Vec::new();

    let mut queue = BinaryHeap::new();
    queue.push((0, start_mint_idx, Vec::new(), Vec::new()));

    while let Some((profit, mint_idx, path, pool_path)) = queue.pop() {
        let profit = -(profit as i128);
        let out_edges = self.graph_edges[mint_idx].clone();
        for mut dst_mint_idx in out_edges {
           
            let pools = &mut self
                .graph
                .0
                .get(&PoolIndex(mint_idx))
                .unwrap()
                .0
                .get(&PoolIndex(dst_mint_idx))
                .unwrap();

            let mut pools = pools.clone();
            pools.shuffle(&mut rand::thread_rng());
            for pool in pools {
                // Get updated accounts for the pool
                /*
                let updated_accounts = pool.get_update_accounts();

                // Get the unpacked token balance for the pool
                let token_accounts = self.connection.get_multiple_accounts(&updated_accounts).await?;
                let ata2 = token_accounts.get(1).unwrap().clone();
                let ata1 = token_accounts.get(0).unwrap().clone();
                if ata1.is_none() || ata2.is_none() {
                    continue;
                }

                let token_amounts = [spl_token::state::Account::unpack(
                    &ata1.unwrap().data,
                )?.amount, spl_token::state::Account::unpack(&ata2.unwrap().data)?.amount];
                let pool_src_amt = token_amounts[0] as u128;
                let pool_dst_amt = token_amounts[1] as u128;
                */
                let mut profit = profit as u128;
                
                let new_balance = pool.get_quote_with_amounts_scaled(current_amount, &self.token_mints[mint_idx], &self.token_mints[dst_mint_idx]);

                if new_balance == 0 {
                    continue;
                }
                current_amount = new_balance;

                    let mut new_path = path.clone();
                    new_path.push(dst_mint_idx);

                    let mut new_pool_path = pool_path.clone();
                    new_pool_path.push(pool.clone());
                    profit = new_balance - init_balance;
                    if profit > best_profit {
                        best_profit = profit;
                        best_path = new_path.clone();
                        best_pool_path = new_pool_path.clone();
                    }
                    

                    if new_path.len() >= 3 && dst_mint_idx == start_mint_idx && new_balance > init_balance {
                        println!("best path: {:?}", best_path);
                        println!("best pool path: {:?}", best_pool_path);
                        return Ok(Some((current_amount, new_path, new_pool_path)));
                    }
                    if new_path.len() > 7 {
                        return Ok(None)
                    }
                    queue.push((-(new_balance as i128), dst_mint_idx, new_path, new_pool_path));
                }
            }
        }
        return Ok(None)
    }
}
}

// from https://github.com/solana-labs/solana/blob/10d677a0927b2ca450b784f750477f05ff6afffe/sdk/program/src/message/versions/v0/mod.rs#L209
async fn create_tx_with_address_table_lookup(
    client: &RpcClient,
    instructions: &[Instruction],
    address_lookup_table_key: Pubkey,
    payer: &Keypair,
) -> VersionedTransaction {
    let raw_account = client.get_account(&address_lookup_table_key).await.unwrap();
    let address_lookup_table = AddressLookupTable::deserialize(&raw_account.data).unwrap();
    let address_lookup_table_account = AddressLookupTableAccount {
        key: address_lookup_table_key,
        addresses: address_lookup_table.addresses.to_vec(),
    };

    let blockhash = client.get_latest_blockhash().await.unwrap();
    let tx = VersionedTransaction::try_new(
        VersionedMessage::V0(v0::Message::try_compile(
            &payer.pubkey(),
            instructions,
            &[address_lookup_table_account],
            blockhash,
        ).unwrap()),
        &[payer],
    ).unwrap();

    assert!(tx.message.address_table_lookups().unwrap().len() > 0);
    tx
}