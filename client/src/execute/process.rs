use rand::seq::SliceRandom;
use solana_address_lookup_table_program::instruction::{extend_lookup_table, create_lookup_table};
use solana_client::nonblocking::rpc_client::RpcClient;
use anchor_client::solana_client::rpc_config::RpcSendTransactionConfig;
use anchor_lang::system_program;
use bincode::serialize;
use rayon::prelude::*;
use futures::stream::{FuturesUnordered, StreamExt};

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
impl Arbitrager {
    pub async fn optimized_search(
        &mut self,
        start_mint_idx: usize,
        init_balance: u128,
        path: Vec<usize>,
        token_mints: Vec<Pubkey>,
        graph_edges: Vec<HashSet<usize>>,
        graph: PoolGraph,
    ) -> Result<Option<(u128, Vec<usize>, Vec<PoolQuote>)>, anyhow::Error> {

// Create a FuturesUnordered to hold the tasks
let mut tasks = FuturesUnordered::new();
        let mut best_profit = 0;
        let mut current_amount = init_balance;
        let mut best_path = Vec::new();
        let mut best_pool_path = Vec::new();
        let mut best_routes = BinaryHeap::new(); // Keep track of the top 10 most profitable routes

        let mut queue = BinaryHeap::new();
        queue.push((0, start_mint_idx, path, Vec::new()));
    
    // ** setup RPC connection
    let connection_url = "https://jarrett-solana-7ba9.mainnet.rpcpool.com/8d890735-edf2-4a75-af84-92f7c9e31718";
    
    println!("using connection: {}", connection_url);

    let connection = Arc::new( solana_client::rpc_client::RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::confirmed()));
let cluster = Cluster::Custom(
    connection_url.to_string(),
    connection_url.to_string(),
);

let owner_kp_path = "/home/ubuntu/.config/solana/id.json";

// setup anchor things
let owner = read_keypair_file(owner_kp_path).unwrap();
let rc_owner = Arc::new(owner);

let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();

    // setup anchor things

    let provider = anchor_client::Client::new_with_options(
        Cluster::Custom(
            connection_url.to_string(),
            connection_url.to_string(),
        ),
        rc_owner.clone(),
        solana_sdk::commitment_config::CommitmentConfig::confirmed(),
    );
    let program = provider.program(*ARB_PROGRAM_ID).unwrap();
    let program_async = program.async_rpc();
    // setup anchor things
    let owner2 = rc_owner.clone();
    let _owner2 = Arc::new(owner2);
    let provider = Client::new_with_options(
        cluster.clone(),
        rc_owner.clone(),
        CommitmentConfig::confirmed(),
    );
    let _program = provider.program(*ARB_PROGRAM_ID);

    let _min_swap_amount = 10_u128.pow(3_u32); // scaled! -- 1 USDC

    let src_ata = derive_token_address(&rc_owner.pubkey(), &usdc_mint);

    let init_token_acc = connection.clone().get_account(&src_ata).unwrap();
    let init_token_balance: u128 = spl_token::state::Account::unpack(&init_token_acc.data).unwrap().amount as u128;
    let swap_start_amount = init_token_balance; // scaled!
    
    println!("searching for arbitrages...");
    let _min_swap_amount = 10_u128.pow(4_u32); // scaled! -- 1 USDC
    let swap_start_amount = init_token_balance; // scaled!
    println!("swap start amount = {}", swap_start_amount);
    // Spawn as tokio task
  
    // create Arbitrager and iterate loops in 60 futures at a time
    
        
    let arbitrager =Arbitrager {
        token_mints: token_mints.clone(),
        graph_edges: graph_edges.clone(),
        graph: graph.clone(),
        cluster: Cluster::Custom(
            connection_url.to_string(),
            connection_url.to_string()),
        connection: Arc::new(solana_client::nonblocking::rpc_client::RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::confirmed()))
    };
    let mut a = arbitrager.clone();
    let connection = solana_client::rpc_client::RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::confirmed());
            let mut arbs: Vec<(u128, Vec<usize>, Vec<PoolQuote>)> = vec![];
          let mut graph_edges = &self.graph_edges;
        while let Some((profit,  mint_idx, path, pool_path)) = queue.pop() {
            let profit = -(profit as i128);
            let out_edges = &graph_edges[mint_idx];

            let mut arb_pools = vec![];
            let mut arb_paths = vec![];
            let mut arb_amounts = vec![];
for dst_mint_idx in out_edges.clone() {
    // Clone necessary data to move into the async block
    let token_mints = Arc::new(a.token_mints.clone());
    let mut graph = self.graph.clone();
    let mut mint_idx = mint_idx.clone();
    let mut path = path.clone();
    let mut pool_path = pool_path.clone();
    let mut best_path = best_path.clone();
    let mut best_pool_path = best_pool_path.clone();
    let mut best_routes = best_routes.clone();
    let mut arbs = arbs.clone();
    let mut arb_paths = arb_paths.clone();
    let mut arb_pools = arb_pools.clone();
    let mut arb_amounts = arb_amounts.clone();

    let task = tokio::task::spawn(async move {
        
        if let Some(pools) = graph
        .0
        .get(&PoolIndex(mint_idx))
        .and_then(|p| p.0.get(&PoolIndex(dst_mint_idx)))
    {
        for mut pool in pools {
            pool = pools.choose(&mut rand::thread_rng()).unwrap();
                let mut profit = profit as u128;
                let new_balance = pool.get_quote_with_amounts_scaled(current_amount, &token_mints.clone()[mint_idx], &token_mints.clone()[dst_mint_idx]);

                if new_balance == 0 {
                    continue;
                }
                 current_amount = new_balance;

                let mut new_path = path.clone();
                new_path.push(dst_mint_idx);

                let mut new_pool_path = pool_path.clone();
                new_pool_path.push(pool.clone());

                    best_profit = new_balance;
                    best_path = new_path.clone();
                    best_pool_path = new_pool_path.clone();
                profit = new_balance - init_balance;

                if new_path.len() >= 2 && dst_mint_idx == start_mint_idx && new_balance > init_balance && new_balance <= init_balance * 3 {
                    best_routes.push((current_amount, new_path.clone(), new_pool_path.clone()));
                    println!("best routes len {}", best_routes.len());
                    if best_routes.len() > 3 {
                        println!("best_routes.len() > 10");
                        let connection = solana_client::rpc_client::RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::confirmed());
                        let init_token_acc = connection.get_account(&src_ata).unwrap();
                        let init_token_balance: u128 = spl_token::state::Account::unpack(&init_token_acc.data).unwrap().amount as u128;
                        let swap_start_amount = init_token_balance; // scaled!
                        
                        let _min_swap_amount = 10_u128.pow(4_u32); // scaled! -- 1 USDC
                        let swap_start_amount = init_token_balance; // scaled!
            
             println!("searching for arbitrages7...");
          println!("there are {} arbs", arbs.len());
                for arb in arbs.clone() {
                    let mut arb_amount = swap_start_amount;
                    for i in 0..arb.2.len() - 1 {
                        let [mint_idx0, mint_idx1] = [arb.1[i], arb.1[i + 1]];
                        let pool = arb.2[i].clone();
                        let mints = pool.get_mints();
                        let mint0 = token_mints.clone()[mint_idx0].clone(); 
                        let mint1 = token_mints.clone()[mint_idx1].clone();
        
                        
                        arb_amount = pool.get_quote_with_amounts_scaled(
                            arb_amount,
                            &mint0,
                            &mint1
                        );
                        if arb_amount == 0 {
                            break;
                        }
                        arb_paths.push(arb.1.clone());
                        arb_pools.push(arb.2.clone());
        
                    }
                    arb_amounts.push(arb_amount);
                }
               
                }
                  // Handle the successful case
           
                                    }
                                }
                            }
                            (arb_paths, arb_pools, arb_amounts)
    });

    tasks.push(task);
}
        // Process the tasks in batches
        while let Some(result) = tasks.next().await {
            match result {
                Ok((arb_paths, arb_pools, arb_amounts)) => {
                    
                    let mut largest_idx = 0;
                    let mut largest_amount = 0;
                    for i in 0..arb_amounts.len() {
                        if arb_amounts[i] > largest_amount {
                            largest_idx = i;
                            largest_amount = arb_amounts[i];
                        }
                    }

                    if arb_paths.len() > 0 {
            
                        let arb_path = arb_paths[largest_idx].clone();
                 let arb_pools = arb_pools[largest_idx].clone();
                 let arb_amount = arb_amounts[largest_idx];
                 println!("lorgest arb amount is {:?}", arb_amount);
                
                 let mint_keys: Vec<String> =
                 arb_path.clone().iter_mut().map(|i| i.to_string()).collect();
             let pool_keys: Vec<String> =
             arb_pools.iter().map(|p| p.0.get_name()).collect();
             let _arb_key = format!("{}{}", mint_keys.join(""), pool_keys.join(""));
             //println!("arbkey: {:?}", arb_key);/* 
             let usdc_mint = usdc_mint.clone();
             let src_ata = src_ata.clone();
            let init_token_balance = init_token_balance.clone();
            let arb_path = arb_path.clone();
            let pubkey = rc_owner.clone().pubkey();
            let owner2 = rc_owner.clone();
                let usdc_mint = Arc::new(usdc_mint.clone());
                let arb_pools = Arc::new(arb_pools.clone());
                
               let mut ixs = get_arbitrage_instructions(
                    Arc::new(a.token_mints.clone()),
                    *usdc_mint.clone(),
                    init_token_balance.clone(),
                    arb_path.clone(),
                    arb_pools.clone(),
                    owner2.clone(),
                ).await;
            
            let mut ixs = ixs.0.concat();
            
            println!("ixs: {:?}", ixs.len());
            let _hydra_ata = derive_token_address(&Pubkey::from_str("2bxwkKqwzkvwUqj3xYs4Rpmo1ncPcA1TedAPzTXN1yHu").unwrap(), &usdc_mint);
            let ix = spl_token::instruction::transfer(
             &spl_token::id(),
             &src_ata,
             &src_ata,
             &pubkey,
             &[  
             ],
             init_token_balance as u64,
            ).unwrap();
            ixs.push(ix);
            let connection = solana_client::rpc_client::RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::confirmed());
            let recent_fees = calculate_recent_fee(ixs.
            iter()
            .flat_map(|ix| ix.accounts.iter().map(|acc| 
            if acc.is_writable { acc.pubkey } else { Pubkey::default() })
            .collect::<Vec<Pubkey>>()
            .iter()
            .cloned()
            .collect::<std::collections::HashSet<Pubkey>>()
            .iter()
            .filter(|pubkey| **pubkey != Pubkey::default())
            .cloned()
            .collect::<Vec<Pubkey>>())
            .collect::<Vec<Pubkey>>().as_slice(),
            &connection);
            println!("recent fees: {:?}", recent_fees);
            
            let mut  needed_keys = ixs.
            iter()
            .flat_map(|ix| ix.accounts.iter().map(|acc| 
            acc.pubkey.to_string()
            )
            .collect::<Vec<String>>())
            .collect::<Vec<String>>();
            let mut missing_keys = Vec::new();
            
            let file = std::fs::read("./src/luts.json").unwrap();
            let string = String::from_utf8(file).unwrap();
            let mut lutties: Vec<String> = serde_json::from_str(&string).unwrap();
            ////println !("lutties: {:?}", lutties.len());
            // dedupe
            lutties.sort();
            lutties.dedup();
            let mut lutties: Vec<AddressLookupTableAccount> = get_address_lookup_table_accounts(&connection, lutties.clone(), pubkey).await;
            
            let mut lutties_public_keys = lutties.
            iter()
            .flat_map(|lut| {
            lut.addresses.clone()
            })
            .collect::<Vec<Pubkey>>();
            
            lutties_public_keys.sort();
            lutties_public_keys.dedup();
            needed_keys.sort();
            needed_keys.dedup();
            for key in needed_keys.clone() {
            if !lutties_public_keys.contains(&Pubkey::from_str(&key).unwrap()) {
            missing_keys.push(key);
            }
            }
            //println!("missing keys: {:?}", missing_keys.len());
            let mut new_lutties = create_and_or_extend_luts(
            &missing_keys.iter().map(|key| Pubkey::from_str(key).unwrap()).collect::<Vec<Pubkey>>(),
            &connection,
            &mut lutties,
            &rc_owner,
            ).unwrap();
            // find the top 4 luts with the most needed keys
            let mut usized_lutties = lutties.
            iter()
            .map(|lut| {
            let mut num_keys = 0;
            for key in &needed_keys.clone() {
            if lut.addresses.contains(&Pubkey::from_str(key).unwrap()) {
            num_keys += 1;
            }
            }
            (lut.clone(), num_keys)
            })
            .collect::<Vec<(AddressLookupTableAccount, usize)>>()
            .iter().filter(|&lut| lut.1 > 5).cloned()
            .collect::<Vec<(AddressLookupTableAccount, usize)>>();
            usized_lutties.sort_by(|a, b| a.1.cmp(&b.1));
            usized_lutties.reverse();
            let rounded = round::round(usized_lutties.len() as f64 / 1.0, 0) as usize;
            usized_lutties = usized_lutties[0..rounded].to_vec();
            lutties = usized_lutties.iter().map(|lut| lut.0.clone()).collect::<Vec<AddressLookupTableAccount>>();
            lutties.append(&mut new_lutties);
            println!("lutties {:?}, needed_keys {:?}, missing_keys {:?}", lutties.len(), needed_keys.len(), missing_keys.len());
            // find needed_keys that are missing from lutties
            
            
            let priority_fee_ix = ComputeBudgetInstruction::set_compute_unit_price(
            recent_fees );
            ixs.insert(
            0, priority_fee_ix
            );
            let blockhash = connection.get_latest_blockhash();
            
            let owner_signer: &dyn solana_sdk::signature::Signer = &*rc_owner;
            let signers = [owner_signer];
               
                
                        let tx = VersionedTransaction::try_new( VersionedMessage::V0(v0::Message::try_compile(
                            &rc_owner.pubkey(),
                            &ixs,
                            &lutties,
                            blockhash.unwrap(),
                            ).unwrap()), &signers);
                        if tx.is_err() {
                            continue;
                        }
                        let tx = tx.unwrap();
                        let connection = solana_client::rpc_client::RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::confirmed());
                        let sig = connection.send_transaction(&tx);
                        if sig.is_err() {
                            continue;
                        }
                        let sig = sig.unwrap();
                        println!("sent tx: {:?}", sig);
                }
            }
                Err(e) => {
                    println!("error: {:?}", e);
                }
            }
        }
        }
        Ok(best_routes.pop())
    }
}



pub  async  fn get_arbitrage_instructions<'a>(
    token_mints: Arc<Vec<Pubkey>>,
    src_mint: Pubkey,
        swap_start_amount: u128,
        mint_idxs: Vec<usize>,
        pools: Arc<Vec<PoolQuote>>,   
    owner: Arc<Keypair>
     ) -> (Vec<Vec<Instruction>>, bool) {
            
        // gather swap ixs
        let mut ixs = vec![];
        let swap_state_pda =
            (Pubkey::from_str("8cjtn4GEw6eVhZ9r1YatfiU65aDEBf1Fof5sTuuH6yVM").unwrap());
        let src_ata = derive_token_address(&owner.clone().pubkey(), &src_mint);


    // setup anchor things
    let connection_url: &str = "https://jarrett-solana-7ba9.mainnet.rpcpool.com/8d890735-edf2-4a75-af84-92f7c9e31718";

    let provider = anchor_client::Client::new_with_options(
        Cluster::Custom(
            connection_url.to_string(),
            connection_url.to_string(),
        ),
        owner.clone(),
        solana_sdk::commitment_config::CommitmentConfig::confirmed(),
    );
    let program = provider.program(*ARB_PROGRAM_ID).unwrap();
println!("getting quote with amounts scaled");
        // initialize swap ix
        let ix = 

            program
            .request()
            .accounts(tmp_accounts::TokenAndSwapState {
                swap_state: swap_state_pda,
            })
            .args(tmp_ix::StartSwap {
                swap_input: swap_start_amount as u64,
            })
            .instructions();
        println ! ("ix issss");
if ix.is_err() {
    println!("ix is err");
return (vec![], false);
}
let ix = ix.unwrap();

            
        ixs.push(ix);
        let mut flag = false;
        let pubkey = owner;
        let mut swap_start_amount = swap_start_amount;


        for i in 0..mint_idxs.len() - 1 {
            let [mint_idx0, mint_idx1] = [mint_idxs[i], mint_idxs[i + 1]];
            let mint0 = token_mints[mint_idx0].clone();
            let mint1 = token_mints[mint_idx1].clone();
            let pool = &pools[i];
            let mint0_clone = mint0.clone();
            let mint1_clone = mint1.clone();
            let pool3 = pool.clone();
            let token_mints = token_mints.clone();
            println!("getting quote with amounts scaled {} {} {} ", i, mint0_clone, mint1_clone);
            let swap_ix =tokio::task::spawn_blocking(move || { pool3.swap_ix(token_mints[mint_idx0].clone(), token_mints[mint_idx1].clone(), swap_start_amount) }).await.unwrap();
            
            let pool2 = pool.0.clone();
             swap_start_amount = tokio::task::spawn_blocking(move || {
 pool2.get_quote_with_amounts_scaled(
                swap_start_amount,
                &mint0,
                &mint1
            )}).await.unwrap();
            if swap_start_amount == 0 {
                return (vec![], false);
            }
            ixs.push(swap_ix.1);
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
    
     fn create_and_or_extend_luts(
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
    
    async fn get_address_lookup_table_accounts(client: &solana_client::rpc_client::RpcClient, keys: Vec<String>, payer: Pubkey) -> Vec<AddressLookupTableAccount> {
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