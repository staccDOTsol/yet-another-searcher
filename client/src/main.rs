use solana_client::nonblocking::rpc_client::RpcClient;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::read_keypair_file;
use anchor_client::solana_sdk::signature::{Keypair, Signer};
use tmp::accounts as tmp_accounts;
use tmp::instruction as tmp_ix;

use futures::Future;
use rand::{seq::SliceRandom};


use futures::future::join_all;
use log::{debug};
use solana_address_lookup_table_program::instruction::{extend_lookup_table, create_lookup_table};
use solana_address_lookup_table_program::state::AddressLookupTable;
use solana_program::address_lookup_table_account::AddressLookupTableAccount;
use solana_program::instruction::Instruction;
use solana_program::message::{VersionedMessage, v0};
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use core::panic;
use redb::{ReadableTable, TableDefinition};
use solana_sdk::program_pack::Pack;
 // 0.2.23


// Create the runtime
use client::execute::process::Arbitrager;





use serde::{Deserialize};

use std::sync::{Arc, Mutex};


use {
    clap::{Parser, ValueEnum},
    futures::{sink::SinkExt, stream::StreamExt},
    log::{error},
    std::{
        collections::{HashMap},
    },
   
};
use {
    futures::{future::TryFutureExt},
    yellowstone_grpc_client::{GeyserGrpcClient},
    yellowstone_grpc_proto::{
        prelude::{
            subscribe_update::UpdateOneof, CommitmentLevel, SubscribeRequest, SubscribeRequestFilterAccounts,
            SubscribeRequestFilterAccountsFilter, SubscribeUpdateAccount,
        },
    },
};

#[derive(Debug)]
#[allow(dead_code)]
pub struct AccountPretty {
    is_startup: bool,
    slot: u64,
    pubkey: Pubkey,
    lamports: u64,
    owner: Pubkey,
    executable: bool,
    rent_epoch: u64,
    data: Vec<u8>,
    write_version: u64,
    txn_signature: String,
}

impl From<SubscribeUpdateAccount> for AccountPretty {
    fn from(
        SubscribeUpdateAccount {
            is_startup,
            slot,
            account,
        }: SubscribeUpdateAccount,
    ) -> Self {
        let account = account.expect("should be defined");
        Self {
            is_startup,
            slot,
            pubkey: Pubkey::try_from(account.pubkey).expect("valid pubkey"),
            lamports: account.lamports,
            owner: Pubkey::try_from(account.owner).expect("valid pubkey"),
            executable: account.executable,
            rent_epoch: account.rent_epoch,
            data: (account.data),
            write_version: account.write_version,
            txn_signature: bs58::encode(account.txn_signature.unwrap_or_default()).into_string(),
        }
    }
}













use solana_sdk::transaction::{VersionedTransaction};





use anchor_client::{Client, Cluster};

use std::collections::{ HashSet};
use std::fmt::Debug;

use std::rc::Rc;
use std::str::FromStr;

use std::borrow::{Borrow, BorrowMut};
use std::vec;


use solana_sdk::account::Account;

type ShardedDb = Arc<Mutex<HashMap<String, Arc<Mutex<PoolQuote>>>>>;

use client::constants::*;
use client::monitor::pools::pool::{pool_factory, PoolDir, PoolOperations, PoolType, self};
// use spl_token spl_token::state::Account::unpack


use client::utils::{
    derive_token_address, read_json_dir, PoolEdge, PoolGraph, PoolIndex, PoolQuote,
};

const TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("my_data");
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(short, long)]
    pub cluster: String,
    #[clap(short, long)]
    pub flashing: String,
    #[clap(short, long)]
    pub keypair: String,
}

fn add_pool_to_graph<'a>(
    graph: &mut PoolGraph,
    idx0: PoolIndex,
    idx1: PoolIndex,
    quote: &PoolQuote,
) {
    // idx0 = A, idx1 = B
    let edges = graph
        .0
        .entry(idx0)
        .or_insert_with(|| PoolEdge(HashMap::new()));
    let quotes = edges.0.entry(idx1).or_default();
    quotes.push(quote.clone());
}

pub async    fn get_arbitrage_instructions<'a>(
    token_mints: &Vec<Pubkey>,
    src_mint: Pubkey,
        swap_start_amount: u128,
        mint_idxs: &Vec<usize>,
        pools: &Vec<PoolQuote>,    ) -> (Vec<Vec<Instruction>>, bool) {
            
    let owner3 = Arc::new(read_keypair_file("/root/.config/solana/id.json".clone()).unwrap());

    let owner = owner3.try_pubkey().unwrap();
        // gather swap ixs
        let mut ixs = vec![];
        let swap_state_pda =
            (Pubkey::from_str("8cjtn4GEw6eVhZ9r1YatfiU65aDEBf1Fof5sTuuH6yVM").unwrap());
        let src_ata = derive_token_address(&owner, &src_mint);


    // setup anchor things
    
    let provider = anchor_client::Client::new_with_options(
        Cluster::Mainnet,
        owner3.clone(),
        solana_sdk::commitment_config::CommitmentConfig::confirmed(),
    );
    let program = provider.program(*ARB_PROGRAM_ID).unwrap();

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
            let [mint0, mint1] = [token_mints[mint_idx0], token_mints[mint_idx1]];
            let pool = &pools[i];
            let mut swap_ix = pool
                .0
                .swap_ix(&mint0, &mint1, swap_start_amount);

            swap_start_amount = pool.0.get_quote_with_amounts_scaled(
                swap_start_amount,
                &mint0,
                &mint1);
                
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


async fn yellowstone( mut og_pools: &mut  Vec<Box< dyn PoolOperations>>,
    arbitrager: Arc<Arbitrager>,
    connection_url: &str,
     accounts: HashMap<String, SubscribeRequestFilterAccounts>,
     owner: Arc<Keypair>,
     start_mint_idx: usize
     ) -> Result<(), Box<dyn std::error::Error>> {

        let token_mints = arbitrager.clone().token_mints.clone();
                        
        let connection = RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::confirmed());

        let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();

        let start_mint: Pubkey = usdc_mint;
        
    let _min_swap_amount = 10_u128.pow(3_u32); // scaled! -- 1 USDC
    let _owner_kp_path = "/root/.config/solana/id.json";
    let rc_owner = owner;
    let src_ata = derive_token_address(&rc_owner.pubkey(), &start_mint);

let rc_owner_signer: &dyn solana_sdk::signature::Signer = &*rc_owner;
let signers = [rc_owner_signer];
    let init_token_acc = connection.get_account(&src_ata).await.unwrap();
    let init_token_balance: u128 = spl_token::state::Account::unpack(&init_token_acc.data).unwrap().amount as u128;
    let swap_start_amount = init_token_balance; // scaled!
    println!("swap start amount = {}", swap_start_amount); // track what arbs we did with a larger size
    let _init_token_acc = connection.get_account(&src_ata).await.unwrap();

    println!("searching for arbitrages...");
    let _min_swap_amount = 10_u128.pow(4_u32); // scaled! -- 1 USDC
    println!("searching for arbitrages2...");
    let mut client = GeyserGrpcClient::connect("https://jarrett-solana-7ba9.mainnet.rpcpool.com", Some("8d890735-edf2-4a75-af84-92f7c9e31718".to_string()), None).unwrap();
    println!("searching for arbitrages3...");
    let (mut subscribe_tx, mut stream) = client.subscribe().await.unwrap();
        println!("searching for arbitrages4...");
    let commitment: CommitmentLevel = CommitmentLevel::Confirmed;
    subscribe_tx
        .send(SubscribeRequest {
            slots: HashMap::new(),
            accounts,
            transactions: HashMap::new(),
            entry: HashMap::new(),
            blocks: HashMap::new(),
            blocks_meta: HashMap::new(),
            commitment: Some(commitment as i32),
            accounts_data_slice: vec![],
            ping: None,
        })
        .await.unwrap();
        println!("searching for arbitrages5...");
    
        let mut newfuts = vec![];

    while let Some(message) = stream.next().await {
        match message {
            Ok(msg) => {
                match msg.update_oneof {
                    Some(UpdateOneof::Account(account)) => {
                        let account: AccountPretty = account.into();
                        for pool in og_pools.iter_mut() {
                            let update_accounts = pool.get_update_accounts();
                            for update_account in update_accounts {
                                if update_account == account.pubkey {
                                    println!("found update account {} with data {:?}", update_account, account.data.len());
                                    pool.set_update_accounts2(account.pubkey, &account.data, Cluster::Mainnet);
                                }
                            }
                        }



                        let a = arbitrager.clone();
                        let token_mints = a.token_mints.clone();
                        let rc_owner = rc_owner.clone();
                        let connection = RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::confirmed());
                                let mut arbs = vec![];
                                    println!("searching for arbitrages6...");
                          newfuts.push(a.brute_force_search(
                            start_mint_idx,
                            init_token_balance,
                            init_token_balance,
                            swap_start_amount,
                            vec![start_mint_idx],
                            vec![start_mint_idx],
                            vec![],
                            vec![],
                           // 0
                        ).await);
                        if newfuts.len() > 70 {
                            let results = join_all(newfuts).await;
                            for result in results {

                                if result.is_err() {
                                    continue;
                                }
                                let arb = result.unwrap();
                                if arb.is_none() {
                                    continue;
                                }
                                let arb = arb.unwrap();
                                arbs.push(arb);
                            }

                            newfuts = vec![];
                        }
                        println!("searching for arbitrages7...");
                     println!("there are {} arbs", arbs.len());
                             
                         let mut arb_paths = vec![];
                         let mut arb_pools = vec![];
                         let arb_amounts = arbs.iter()
                         .map(|arb| {
                             let arb_amount = arb.0;
                             arb_paths.push(arb.1.clone());
                             arb_pools.push(arb.2.clone());
                             arb_amount
                         })
                         .collect::<Vec<u128>>();
                         let arb_amounts = arb_amounts.clone();
                         // find largest
                         let mut largest = 0;
                         let mut largest_idx = 0;
                         for (i, arb_amount) in arb_amounts.iter().enumerate() {
                             if *arb_amount > largest {
                                 largest = *arb_amount;
                                 largest_idx = i;
                             }
                         }
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
                    
                     let ixs: (Vec<Vec<solana_program::instruction::Instruction>>, bool) = get_arbitrage_instructions(
                        &token_mints.clone(),
                        usdc_mint,
                        init_token_balance,
                         &arb_path,
                         &arb_pools,
                     ).await;
                    let mut ixs =  ixs.0.concat();
                    let _hydra_ata = derive_token_address(&Pubkey::from_str("2bxwkKqwzkvwUqj3xYs4Rpmo1ncPcA1TedAPzTXN1yHu").unwrap(), &usdc_mint);
                    let ix = spl_token::instruction::transfer(
                     &spl_token::id(),
                     &src_ata,
                     &src_ata,
                     &rc_owner.pubkey(),
                     &[  
                     ],
                     init_token_balance as u64,
                    ).unwrap();
                    ixs.push(ix);
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
                    &connection).await;
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
                    let mut lutties: Vec<AddressLookupTableAccount> = get_address_lookup_table_accounts(&connection, lutties.clone(), rc_owner.clone().pubkey()).await;
                    
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
                    ).await.unwrap();
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
                    arbs = vec![];
                    let blockhash = connection.get_latest_blockhash().await.unwrap();

                    let rc_owner_signer: &dyn solana_sdk::signature::Signer = &*rc_owner;
                    let signers = [rc_owner_signer];
                       
                        
                                let tx = VersionedTransaction::try_new( VersionedMessage::V0(v0::Message::try_compile(
                                    &rc_owner.pubkey(),
                                    &ixs,
                                    &lutties,
                                    blockhash,
                                    ).unwrap()), &signers);
                                if tx.is_err() {
                                    continue;
                                }
                                let tx = tx.unwrap();
                                let connection = RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::confirmed());
                                let sig = connection.send_transaction(&tx).await;
                                if sig.is_err() {
                                    continue;
                                }
                                let sig = sig.unwrap();
                                println!("sent tx: {:?}", sig);
                                }

                    _ => {}

                }
            }
            Err(error) => {
                error!("stream error: {error:?}");
                break;
            }
        }
            
            
    }

Ok(())
}
#[tokio::main(worker_threads = 70)]

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut page_config = ShardedDb::new(Mutex::new(HashMap::new()));

    let args = Args::parse();
    let cluster = match args.cluster.as_str() {
        "localnet" => Cluster::Localnet,
        "mainnet" => Cluster::Mainnet,
        _ => panic!("invalid cluster type"),
    };
    // ** setup RPC connection
    let connection_url = "https://jarrett-solana-7ba9.mainnet.rpcpool.com/8d890735-edf2-4a75-af84-92f7c9e31718";
    println!("using connection: {}", connection_url);

    let connection = Arc::new(RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::confirmed()));

    let owner_kp_path = "/root/.config/solana/id.json";

    // setup anchor things
    let owner = read_keypair_file(owner_kp_path).unwrap();
    let rc_owner = Rc::new(owner);

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
    let owner2 = read_keypair_file(owner_kp_path).unwrap();
    let _rc_owner2 = Rc::new(owner2);
    let provider = Client::new_with_options(
        cluster.clone(),
        rc_owner.clone(),
        CommitmentConfig::confirmed(),
    );
    let _program = provider.program(*ARB_PROGRAM_ID);

    // ** define pool JSONs
    let mut pool_dirs: Vec<PoolDir> = vec![];

    let orca_dir = PoolDir {
        pool_type: PoolType::OrcaPoolType,
        dir_path: "../pools/orca".to_string(),
    };
    pool_dirs.push(orca_dir);
    let saber_dir = PoolDir {
        pool_type: PoolType::SaberPoolType,
        dir_path: "../pools/saber/".to_string(),
    };
    pool_dirs.push(saber_dir);

    let r_dir = PoolDir {
        pool_type: PoolType::RaydiumPoolType,
        dir_path: "../pools/raydium".to_string(),
    };
    pool_dirs.push(r_dir);
    let serum_dir = PoolDir {
        pool_type: PoolType::SerumPoolType,
        dir_path: "../pools/openbook/".to_string(),
    };
    pool_dirs.push(serum_dir);

    // ** json pool -> pool object
    let mut token_mints = vec![];
    let mut pools = vec![];

    let mut update_pks = vec![];
    let mut all_mint_idxs = vec![];

    let mut mint2idx = HashMap::new();
    let mut graph_edges = vec![];

    println!("setting up exchange graph...");
    let mut graph = PoolGraph::new();
    let _pool_count = 0;
    let _account_ptr = 0;
    println!("extracting pool + mints...");
    let mut update_accounts: Vec<Option<Account>> = vec![];
    let mut tuas = vec![];
    let mut tupdate_pks = vec![];
    for pool_dir in pool_dirs.clone() {
        debug!("pool dir: {:#?}", pool_dir);
        let pool_paths = read_json_dir(&pool_dir.dir_path);
       
        for pool_path in &mut pool_paths.clone() {
            let cluster = cluster.clone();
            let connection = connection.clone();

            let json_str = std::fs::read_to_string(&pool_path).unwrap();
            let mut pool = pool_factory(&pool_dir.pool_type, &json_str);

           
            let uas = pool.get_update_accounts();
            tupdate_pks.extend(uas.clone());
            tuas.extend(uas.clone());
        }

    }
    println!("getting update accounts... tuas length is {:?}", tuas.len());
    let mut futures = vec![];
    for chunk in tuas.chunks(100) {

        let fut = program_async.get_multiple_accounts_with_commitment(&chunk, CommitmentConfig::confirmed());
        futures.push(fut);
        if futures.len() > 100 {
            println!("futures length is {:?} update_accounts length is {:?}", futures.len(), update_accounts.len());
                
            let results = join_all(futures).await;
            futures = vec![];
            for result in results {
                if result.is_err() {
                    continue;
                }
                let account_infos = result.unwrap().value;
                for i in 0..account_infos.len(){
                    if account_infos[i].is_none() {
                        continue;
                    }
                    let account = account_infos[i].clone().unwrap();
                    update_accounts.push(Some(account));
                    let update_pk = tupdate_pks[i].clone();
                    update_pks.push(update_pk);
                }
                futures = vec![];
            }
        }
    }
    
            let results = join_all(futures).await;
            futures = vec![];
            for result in results {
                if result.is_err() {
                    continue;
                }
                let account_infos = result.unwrap().value;
                for i in 0..account_infos.len(){
                    if account_infos[i].is_none() {
                        continue;
                    }
                    let account = account_infos[i].clone().unwrap();
                    update_accounts.push(Some(account));
                    let update_pk = tupdate_pks[i].clone();
                    update_pks.push(update_pk);
                }
                futures = vec![];
            }
    for pool_dir in pool_dirs.clone() {
        debug!("pool dir: {:#?}", pool_dir);
        let pool_paths = read_json_dir(&pool_dir.dir_path);
        for pool_path in &mut pool_paths.clone() {
            let cluster = cluster.clone();
            let connection = connection.clone();

            let json_str = std::fs::read_to_string(&pool_path).unwrap();
            let mut pool = pool_factory(&pool_dir.pool_type, &json_str);

            let pool_mints = pool.get_mints();
            if pool_mints.len() != 2 {
                // only support 2 mint pools
                println!("skipping pool with mints != 2: {:?}", pool_path);
                continue;
            }
            
            let uas = pool.get_update_accounts();
            let mut index = 0;
            let mut updateuauas = vec![];
            let mut updatepkpks = vec![];
            for update_pk in update_pks.clone() {
                if uas.contains(&update_pk) {
                    let update_account = update_accounts[index].clone();
                    updatepkpks.push(update_pk);
                    updateuauas.push(update_account);
                }
                index += 1;
            }
            if updateuauas.len() != uas.len() {
               
            }
            else {
                
                pool.set_update_accounts(updateuauas.clone(), cluster.clone());
                let mut index = 0;
                for data in updateuauas {
                    if data.is_none() {
                        continue;
                    }
                    let data = data.unwrap();
                    let pubkey = updatepkpks[index];
                pool.set_update_accounts2(pubkey, &data.clone().data, cluster.clone());
                index += 1;
                }
            }
            if !pool.can_trade(&pool_mints[0], &pool_mints[1]) {
                println!("skipping pool can't trade {:?} ", pool_path);
                continue;
            }
            // ** record pool
            pools.push( pool.clone());
            let locked_pc = page_config.try_lock();
            if locked_pc.is_err() {
                continue;
            }
            let mut pc = locked_pc.unwrap();
            let pool_quote = PoolQuote::new(pool.clone().into());
            let pool_quote = Arc::new(Mutex::new(pool_quote));

            pc.insert(
                pool.get_own_addr().to_string(),
                pool_quote.clone(),
            );
            drop(pc)
        }
    }

    for pool in pools.clone() { 

        let mints = pool.get_mints();
        
            //  ** record pool println for graph
            // token: (mint = graph idx), (addr = get quote amount)
            let mut mint_idxs = vec![];
            for mint in mints.clone() {
                let idx;
                if !token_mints.contains(&mint) {
                    idx = token_mints.len();
                    mint2idx.insert(mint, idx);
                    token_mints.push(mint);
                    // graph_edges[idx] will always exist :)
                    graph_edges.push(HashSet::new());
                } else {
                    idx = *mint2idx.get(&mint).unwrap();
                }
                mint_idxs.push(idx);
            }
        let mint0_idx = *mint2idx.get(&mints.clone()[0]).unwrap();
        let mint1_idx = *mint2idx.get(&mints[1]).unwrap();
        let idx0: PoolIndex = PoolIndex(mint0_idx);
        let idx1: PoolIndex = PoolIndex(mint1_idx);
        let mut pool_ptr = PoolQuote::new(pool.clone().into());
        add_pool_to_graph(&mut graph, idx0, idx1, &mut pool_ptr.clone());
        add_pool_to_graph(&mut graph, idx1, idx0, &mut pool_ptr);

        all_mint_idxs.push(mint0_idx);
        all_mint_idxs.push(mint1_idx);

        // record graph edges if they dont already exist
        if !graph_edges[mint0_idx].contains(&mint1_idx) {
            graph_edges[mint0_idx].insert(mint1_idx);
        }
        if !graph_edges[mint1_idx].contains(&mint0_idx) {
            graph_edges[mint1_idx].insert(mint0_idx);
        }

        
    }
    // !
    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
    let start_mint = usdc_mint;
    let start_mint_idx = *mint2idx.get(&usdc_mint).unwrap();

    let owner: &Keypair = rc_owner.borrow();
    let _src_ata = derive_token_address(&owner.pubkey(), &start_mint);

    // slide it in there
    println!("getting pool amounts...");



        println!("added {:?} update accounts", update_accounts.clone().len());

        let src_ata = derive_token_address(&rc_owner.pubkey(), &usdc_mint);

        let connection = Arc::new(RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::confirmed()));

        let init_token_acc = connection.clone().get_account(&src_ata).await.unwrap();
        let init_token_balance: u128 = spl_token::state::Account::unpack(&init_token_acc.data).unwrap().amount as u128;
    println!("update accounts is {:?}", update_accounts.len());
    // slide it out here
    println!(
        "init token acc: {:?}, balance: {:#}",
        init_token_acc, init_token_balance
    );
    println!("starting balance = {}", init_token_balance);

    update_accounts = update_accounts
        .into_iter()
        .filter(|maybe| maybe.is_some())
        .collect::<Vec<Option<Account>>>();
    let mut filter_map: HashMap<String, SubscribeRequestFilterAccounts> = HashMap::new();
    let update_accounts_chunks: std::slice::Chunks<'_, Option<Account>> = update_accounts.chunks(2666);
    let owners = update_accounts_chunks.clone().map(|chunk| {
        chunk
            .iter()
            .map(|maybe| {
                let account = maybe.as_ref().unwrap();
                account.owner.to_string()
            })
            .collect::<Vec<String>>()
    });
    let mut anindex = 0;
    for (i, chunk) in owners.enumerate() {
        let filters: Vec<SubscribeRequestFilterAccountsFilter> = vec![];
        let mut end = anindex + 100;
        if end > update_pks.len() {
            end = update_pks.len();
        }
        println!("chunk is {:?}", chunk.len());
        filter_map.insert(
            i.to_string(),
            
            SubscribeRequestFilterAccounts {
                account: update_pks[anindex..end].to_vec()
                .iter()
                .map(|maybe| {
                    maybe.to_string()
                })
                .collect::<Vec<String>>(),
                owner: chunk.to_vec(),
                filters
            },
        );
        anindex += 100;
    }
    println!("filter map is {:?}", filter_map.len());
    println!("searching for arbitrages...");
    let _min_swap_amount = 10_u128.pow(3_u32); // scaled! -- 1 USDC

    let src_ata = derive_token_address(&rc_owner.pubkey(), &usdc_mint);

    let init_token_acc = connection.clone().get_account(&src_ata).await.unwrap();
    let init_token_balance: u128 = spl_token::state::Account::unpack(&init_token_acc.data).unwrap().amount as u128;
    let swap_start_amount = init_token_balance; // scaled!
    println!("swap start amount = {}", swap_start_amount); // track what arbs we did with a larger size
    let init_token_acc = connection.clone().get_account(&src_ata).await.unwrap();
    let init_token_balance: u128 = spl_token::state::Account::unpack(&init_token_acc.data).unwrap().amount as u128;

    
    println!("searching for arbitrages...");
    let _min_swap_amount = 10_u128.pow(4_u32); // scaled! -- 1 USDC
    let swap_start_amount = init_token_balance; // scaled!
    println!("swap start amount = {}", swap_start_amount);
    // Spawn as tokio task
  
    // create Arbitrager and iterate loops in 60 futures at a time
    
    let arbitrager =Arbitrager {
        token_mints,
        graph_edges,
        graph,
        cluster: Cluster::Custom(
            connection_url.to_string(),
            connection_url.to_string()),
        connection
    };
        let owner = Arc::new(read_keypair_file(owner_kp_path).unwrap());
      
        yellowstone(&mut pools.clone(), Arc::new(arbitrager), connection_url, filter_map, owner, start_mint_idx)
        .await
        .unwrap()
        ;
       
Ok(())
   
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

async fn create_and_or_extend_luts(
    remaining_public_keys: &Vec<Pubkey>,
    connection: &RpcClient,
    luts: &mut Vec<AddressLookupTableAccount>,
    payer: &Keypair,
) -> Result<Vec<AddressLookupTableAccount>, Box<dyn std::error::Error>> {
    let mut used_luts = Vec::new();

    for pubkeys in remaining_public_keys.chunks(25) {
        let (lut, _index) = find_or_create_lut(connection, payer, luts, remaining_public_keys.len()).await?;
            let extend_ix = extend_lookup_table(
                lut.key,
                payer.pubkey(),
                Some(payer.pubkey()),
                pubkeys.to_vec(),
            );
            let latest_blockhash = connection.get_latest_blockhash().await.unwrap(); 
            //println!("extending lut: {:?}", lut.key);
           let hm = connection
                .send_transaction(&VersionedTransaction::try_new(
                        VersionedMessage::V0(v0::Message::try_compile(
                            &payer.pubkey(),
                            &[extend_ix],
                            &[],
                            latest_blockhash,
                        ).unwrap()),
                        &[payer],
                    ).unwrap()
                ).await;
                if !hm.is_err() {
                    let signature = hm.unwrap();
                    
                }
                

                    
            used_luts.push(lut);
        }

    Ok(used_luts)
}
async fn find_or_create_lut(
    connection:  &RpcClient,
    payer: &Keypair,
    luts: &mut Vec<AddressLookupTableAccount>,
    howmany: usize
) -> Result<(AddressLookupTableAccount, usize), Box<dyn std::error::Error>> {
    luts.shuffle(&mut rand::thread_rng());
    for (index, lut) in luts.iter().enumerate() {
        let acc = connection.get_account(&lut.key).await.unwrap();
        let address_lookup_table = AddressLookupTable::deserialize(&acc.data).unwrap();
        //println!("{}, {}", lut.addresses.len(), address_lookup_table.meta.authority.unwrap() == payer.pubkey());
        if lut.addresses.len() < (255_usize -howmany) && address_lookup_table.meta.authority.unwrap() == payer.pubkey() {
            return Ok((lut.clone(), index));
        }
    }
    Ok((create_new_lut(connection, payer).await.unwrap(), luts.len()))
}

async fn create_new_lut(
    connection: &RpcClient,
    payer: &Keypair,
) -> Result<AddressLookupTableAccount, Box<dyn std::error::Error>> {
    // Create a new AddressLookupTable
    let recent_slot = connection
    .get_slot_with_commitment(CommitmentConfig::confirmed()).await
    .unwrap()//"237009123 is not a recent slot"
    - 50;
    let (create_ix, table_pk) =
        create_lookup_table(
            payer.pubkey(),
            payer.pubkey(),
            recent_slot,
        );
    let latest_blockhash = connection.get_latest_blockhash().await.unwrap();  
    
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
    ).await;
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
pub async fn calculate_recent_fee(
    write_locked_accounts: &[Pubkey],
    connection: &RpcClient
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
            ).await.unwrap().value;
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
            ).await.unwrap_or_default();
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

async fn get_address_lookup_table_accounts(client: &RpcClient, keys: Vec<String>, payer: Pubkey) -> Vec<AddressLookupTableAccount> {
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
            let raw_accounts = client.get_multiple_accounts(chunk).await.unwrap();

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