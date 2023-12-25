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
use std::collections::{HashMap, HashSet};
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
    #[async_recursion::async_recursion]
pub async fn brute_force_search(
    &self,
    start_mint_idx: usize,
    init_balance: u128,
    mut old_best: u128,
    curr_balance: u128,
    path: Vec<usize>,
    best_path: Vec<usize>,
    pool_path: Vec<PoolQuote>,
    best_pool_path: Vec<PoolQuote>,
    owner3: Arc<Keypair>
 //   sent_arbs: &mut HashSet<String>,
) -> Result<Option<(u128, Vec<usize>, Vec<PoolQuote>)>, anyhow::Error> {
        let src_curr = path[path.len() - 1]; // last mint
    let src_mint = self.token_mints[src_curr];

    let out_edges = &self.graph_edges[src_curr];

    if path.len() == 8 {
        if old_best > init_balance {
            println!(" profitable arb... {:?} -> {:?}", init_balance, old_best);
            return Ok(Some((old_best, best_path, best_pool_path)))
        } else {
            return Ok(None)
        }

    }
        
    for dst_mint_idx in out_edges {
        let arandom = rand::random::<usize>() % out_edges.len();
        let dst_mint_idx = out_edges.iter().nth(arandom).unwrap();
        if self
        .graph
        .0
        .get(&PoolIndex(src_curr))
        .unwrap()
        .0
        .get(&PoolIndex(*dst_mint_idx)).is_none(){
            continue;
        }
        let pools = self
            .graph
            .0
            .get(&PoolIndex(src_curr))
            .unwrap()
            .0
            .get(&PoolIndex(*dst_mint_idx))
            .unwrap();

        let dst_mint_idx = *dst_mint_idx;
        let dst_mint = self.token_mints[dst_mint_idx];

    let owner = owner3.try_pubkey().unwrap();
        let dst_mint_acc = derive_token_address(&owner, &dst_mint);

        for pool in pools {
            // choose a pool at random instead
            let arandom = rand::random::<usize>() % pools.len();
            let pool = pools[arandom].clone();
            let mut new_balance =
                pool.0
                    .get_quote_with_amounts_scaled(curr_balance, &src_mint, &dst_mint).await;
                    let mut new_path = path.clone();
                    new_path.push(dst_mint_idx);
        
                    let mut new_pool_path = pool_path.clone();
                    new_pool_path.push(pool.clone()); // clone the pointer
            if new_balance == 0 {
                continue;
            }
            if dst_mint_idx == start_mint_idx && new_balance > old_best && new_balance <= old_best * 2 {
                // println!("{:?} -> {:?} (-{:?})", init_balance, new_balance, init_balance - new_balance);
                
                // if new_balance > init_balance - 1086310399 {
                    // ... profitable arb!

                    old_best = new_balance;
                    println!(" profitable arb... {:?} -> {:?}", curr_balance, old_best);
                    
                    return (self.brute_force_search(
                        start_mint_idx,
                        init_balance,
                        old_best,
                        new_balance,   // !
                        new_path.clone(),      // !
                        new_path,
                        new_pool_path.clone(), // !
                        new_pool_path,
                        owner3
                    )).await
            } else if dst_mint_idx != start_mint_idx && !path.contains(&dst_mint_idx) {
                return (self.brute_force_search(
                    start_mint_idx,
                    init_balance,
                    old_best,
                    new_balance,   // !
                    new_path.clone(),      // !
                    path,
                    new_pool_path.clone(), // !
                    pool_path,
                    owner3
                )).await

            }
        }
    }
    return Ok(None)
   
    
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