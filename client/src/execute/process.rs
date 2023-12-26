use anyhow::Error;
use axum::extract::path;
use chrono::Utc;
use futures::lock::Mutex;
use rand::Rng;
use rand::seq::SliceRandom;
use solana_address_lookup_table_program::instruction::{extend_lookup_table, create_lookup_table};
use solana_client::nonblocking::rpc_client::RpcClient;
use anchor_client::solana_client::rpc_config::RpcSendTransactionConfig;
use anchor_lang::{system_program, AnchorSerialize};
use bincode::serialize;
use rayon::prelude::*;
use futures::stream::{FuturesUnordered, StreamExt};
use futures::executor::LocalPool;
use futures::task::LocalSpawnExt;

use anchor_client::solana_sdk::pubkey::Pubkey;

use anchor_client::solana_sdk::signature::{Keypair, Signer};
use anchor_client::{Cluster, Program, Client};
use serde_json::json;
use solana_address_lookup_table_program::state::{AddressLookupTable};
use solana_client::rpc_request::RpcRequest;
use solana_program::instruction::AccountMeta;
use solana_program::message::{VersionedMessage, v0};
use solana_program::program_pack::Pack;
use solana_sdk::account::Account;
use solana_sdk::address_lookup_table_account::AddressLookupTableAccount;
use solana_sdk::commitment_config::{CommitmentLevel, CommitmentConfig};
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::signature::{read_keypair_file, Signature};
use solana_transaction_status::UiTransactionEncoding;
use switchboard_solana::get_ixn_discriminator;
use std::cell::RefCell;
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet, BinaryHeap};
use std::f32::consts::E;
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

use crate::constants::{ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID, ARB_PROGRAM_ID};
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
// Assuming each edge has an associated cost or weight.
// This is a placeholder - replace it with your actual method to get the edge weight.
impl Arbitrager {    
    // Entry function
    pub async fn find_weights(&self, start_mint_idx: usize, init_balance: u128, max_hops: usize) -> (Vec<usize>, u64, Vec<PoolQuote>, Vec<u128>) {
        let mut routes = self.find_weights_recursive(start_mint_idx, start_mint_idx, init_balance, max_hops, 0, Vec::new(), vec![start_mint_idx], vec![init_balance]).await;
        if routes.is_err()
        {
            return (vec![], 0, vec![], vec![]);
        }
        let mut routes = routes.unwrap();
        if routes.is_none() {
            return (vec![], 0, vec![], vec![]);
        }
        let mut routes = routes.unwrap();
        routes


    }
    pub async fn find_max_yield_path(&self, mint_idx: usize, 
        pool_path: Vec<PoolQuote>, balances: Vec<u128>, 
        start_mint_idx: usize, amount: u128, max_hops: usize) -> Result<Option<(Vec<usize>, u64, Vec<PoolQuote>, Vec<u128>)>, Box<dyn std::error::Error + Send>> {
        let path = self.find_yield_recursive(mint_idx, 
            pool_path,
            balances,
            start_mint_idx, amount, amount, max_hops, 0, vec![start_mint_idx], 0).await?;
        if path.is_none(){
            return Ok(None);
        }
        let path = path.unwrap();
        if path.0.len() > 2 {
            println!("path is {:?}", path.0);
            
        Ok(Some(path))
        }
        else {
            Ok(None)
        }
    }
    #[async_recursion::async_recursion]

    async fn find_yield_recursive (&self, current_mint_idx: usize,
        pool_path: Vec<PoolQuote>, balances: Vec<u128>,

         start_mint_idx: usize, init_amount: u128, amount: u128, max_hops: usize, current_hop: usize, path: Vec<usize>, current_yield: u64) -> Result<Option<(Vec<usize>, u64, Vec<PoolQuote>, Vec<u128>)>, Box<dyn std::error::Error + Send>> {
        if current_hop >= max_hops {
            return Ok(None);
        }


        if let Some(edges) = self.graph_edges.get(current_mint_idx) {
            for mut edge in edges {
                let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() % edges.len() as u128;
                edge = &edges.iter().nth(timestamp as usize).unwrap();
                if path.contains(edge) {
                    continue;
                }
                if current_hop >= max_hops - 1 {
                    edge = &start_mint_idx;
                }
                let edge = *edge;
                let pools = self.graph
                    .0
                    .get(&PoolIndex(current_mint_idx))
                    .and_then(|p| p.0.get(&PoolIndex(edge)));
                   if pools.is_none() {
                       continue;
                     }
                let pool = pools.unwrap();
                let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() % pool.len() as u128;

                let pool = pool[timestamp as usize].clone();
                    let new_balance = pool.0.get_quote_with_amounts_scaled(amount, &self.token_mints[current_mint_idx], &self.token_mints[edge]);
                    if new_balance > 0 {
                        let mut new_pool_path = pool_path.clone();
                            new_pool_path.push(pool.clone());


                            let mut new_balances = balances.clone();
                            new_balances.push(new_balance);

                            let mut new_path = path.clone();
                            new_path.push(edge);

                            if start_mint_idx == edge  && new_balance as f64 > init_amount as f64 * 1.02 && new_balance as f64 <= init_amount as f64 * 3.0{
                                println!("found a path with yield {} {:?}", new_balance as f64 / init_amount as f64, new_path);

                                return Ok(Some((new_path, new_balance as u64, new_pool_path, new_balances)));
                            }
                            
                           return Ok(self.find_yield_recursive(edge, new_pool_path, new_balances, start_mint_idx, init_amount, new_balance, max_hops, current_hop + 1, new_path, new_balance as u64).await?)
                            
                        }
                    }


    }
    Ok(None)

}
                        
                                // Recursive function
   async fn find_weights_recursive(&self, current_mint_idx: usize, start_mint_idx: usize, amount: u128, max_hops: usize, current_hop: usize, pool_path: Vec<PoolQuote>, path: Vec<usize>, balances: Vec<u128>) ->
   
   Result<Option<(Vec<usize>, u64, Vec<PoolQuote>, Vec<u128>)>, Box<dyn std::error::Error + Send>> {
      
        
        if let Some(edges) = self.graph_edges.get(current_mint_idx) {
            for mut edge in edges {
                let pp: Vec<PoolQuote> = vec![];
                let new_edge = self.find_max_yield_path(*edge, pp, vec![amount], start_mint_idx, amount, max_hops-current_hop).await.unwrap();
                if new_edge.is_none() {
                    continue;
                }
                let new_edge = new_edge.unwrap();
                if new_edge.0.len() == 0 {
                    continue;
                }
                
                return Ok(Some((new_edge.0, new_edge.1, new_edge.2, new_edge.3)));
                }
            }
        Ok(None)
        

    }

    #[async_recursion::async_recursion]
    async fn optimized_search_recursive(
        &self,
        current_balance: u128,
        init_balance: u128,
        mint_idx: usize,
        start_mint_idx: usize,
        path: Vec<usize>,
        amounts: Vec<u128>,
        pool_path: Vec<PoolQuote>,
        best_balance: u128
    ) -> Result<Option<(Vec<usize>, u64, Vec<PoolQuote>, Vec<u128>)>, Box<dyn std::error::Error + Send>> {
      
    
       
    
        // Recursively explore each edge
        let mut edge_weight_vecs = self.find_weights(mint_idx, current_balance, 8).await;
        
        if edge_weight_vecs.0.len() == 0 {
            return Ok(None);
        }

        Ok(Some(edge_weight_vecs))
    }
    
    
    // Wrapper function to start the recursion
    pub async fn optimized_search(
        self,
        start_mint_idx: usize,
        init_balance: u128
    ) -> Result<Option<(Vec<usize>, u64, Vec<PoolQuote>, Vec<u128>)>, Box<dyn std::error::Error + Send>> {
        self.optimized_search_recursive(init_balance, init_balance, start_mint_idx, start_mint_idx, vec![start_mint_idx], vec![init_balance], vec![], init_balance).await
    }
    
    
}




pub  async  fn get_arbitrage_instructions<'a>(
    token_mints: Arc<Vec<Pubkey>>,
    src_mint: Pubkey,
        swap_start_amount: u128,
        mint_idxs: Vec<usize>,
        amounts: Vec<u128>,
        pools: Arc<Vec<PoolQuote>>,   
    owner: Arc<Keypair>
     ) -> (Vec<Vec<Instruction>>, bool) {
            
        // gather swap ixs
        let mut ixs = vec![];
        let swap_state_pda =
            (Pubkey::from_str("8cjtn4GEw6eVhZ9r1YatfiU65aDEBf1Fof5sTuuH6yVM").unwrap());
        let src_ata = derive_token_address(&owner.clone().pubkey(), &src_mint);

    let provider = anchor_client::Client::new_with_options(
        Cluster::Mainnet,
        owner.clone(),
        solana_sdk::commitment_config::CommitmentConfig::confirmed(),
    );
        let program: Program<Arc<Keypair>> = provider.program(*crate::constants::ARB_PROGRAM_ID).unwrap();

        // initialize swap ix
        let ix = program
            .request()
            .accounts(tmp_accounts::TokenAndSwapState {
                swap_state: swap_state_pda,
            })
            .args(tmp_ix::StartSwap {
                swap_input: swap_start_amount as u64,
            })
            .instructions()
            .unwrap();
            
        ixs.push(ix);
        let mut flag = false;
        println!("{:?}", amounts);
        let mut swap_start_amount = amounts[0];


        for i in 0..mint_idxs.len() - 1 {

            let program: Program<Arc<Keypair>> = provider.program(*crate::constants::ARB_PROGRAM_ID).unwrap();

            let [mint_idx0, mint_idx1] = [mint_idxs[i], mint_idxs[i + 1]];
            println!("mint idxs are {} {}", mint_idx0, mint_idx1);
            let pool = &pools[i];
            let pool3 = pool.clone();
            let token_mints = token_mints.clone();
            println!("getting quote with amounts scaled {} {} {} ", i, token_mints[mint_idx0].clone(), token_mints[mint_idx1].clone());
            let swap_ix = pool3.0.swap_ix(token_mints[mint_idx0].clone(),
            token_mints[mint_idx1].clone(), swap_start_amount,
            owner.clone().pubkey(), program);
            println!("swap ix is {:?}", swap_ix.1.len());
             swap_start_amount = amounts[i];
            if swap_start_amount == 0 {
                return (vec![], false);
            }
            ixs.push(swap_ix.1);
            println!("ixs is {}", ixs.len());
        }
       
        ixs.concat();
        (ixs,  flag) 

    }


    fn deduplicate_public_keys(public_keys: &Vec<String>) -> Vec<Pubkey> {
        let mut unique_keys = HashSet::new();
        for key in public_keys {
            let pubkey = Pubkey::from_str(key).unwrap();
            unique_keys.insert(pubkey);
        }
        unique_keys.into_iter().collect()
    }
    
    fn fetch_existing_luts(
        lutties: &Vec<AddressLookupTableAccount>,
        _connection: &RpcClient,
        needed_keys: &Vec<Pubkey>,
    ) -> Result<Vec<AddressLookupTableAccount>, Box<dyn std::error::Error>> {
       
        // iterate thru luts. 
        // count how many keys we have in each lut - create a HashMap of lut key to number of keys
    
        let mut lut_key_to_num_keys = HashMap::new();
        for lut in lutties {
            // count how many public_keys are in lut.addresses
            let mut num_keys = 0;
            for address in &lut.addresses {
                if needed_keys.contains(address) {
                    num_keys += 1;
                }
            }
            lut_key_to_num_keys.insert(lut.key, num_keys);
        }
    
        // sort lut_key_to_num_keys by num_keys
        let mut lut_key_to_num_keys = lut_key_to_num_keys
            .into_iter()
            .collect::<Vec<(Pubkey, usize)>>();
        lut_key_to_num_keys.sort_by(|a, b| a.1.cmp(&b.1));
    
        // create a new vector of luts sorted by num_keys
        let mut sorted_luts = Vec::new();
        for lut in lut_key_to_num_keys {
            for lut2 in lutties {
                if lut.0 == lut2.key {
                    sorted_luts.push(lut2.clone());
                }
            }
        }
        //println!("sorted luts: {:?}", sorted_luts.len());
        Ok(sorted_luts)
    
    }
    
    fn get_public_keys_from_luts(luts: &Vec<AddressLookupTableAccount>) -> Vec<String> {
        let mut public_keys = Vec::new();
        for lut in luts {
            for address in &lut.addresses {
                public_keys.push(address.to_string());
            }
        }
        public_keys
    }
    
    fn get_remaining_public_keys(
        unique_public_keys: &Vec<Pubkey>,
        luts_public_keys: &Vec<String>,
    ) -> Vec<Pubkey> {
        let luts_public_keys: HashSet<Pubkey> = luts_public_keys
        .iter()
        .map(|key| Pubkey::from_str(key).unwrap())
        .collect();
    
    unique_public_keys
        .iter()
        .filter(|key| !luts_public_keys.contains(key))
        .cloned()
        .collect()}
    
pub     fn create_and_or_extend_luts(
        remaining_public_keys: &Vec<Pubkey>,
        connection: &solana_client::rpc_client::RpcClient,
        luts: &mut Vec<AddressLookupTableAccount>,
        payer: &Keypair,
    ) -> Result<Vec<AddressLookupTableAccount>, Box<dyn std::error::Error>> {
        let mut used_luts = Vec::new();
    
        for pubkeys in remaining_public_keys.chunks(25) {
            let (lut, _index) = find_or_create_lut(connection, payer, luts, remaining_public_keys.len()).unwrap()   ;
                let extend_ix = extend_lookup_table(
                    lut.key,
                    payer.pubkey(),
                    Some(payer.pubkey()),
                    pubkeys.to_vec(),
                );
                let latest_blockhash = connection.get_latest_blockhash(); 
                //println!("extending lut: {:?}", lut.key);
               let hm = connection
                    .send_transaction(&VersionedTransaction::try_new(
                            VersionedMessage::V0(v0::Message::try_compile(
                                &payer.pubkey(),
                                &[extend_ix],
                                &[],
                                latest_blockhash.unwrap(),
                            ).unwrap()),
                            &[payer],
                        ).unwrap()
                    );
                    if !hm.is_err() {
                        let signature = hm.unwrap();
                        
                    }
                    
    
                        
                used_luts.push(lut);
            }
    
        Ok(used_luts)
    }
     fn find_or_create_lut(
        connection:  &solana_client::rpc_client::RpcClient,
        payer: &Keypair,
        luts: &mut Vec<AddressLookupTableAccount>,
        howmany: usize
    ) -> Result<(AddressLookupTableAccount, usize), Box<dyn std::error::Error>> {
        luts.shuffle(&mut rand::thread_rng());
        for (index, lut) in luts.iter().enumerate() {
            let acc = connection.get_account(&lut.key);
            if acc.is_err() {
                continue;
            }
            let acc = acc.unwrap();
            let address_lookup_table = AddressLookupTable::deserialize(&acc.data).unwrap();
            //println!("{}, {}", lut.addresses.len(), address_lookup_table.meta.authority.unwrap() == payer.pubkey());
            if lut.addresses.len() < (255_usize -howmany) && address_lookup_table.meta.authority.unwrap() == payer.pubkey() {
                return Ok((lut.clone(), index));
            }
        }
        Ok((create_new_lut(connection, payer).unwrap(), luts.len()))
    }
    
     fn create_new_lut(
        connection: &solana_client::rpc_client::RpcClient,
        payer: &Keypair,
    ) -> Result<AddressLookupTableAccount, Box<dyn std::error::Error>> {
        // Create a new AddressLookupTable
        let recent_slot = connection
        .get_slot_with_commitment(CommitmentConfig::confirmed())
        .unwrap()//"237009123 is not a recent slot"
        - 50;
        let (create_ix, table_pk) =
            create_lookup_table(
                payer.pubkey(),
                payer.pubkey(),
                recent_slot,
            );
        let latest_blockhash = connection.get_latest_blockhash().unwrap();  
        
        //println!("creating lut: {:?}", table_pk);
      let hm = connection
        .send_transaction(&VersionedTransaction::try_new(
                VersionedMessage::V0(v0::Message::try_compile(
                    &payer.pubkey(),
                    &[create_ix],
                    &[],
                    latest_blockhash,
                ).unwrap()),
                &[payer],
            ).unwrap()
        );
        if !hm.is_err() {
            let signature = hm.unwrap();
            
        }
    
        
        let lut = AddressLookupTableAccount {
            key: table_pk,
            addresses: vec![],
        };
    
    
        let file = std::fs::read("./src/luts.json").unwrap();
        let string = String::from_utf8(file).unwrap();
            let mut lutties: Vec<String> = serde_json::from_str(&string).unwrap();
            ////println !("lutties: {:?}", lutties.len());
    // dedupe
            lutties.sort();
            lutties.dedup();
            ////println !("lutties: {:?}", lutties.len());
        // write new lut to lutties to file
        lutties.push(lut.key.to_string());
        ////println !("lutties+1: {:?}", lutties.len());
        save_luts_to_file(&lutties).unwrap();
        
        Ok(lut)
    
    }
    use std::fs;
    
    fn save_luts_to_file(lutties: &Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        // write to lut.json 
        let mut lutties = lutties.clone();
        lutties.sort();
        lutties.dedup();
        let lutties = serde_json::to_string(&lutties).unwrap();
        fs::write("./src/luts.json", lutties).unwrap();
    
        Ok(())
    }
    pub fn calculate_recent_fee(
        write_locked_accounts: &[Pubkey],
        connection: &solana_client::rpc_client::RpcClient,
    ) -> u64 {
        //println!("calculating recent fee");
        //println!("write locked accounts: {:?}", write_locked_accounts.len());
        // do in chunks of 100 
        let write_locked_accounts = write_locked_accounts.to_vec();
        let chunks = write_locked_accounts.chunks(100);
        for chunk in chunks {
                let account_infos = connection.get_multiple_accounts_with_commitment(
                    chunk,
                    CommitmentConfig::confirmed()
                ).unwrap().value;
                let mut index = 0;
                let write_locked_accounts = &account_infos
                .into_iter()
                .map(|account: Option<Account>| {
                    index += 1;
                    if account.is_some() {
                        write_locked_accounts[index-1]
                    }
                    else {
                        Pubkey::default()
                    }
                })
                .collect::<Vec<Pubkey>>()
                .iter()
                .cloned()
                .collect::<std::collections::HashSet<Pubkey>>()
                .iter()
                .filter(|pubkey| **pubkey != Pubkey::default())
                .cloned()
                .collect::<Vec<Pubkey>>();
                //println!("write locked accounts that were resolved on this cluster: {:?}", write_locked_accounts.len());
                let recent_fees = connection.get_recent_prioritization_fees(
                    write_locked_accounts
                ).unwrap_or_default();
                let fee = recent_fees
                .iter()
                .map(|fee| fee.prioritization_fee)
                .filter(|fee| *fee != 0)
                .sum::<u64>()
                .checked_div(recent_fees.len() as u64)
                .unwrap_or(138 * write_locked_accounts.len() as u64)
                .checked_div(write_locked_accounts.len() as u64)
                .unwrap_or(138);
                if fee != 138 {
                    return fee;
                }
            }
        138
    }
    
pub    async fn get_address_lookup_table_accounts(client: &solana_client::rpc_client::RpcClient, keys: Vec<String>, payer: Pubkey) -> Vec<AddressLookupTableAccount> {
        let keys = &keys.iter().
        map(|key| {
            Pubkey::from_str(key).unwrap()
        })
        .collect::<Vec<Pubkey>>();
        let mut luts: Vec<AddressLookupTableAccount> = Vec::new();
        // do in chunks of 100
        let  keys = keys.clone();
        let  chunks= keys.chunks(100);
        
        for chunk in chunks {
                let raw_accounts = client.get_multiple_accounts(chunk).unwrap();
    
                for i in 0..raw_accounts.len() {
                    if raw_accounts[i].is_some() {
                        let raw_account = raw_accounts[i].as_ref().unwrap();
                        let address_lookup_table = AddressLookupTable::deserialize(&raw_account.data).unwrap();
                        if address_lookup_table.meta.authority.unwrap() == payer {
                            let address_lookup_table_account = AddressLookupTableAccount {
                                key: chunk[i],
                                addresses: address_lookup_table.addresses.to_vec(),
                            };
                            luts.push(address_lookup_table_account);
                        }
                        
                    }
                }
            }
        
        luts 
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