use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_client::rpc_config::RpcSendTransactionConfig;
use bincode::serialize;

use anchor_client::solana_sdk::pubkey::Pubkey;

use anchor_client::solana_sdk::signature::{Keypair, Signer};
use anchor_client::{Cluster, Program};
use serde_json::json;
use solana_address_lookup_table_program::state::AddressLookupTable;
use solana_client::rpc_request::RpcRequest;
use solana_program::address_lookup_table::AddressLookupTableAccount;
use solana_program::message::{VersionedMessage, v0};
use solana_sdk::commitment_config::{CommitmentLevel, CommitmentConfig};
use solana_sdk::signature::{read_keypair_file, Signature};
use solana_transaction_status::UiTransactionEncoding;
use std::collections::{HashMap, HashSet};
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

use crate::monitor::pools::pool::{PoolOperations, PoolType};

use crate::utils::{derive_token_address, PoolGraph, PoolIndex, PoolQuote};

pub struct Arbitrager {
    pub token_mints: Vec<Pubkey>,
    pub graph_edges: Vec<HashSet<usize>>, // used for quick searching over the graph
    pub graph: PoolGraph,
    pub cluster: Cluster,
    // vv -- need to clone these explicitly -- vv
    pub connection: RpcClient,
}
impl Arbitrager {
#[async_recursion::async_recursion]

    pub async     fn brute_force_search(
        &self,
        start_mint_idx: usize,
        init_balance: u128,
        curr_balance: u128,
        path: Vec<usize>,
        pool_path: Vec<PoolQuote>,
    ) {
        if curr_balance == 0 {
            return;
        }
        let src_curr = path[path.len() - 1]; // last mint
        let src_mint = self.token_mints[src_curr];

        let out_edges = &self.graph_edges[src_curr];

        // path = 4 = A -> B -> C -> D
        // path >= 5 == not valid bc max tx size is swaps
        if path.len() == 4 {
            return;
        };

        for dst_mint_idx in out_edges {

            
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

            for pool in pools {
                let new_balance =
                    pool.0
                        .get_quote_with_amounts_scaled(curr_balance, &src_mint, &dst_mint);
                let mut new_path = path.clone();
                new_path.push(dst_mint_idx);

                let mut new_pool_path = pool_path.clone();
                new_pool_path.push(pool.clone()); // clone the pointer
                if dst_mint_idx == start_mint_idx {
                    
                    // if new_balance > init_balance - 1086310399 {
                    if (new_balance as f64 > curr_balance as f64 * 1.001) && (new_balance  < u128::from((curr_balance * 333 )/ 100)) {
                        // ... profitable arb!
                        println!("found arbitrage: {:?} -> {:?}", curr_balance, new_balance);

                        // check if arb was sent with a larger size
                        // key = {mint_path}{pool_names}
                        let mint_keys: Vec<String> =
                            new_path.clone().iter_mut().map(|i| i.to_string()).collect();
                        let pool_keys: Vec<String> =
                            new_pool_path.iter().map(|p| p.0.get_name()).collect();
                        let arb_key = format!("{}{}", mint_keys.join(""), pool_keys.join(""));
                        println!("arbkey: {:?}", arb_key);
                        let mut ixs = self.get_arbitrage_instructions(
                            init_balance,
                            &new_path,
                            &new_pool_path,
                        ).await;
                        let mut ix;
                        ixs.0.concat();
                            
    let owner3 = Arc::new(read_keypair_file("/Users/stevengavacs/.config/solana/id.json".clone()).unwrap());
    let owner = owner3.try_pubkey().unwrap();
        let src_ata = derive_token_address(&owner, &dst_mint);
                                // PROFIT OR REVERT instruction
                                 ix = spl_token::instruction::transfer(
                                    &spl_token::id(),
                                    &src_ata,
                                    &src_ata,
                                    &owner,
                                    &[
                                    ],
                                    init_balance as u64,
                                );
                                // flatten to Vec<Instructions>
                                ixs.0.push(vec![ix.unwrap()]);
                                let owner_keypair = Rc::new(read_keypair_file("/Users/stevengavacs/.config/solana/id.json".clone()).unwrap());
                                /* 
    let recent_slot = self.connection
    .get_slot_with_commitment(CommitmentConfig::finalized())
    .unwrap();
                                let (create_ix, table_pk) =
                                
                                solana_address_lookup_table_program::instruction::create_lookup_table(
                                    owner,
                                    owner,
                                    recent_slot,
                                );

                            let latest_blockhash =  self.connection.get_latest_blockhash().unwrap();
                            self.connection
                                .send_and_confirm_transaction(&Transaction::new_signed_with_payer(
                                    &[create_ix],
                                    Some(&owner),
                                    &[&owner3],
                                    self.connection.get_latest_blockhash().unwrap(),
                                ))
                                .unwrap();

                            let mut pool_keys;
                              for ix in ixs.0.iter_mut() {
                                pool_keys = vec![];
                                for ixx in ix.iter_mut() {
                                    for key in &ixx.accounts {
                                        pool_keys.push(key.pubkey);
                                    }
                                let extend_ix = solana_address_lookup_table_program::instruction::extend_lookup_table(
                                    table_pk,
                                    owner,
                                    Some(owner),
                                    pool_keys.to_vec(),
                                );
                        
                                let signature = self.connection
                                    .send_and_confirm_transaction(&Transaction::new_signed_with_payer(
                                        &[extend_ix],
                                        Some(&owner),
                                        &[&owner3],
                                        self.connection.get_latest_blockhash().unwrap()
                                    ))
                                    .unwrap();
                              }
                              
                                
                            } 
                            let versioned_tx =
                            create_tx_with_address_table_lookup(&self.connection, &ixs.0.concat(), 
                                table_pk
                                , &owner3);
                        let serialized_versioned_tx = serialize(&versioned_tx).unwrap();
                        println!(
                            "The serialized versioned tx is {} bytes",
                            serialized_versioned_tx.len()
                        );
                        let serialized_encoded = base64::encode(serialized_versioned_tx);
                        let config: RpcSendTransactionConfig = RpcSendTransactionConfig {
                            skip_preflight: true,
                            preflight_commitment: Some(CommitmentLevel::Processed),
                            encoding: Some(UiTransactionEncoding::Base64),
                            ..RpcSendTransactionConfig::default()
                        };
                    
                        let signature = self.connection
                            .send::<String>(
                                RpcRequest::SendTransaction,
                                json!([serialized_encoded, config]),
                            )
                            ;
                            if signature.is_err(){
                                println!("signature error");
                                println!("{}" , signature.err().unwrap());
                                continue 
                            }
                            let signature = signature.unwrap();
                        println!("Multi swap txid: {}", signature);
                     let resulty =   self.connection
                            .confirm_transaction_with_commitment(
                                &Signature::from_str(signature.as_str()).unwrap(),
                                CommitmentConfig::finalized(),
                            );
                            if resulty.is_err() {
                                println!("resulty error");
                                println!("{}" , resulty.err().unwrap());
                                continue
                            }
                            let resulty = resulty.unwrap();
                            println!("resulty: {:?}", resulty);
                              */
                              let tx = Transaction::new_signed_with_payer(
                                &ixs.0.concat(),
                                Some(&owner),
                                &[&owner3],
                                self.connection.get_latest_blockhash().unwrap(),
                            );
                            let signature = self.connection
                                .send_and_confirm_transaction(&tx)
                                ;
                                if signature.is_err() {
                                    println!("signature error");
                                    println!("{}" , signature.err().unwrap());
                                    
                                }
                            
                                
                    }
                } else if !path.contains(&dst_mint_idx) {
                    // ... search deeper
                    
                    self.brute_force_search(
                        start_mint_idx,
                        init_balance,
                        new_balance,   // !
                        new_path,      // !
                        new_pool_path, // !
                    ).await;
                }
            }
        }
    }

async    fn get_arbitrage_instructions<'a>(
        &self,
        swap_start_amount: u128,
        mint_idxs: &Vec<usize>,
        pools: &Vec<PoolQuote>,    ) -> (Vec<Vec<Instruction>>, bool) {
            
    let owner3 = Arc::new(read_keypair_file("/Users/stevengavacs/.config/solana/id.json".clone()).unwrap());

    let owner = owner3.try_pubkey().unwrap();
        // gather swap ixs
        let mut ixs = vec![];
        let swap_state_pda =
            (Pubkey::from_str("8cjtn4GEw6eVhZ9r1YatfiU65aDEBf1Fof5sTuuH6yVM").unwrap());
        let src_mint = self.token_mints[mint_idxs[0]];
        let src_ata = derive_token_address(&owner, &src_mint);


    // setup anchor things
    
    let provider = anchor_client::Client::new_with_options(
        Cluster::Mainnet,
        owner3.clone(),
        solana_sdk::commitment_config::CommitmentConfig::recent(),
    );
    let program = provider.program(*crate::constants::ARB_PROGRAM_ID).unwrap();

        // initialize swap ix
        let ix = 
        tokio::task::spawn_blocking(move || program
            .request()
            .accounts(tmp_accounts::TokenAndSwapState {
                swap_state: swap_state_pda,
            })
            .args(tmp_ix::StartSwap {
                swap_input: swap_start_amount as u64,
            })
            .instructions()
            .unwrap()).await.unwrap();
        ixs.push(ix);
        let mut flag = false;
        let pubkey = owner;
        for i in 0..mint_idxs.len() - 1 {
            let [mint_idx0, mint_idx1] = [mint_idxs[i], mint_idxs[i + 1]];
            let [mint0, mint1] = [self.token_mints[mint_idx0], self.token_mints[mint_idx1]];
            let pool = &pools[i];
            let mut swap_ix = pool
                .0
                .swap_ix(&mint0, &mint1, swap_start_amount);
            ixs.push(swap_ix.await.1);
            let pool_type = pool.0.get_pool_type();
            match pool_type {
                PoolType::OrcaPoolType => {
                }
                PoolType::MercurialPoolType => {
                }
                PoolType::SaberPoolType => {
                }
                PoolType::AldrinPoolType => {
                }
                PoolType::RaydiumPoolType => {
                }
                PoolType::SerumPoolType => {
                    
                    flag = true;
                }
            }
        }
       
        ixs.concat();
        (ixs,  flag) 

    }

}

// from https://github.com/solana-labs/solana/blob/10d677a0927b2ca450b784f750477f05ff6afffe/sdk/program/src/message/versions/v0/mod.rs#L209
fn create_tx_with_address_table_lookup(
    client: &RpcClient,
    instructions: &[Instruction],
    address_lookup_table_key: Pubkey,
    payer: &Keypair,
) -> VersionedTransaction {
    let raw_account = client.get_account(&address_lookup_table_key).unwrap();
    let address_lookup_table = AddressLookupTable::deserialize(&raw_account.data).unwrap();
    let address_lookup_table_account = AddressLookupTableAccount {
        key: address_lookup_table_key,
        addresses: address_lookup_table.addresses.to_vec(),
    };

    let blockhash = client.get_latest_blockhash().unwrap();
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