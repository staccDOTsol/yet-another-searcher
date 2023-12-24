use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::read_keypair_file;
use anchor_client::solana_sdk::signature::{Keypair, Signer};
use rand::{seq::SliceRandom};

use base64::engine::{GeneralPurpose, general_purpose};
use futures::future::join_all;
use log::{debug, warn};
use solana_address_lookup_table_program::instruction::{extend_lookup_table, create_lookup_table};
use solana_address_lookup_table_program::state::AddressLookupTable;
use solana_program::address_lookup_table_account::AddressLookupTableAccount;
use solana_program::message::{VersionedMessage, v0};
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use core::panic;
use redb::{Database, Error, ReadableTable, TableDefinition};
use solana_sdk::program_pack::Pack;
use tokio::runtime::Runtime; // 0.2.23
use tokio::sync::mpsc;

// Create the runtime
use client::execute::process::Arbitrager;

use std::fs::File;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use serde;
use serde::{Deserialize, Serialize};

use std::sync::{Arc, Mutex};


use {
    chrono::{DateTime, NaiveDateTime, Utc},
    clap::{Parser, ValueEnum},
    futures::{sink::SinkExt, stream::StreamExt},
    log::{error, info},
    maplit::hashmap,
    solana_sdk::signature::Signature,
    std::{
        collections::{BTreeMap, HashMap},
        env,
    },
   
};
use {
    futures::{future::TryFutureExt},
    solana_transaction_status::{EncodedTransactionWithStatusMeta, UiTransactionEncoding},
    std::{
        fmt,
        time::Duration,
    },
    yellowstone_grpc_client::{GeyserGrpcClient, GeyserGrpcClientError},
    yellowstone_grpc_proto::{
        prelude::{
            subscribe_request_filter_accounts_filter::Filter as AccountsFilterDataOneof,
            subscribe_request_filter_accounts_filter_memcmp::Data as AccountsFilterMemcmpOneof,
            subscribe_update::UpdateOneof, CommitmentLevel, SubscribeRequest,
            SubscribeRequestAccountsDataSlice, SubscribeRequestFilterAccounts,
            SubscribeRequestFilterAccountsFilter, SubscribeRequestFilterAccountsFilterMemcmp,
            SubscribeRequestFilterBlocks, SubscribeRequestFilterBlocksMeta,
            SubscribeRequestFilterEntry, SubscribeRequestFilterSlots,
            SubscribeRequestFilterTransactions, SubscribeRequestPing, SubscribeUpdateAccount,
            SubscribeUpdateTransaction,
        },
        tonic::service::Interceptor,
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


use axum::routing::post;
use axum::{
    body,
    extract::{Extension, Json, Path, Query},
    http::{header, HeaderMap, HeaderValue, StatusCode, Uri},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use axum_server::Handle;
use std::net::ToSocketAddrs;
use std::path::PathBuf;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    set_header::SetResponseHeaderLayer,
};

use derive_more::FromStr;
use num_traits::pow;
use solana_client::client_error::ClientError;
use solana_sdk::signers::Signers;
use solana_sdk::transaction::{Transaction, VersionedTransaction};
use structopt::StructOpt;

use flash_loan_sdk::instruction::{flash_borrow, flash_repay};
use flash_loan_sdk::{available_liquidity, flash_loan_fee, get_reserve, FLASH_LOAN_ID};

use anchor_client::{Client, Cluster};

use std::collections::{ HashSet};
use std::fmt::Debug;
use std::io::Write;
use std::rc::Rc;
use std::str::FromStr;

use std::borrow::Borrow;
use std::vec;


use solana_sdk::account::Account;

type ShardedDb = Arc<Mutex<HashMap<String, Account>>>;
use client::execute::*;
use client::constants::*;
use client::monitor::pools::pool::{pool_factory, PoolDir, PoolOperations, PoolType};
// use spl_token spl_token::state::Account::unpack
use spl_token::state::Account as TokenAccount;

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
    let quotes = edges.0.entry(idx1).or_insert_with(|| vec![]);
    quotes.push(quote.clone());
}

async fn yellowstone(og_pools: &mut Vec<Box<dyn PoolOperations>>,
    arbitrager: Arc<Arbitrager>,
    connection_url: &str,
     accounts: HashMap<String, SubscribeRequestFilterAccounts>,
     owner: Arc<Keypair>,
     start_mint_idx: usize
     ) -> Option<u128> {


        let connection = RpcClient::new_with_commitment(connection_url, CommitmentConfig::recent());

        let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();

        let start_mint: Pubkey = usdc_mint;
        
    let min_swap_amount = 10_u128.pow(3_u32); // scaled! -- 1 USDC
    let owner_kp_path = "/root/.config/solana/id.json";
    let rc_owner = owner;
    let src_ata = derive_token_address(&rc_owner.pubkey(), &start_mint);
    let init_token_acc = connection.get_account(&src_ata).unwrap();
    let init_token_balance: u128 = spl_token::state::Account::unpack(&init_token_acc.data).unwrap().amount as u128;
    let mut swap_start_amount = init_token_balance; // scaled!
    println!("swap start amount = {}", swap_start_amount); // track what arbs we did with a larger size
    let init_token_acc = connection.get_account(&src_ata).unwrap();

    println!("searching for arbitrages...");
    let min_swap_amount = 10_u128.pow(4_u32); // scaled! -- 1 USDC
    let mut client = GeyserGrpcClient::connect("https://jarrett-solana-7ba9.mainnet.rpcpool.com", Some("8d890735-edf2-4a75-af84-92f7c9e31718".to_string()), None).unwrap();
    let (mut subscribe_tx, mut stream) = client.subscribe().await.unwrap();

    let commitment: CommitmentLevel = CommitmentLevel::Processed.into();
    subscribe_tx
        .send(SubscribeRequest {
            slots: HashMap::new(),
            accounts: accounts,
            transactions: HashMap::new(),
            entry: HashMap::new(),
            blocks: HashMap::new(),
            blocks_meta: HashMap::new(),
            commitment: Some(commitment as i32),
            accounts_data_slice: vec![],
            ping: None,
        })
        .await.unwrap();
    let mut arbs = vec![];
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
                                pool.set_update_accounts2(account.pubkey, &account.data, Cluster::Mainnet);
                            }
                        }
                       }
                       let mut arbitrager = arbitrager.clone();
                       let mut arb = arbitrager.brute_force_search(
                           start_mint_idx,
                           init_token_balance,
                           swap_start_amount,
                           vec![start_mint_idx],
                           vec![],
                          // 0
                       );
                          if arb.is_ok() {
                            let arb = arb.unwrap();
                            if arb.is_some() {
                               let arb = arb.unwrap();
                            arbs.push(arb);
                            println!("arbs: {:?}", arbs.len());
                            if arbs.len() >= 1 {
                                let mut arb_paths = vec![];
                                let mut arb_pools = vec![];
                                let arb_amounts = arbs.iter()
                                .map(|arb| {
                                    let mut arb_amount = arb.0;
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
                                let arb_amount = arb_amounts[largest_idx].clone();
                                println!("lorgest arb path is {:?}", arb_path);
                                println!("lorgest arb pools is {:?}", arb_pools);
                                println!("lorgest arb amount is {:?}", arb_amount);

                                let mint_keys: Vec<String> =
                                arb_path.clone().iter_mut().map(|i| i.to_string()).collect();
                            let pool_keys: Vec<String> =
                            arb_pools.iter().map(|p| p.0.get_name()).collect();
                            let arb_key = format!("{}{}", mint_keys.join(""), pool_keys.join(""));
                            //println!("arbkey: {:?}", arb_key);/* 
                            let mut ixs: (Vec<Vec<solana_program::instruction::Instruction>>, bool) = arbitrager.get_arbitrage_instructions(
                                init_token_balance,
                                &arb_path,
                                &arb_pools,
                            ).await;
                          let mut ixs =  ixs.0.concat();
                          let hydra_ata = derive_token_address(&Pubkey::from_str("2bxwkKqwzkvwUqj3xYs4Rpmo1ncPcA1TedAPzTXN1yHu").unwrap(), &usdc_mint);
                          let ix = spl_token::instruction::transfer(
                            &spl_token::id(),
                            &src_ata,
                            &hydra_ata,
                            &rc_owner.pubkey(),
                            &[  
                            ],
                            init_token_balance as u64 / 10 as u64,
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
        &connection);
        //println!("recent fees: {:?}", recent_fees);

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
            let mut lutties: Vec<AddressLookupTableAccount> = get_address_lookup_table_accounts(&connection, lutties.clone(), rc_owner.clone().pubkey());

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
        let rounded = round::round(usized_lutties.len() as f64 / 4.0, 0) as usize;
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
                let rc_owner_signer: &dyn solana_sdk::signature::Signer = &*rc_owner;
                let signers = &[rc_owner_signer];
                let blockhash = connection.get_latest_blockhash().unwrap();
                let tx = VersionedTransaction::try_new(
                    VersionedMessage::V0(v0::Message::try_compile(
                        &rc_owner.pubkey(),
                        &ixs,
                        &lutties,
                        blockhash,
                    ).unwrap()),
                    signers
                ).unwrap();

                    let signature = connection
                        .send_transaction_with_config(
                            &tx,
                            solana_client::rpc_config::RpcSendTransactionConfig {
                                skip_preflight: false,
                                ..solana_client::rpc_config::RpcSendTransactionConfig::default()
                            }, 
                        )
                        ;
                        if signature.is_ok() {
                            println!("https://solscan.io/tx/{}", signature.unwrap());
                        }
                        else {
                            println!("error: {:?}", signature.err().unwrap());
                        }
                
    
                                arbs = vec![];
                    }
                            }
                          }

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
    None 

}
#[tokio::main(worker_threads = 70)]

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut page_config = Arc::new(ShardedDb::new(Mutex::new(HashMap::new())));

    let args = Args::parse();
    let cluster = match args.cluster.as_str() {
        "localnet" => Cluster::Localnet,
        "mainnet" => Cluster::Mainnet,
        _ => panic!("invalid cluster type"),
    };
    // ** setup RPC connection
    let connection_url = "https://jarrett-solana-7ba9.mainnet.rpcpool.com/8d890735-edf2-4a75-af84-92f7c9e31718";
    println!("using connection: {}", connection_url);

    let connection = Arc::new(RpcClient::new_with_commitment(connection_url, CommitmentConfig::recent()));

    let owner_kp_path = "/root/.config/solana/id.json";

    // setup anchor things
    let owner = read_keypair_file(owner_kp_path.clone()).unwrap();
    let rc_owner = Rc::new(owner);
    // setup anchor things
    let owner2 = read_keypair_file(owner_kp_path.clone()).unwrap();
    let rc_owner2 = Rc::new(owner2);
    let provider = Client::new_with_options(
        cluster.clone(),
        rc_owner.clone(),
        CommitmentConfig::recent(),
    );
    let program = provider.program(*ARB_PROGRAM_ID);

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
    let mut update_pks_lengths = vec![];
    let mut all_mint_idxs = vec![];

    let mut mint2idx = HashMap::new();
    let mut graph_edges = vec![];

    println!("setting up exchange graph...");
    let mut graph = PoolGraph::new();
    let mut pool_count = 0;
    let mut account_ptr = 0;
    println!("extracting pool + mints...");
    let mut update_accounts: Vec<Option<Account>> = vec![];
    let mut futures = vec![];
    for pool_dir in pool_dirs {
        debug!("pool dir: {:#?}", pool_dir);
        let mut pool_paths = read_json_dir(&pool_dir.dir_path);
        let mut max = 11111;
        if pool_paths.len() < 11111 {
            max = pool_paths.len();
        }
        for pool_path in &mut pool_paths[0..max] {
            let cluster = cluster.clone();
            let connection = connection.clone();
            let pool_factory = pool_factory.clone();

            let json_str = std::fs::read_to_string(&pool_path).unwrap();
            let mut pool = pool_factory(&pool_dir.pool_type, &json_str);

            let pool_mints = pool.get_mints();
            if pool_mints.len() != 2 {
                // only support 2 mint pools
                println!("skipping pool with mints != 2: {:?}", pool_path);
                continue;
            }

            //  ** record pool println for graph
            // token: (mint = graph idx), (addr = get quote amount)
            let mut mint_idxs = vec![];
            for mint in pool_mints {
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
            let mut all_mint_idxs = all_mint_idxs.clone();
            let mut graph_edges = graph_edges.clone();
            let mut graph = graph.clone();
            futures.push(tokio::spawn(async move {
            // get accounts which need account println to be updated (e.g. pool src/dst amounts for xy=k)
            let uas = pool.get_update_accounts();
            let account_infos = connection.get_multiple_accounts(&uas).unwrap();
            
            pool.set_update_accounts(account_infos.clone(), cluster.clone());
 let mints = pool.get_mints();
            if !pool.can_trade(&mints[0], &mints[1]) {
                println!("can't trade {:?}", pool.get_own_addr());
                return None
            }
            pool_count += 1;
            Some((pool,account_infos.clone(), uas))
        }));
        if futures.len() > 70 {
            let results = join_all(futures).await;
            for result in results {
                if result.is_err() {
                    continue;
                }
                if let Some(pool) = result.unwrap() {
                    pools.push(pool.0);
                    update_accounts.extend(pool.1);
                    let uas = pool.2;
                    update_pks_lengths.push(uas.clone().len());
                    update_pks.extend(uas.clone());
                    println!("added pool {:?}", pools.len());
                }
            }
            
            futures = vec![];
        }
    }
}

    if futures.len() > 70 {
        let results = join_all(futures).await;
        for result in results {
            if let Some(pool) = result.unwrap() {
                pools.push(pool.0);

                let uas = pool.2;
                update_pks_lengths.push(uas.clone().len());
                update_pks.extend(uas.clone());
                update_accounts.extend(pool.1);
                println!("added pool {:?}", pools.len());
            }
        }

        futures = vec![];
    }
    for pool in pools.clone(){ 

        let mints = pool.get_mints();
        
        let mint0_idx = *mint2idx.get(&mints[0]).unwrap();
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
    let src_ata = derive_token_address(&owner.pubkey(), &start_mint);

    // slide it in there
    println!("getting pool amounts...");



        println!("added {:?} update accounts", update_accounts.clone().len());

        let src_ata = derive_token_address(&rc_owner.pubkey(), &usdc_mint);

        let connection = Arc::new(RpcClient::new_with_commitment(connection_url, CommitmentConfig::recent()));

        let init_token_acc = connection.clone().get_account(&src_ata).unwrap();
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
    let update_accounts_chunks: std::slice::Chunks<'_, Option<Account>> = update_accounts.chunks(100);
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
        let mut filters: Vec<SubscribeRequestFilterAccountsFilter> = vec![];
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
                filters: filters
            },
        );
        anindex += 100;
    }
    println!("filter map is {:?}", filter_map.len());
    println!("searching for arbitrages...");
    let min_swap_amount = 10_u128.pow(3_u32); // scaled! -- 1 USDC

    let src_ata = derive_token_address(&rc_owner.pubkey(), &usdc_mint);

    let init_token_acc = connection.clone().get_account(&src_ata).unwrap();
    let init_token_balance: u128 = spl_token::state::Account::unpack(&init_token_acc.data).unwrap().amount as u128;
    let mut swap_start_amount = init_token_balance; // scaled!
    println!("swap start amount = {}", swap_start_amount); // track what arbs we did with a larger size
    let init_token_acc = connection.clone().get_account(&src_ata).unwrap();
    let init_token_balance: u128 = spl_token::state::Account::unpack(&init_token_acc.data).unwrap().amount as u128;

    
    println!("searching for arbitrages...");
    let min_swap_amount = 10_u128.pow(4_u32); // scaled! -- 1 USDC
    let mut swap_start_amount = init_token_balance; // scaled!
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
    let mut futures = vec![tokio::spawn(async move {
        let owner = Arc::new(read_keypair_file(owner_kp_path.clone()).unwrap());
        yellowstone(&mut pools.clone(), Arc::new(arbitrager), connection_url, filter_map, owner, start_mint_idx).await
    })];

        let results = join_all(futures).await;
       
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

fn create_and_or_extend_luts(
    remaining_public_keys: &Vec<Pubkey>,
    connection: &RpcClient,
    luts: &mut Vec<AddressLookupTableAccount>,
    payer: &Keypair,
) -> Result<Vec<AddressLookupTableAccount>, Box<dyn std::error::Error>> {
    let mut used_luts = Vec::new();

    for pubkeys in remaining_public_keys.chunks(25) {
        let (lut, _index) = find_or_create_lut(connection, payer, luts, remaining_public_keys.len())?;
            let extend_ix = extend_lookup_table(
                lut.key,
                payer.pubkey(),
                Some(payer.pubkey()),
                pubkeys.to_vec(),
            );
            let latest_blockhash = connection.get_latest_blockhash().unwrap(); 
            //println!("extending lut: {:?}", lut.key);
            connection
                .send_transaction(&VersionedTransaction::try_new(
                        VersionedMessage::V0(v0::Message::try_compile(
                            &payer.pubkey(),
                            &[extend_ix],
                            &[],
                            latest_blockhash,
                        ).unwrap()),
                        &[payer],
                    ).unwrap()
                ).unwrap();

                    
            used_luts.push(lut);
        }

    Ok(used_luts)
}
fn find_or_create_lut(
    connection:  &RpcClient,
    payer: &Keypair,
    luts: &mut Vec<AddressLookupTableAccount>,
    howmany: usize
) -> Result<(AddressLookupTableAccount, usize), Box<dyn std::error::Error>> {
    luts.shuffle(&mut rand::thread_rng());
    for (index, lut) in luts.iter().enumerate() {
        let acc = connection.get_account(&lut.key).unwrap();
        let address_lookup_table = AddressLookupTable::deserialize(&acc.data).unwrap();
        //println!("{}, {}", lut.addresses.len(), address_lookup_table.meta.authority.unwrap() == payer.pubkey());
        if lut.addresses.len() < (255_usize -howmany) && address_lookup_table.meta.authority.unwrap() == payer.pubkey() {
            return Ok((lut.clone(), index));
        }
    }
    Ok((create_new_lut(connection, payer).unwrap(), luts.len()))
}

fn create_new_lut(
    connection: &RpcClient,
    payer: &Keypair,
) -> Result<AddressLookupTableAccount, Box<dyn std::error::Error>> {
    // Create a new AddressLookupTable
    let recent_slot = connection
    .get_slot_with_commitment(CommitmentConfig::processed())
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
    connection
    .send_and_confirm_transaction_with_spinner(&VersionedTransaction::try_new(
            VersionedMessage::V0(v0::Message::try_compile(
                &payer.pubkey(),
                &[create_ix],
                &[],
                latest_blockhash,
            ).unwrap()),
            &[payer],
        ).unwrap()
    ).unwrap();

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
                CommitmentConfig::processed()
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

fn get_address_lookup_table_accounts(client: &RpcClient, keys: Vec<String>, payer: Pubkey) -> Vec<AddressLookupTableAccount> {
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