use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::read_keypair_file;
use anchor_client::solana_sdk::signature::{Keypair, Signer};
use base64::Engine;
use base64::engine::{GeneralPurpose, general_purpose};
use client::serialize::token::unpack_token_account;
use log::{debug, warn};
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
use solana_sdk::transaction::Transaction;
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
// use spl_token unpack_token_Account
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

async fn yellowstone(page_config: &mut Arc<ShardedDb>, 
    
    token_mints: Vec<Pubkey>, graph_edges: Vec<HashSet<usize>>, graph: PoolGraph, 
    
    cluster: &Cluster, connection_url: &str,
     accounts: HashMap<String, SubscribeRequestFilterAccounts>,
     mint2idx: HashMap<Pubkey, usize>,
     ) -> Result<(), Box<dyn std::error::Error>> {


        let connection = RpcClient::new_with_commitment(connection_url, CommitmentConfig::recent());

        let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
        let start_mint_idx = *mint2idx.get(&usdc_mint).unwrap();

        let start_mint: Pubkey = usdc_mint;
    let min_swap_amount = 10_u128.pow(3_u32); // scaled! -- 1 USDC
    let owner_kp_path = "/Users/jd/.config/solana/id.json";
    let owner = read_keypair_file(owner_kp_path.clone()).unwrap();
    let rc_owner = Rc::new(owner);
    let src_ata = derive_token_address(&rc_owner.pubkey(), &start_mint);
    let init_token_acc = connection.get_account(&src_ata).unwrap();
    let init_token_balance: u128 = unpack_token_account(&init_token_acc.data).amount as u128;
    let mut swap_start_amount = init_token_balance; // scaled!
    println!("swap start amount = {}", swap_start_amount); // track what arbs we did with a larger size
    let init_token_acc = connection.get_account(&src_ata).unwrap();

    println!("searching for arbitrages...");
    let min_swap_amount = 10_u128.pow(4_u32); // scaled! -- 1 USDC
    let mut client = GeyserGrpcClient::connect("https://jarrett-solana-7ba9.mainnet.rpcpool.com", Some("8d890735-edf2-4a75-af84-92f7c9e31718".to_string()), None)?;
    let (mut subscribe_tx, mut stream) = client.subscribe().await?;

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
        .await?;
    let arbitrager =Arbitrager {
        token_mints,
        graph_edges,
        graph,
        cluster: Cluster::Custom(
            connection_url.to_string(),
            connection_url.to_string()),
        connection
    };

    let mut messages: BTreeMap<u64, (Option<DateTime<Utc>>, Vec<String>)> = BTreeMap::new();
    while let Some(message) = stream.next().await {
        match message {
            Ok(msg) => {
                match msg.update_oneof {
                    Some(UpdateOneof::Transaction(tx)) => {
                        let entry = messages.entry(tx.slot).or_default();
                        let sig = Signature::try_from(tx.transaction.unwrap().signature.as_slice())
                            .expect("valid signature from transaction")
                            .to_string();
                        if let Some(timestamp) = entry.0 {
                            info!("received txn {} at {}", sig, timestamp);
                        } else {
                            entry.1.push(sig);
                        }
                    }
                    Some(UpdateOneof::Account(acc)) => {
                        let entry = messages.entry(acc.slot).or_default();
                        let account = acc.account.clone().unwrap();
                        let pubkey: Pubkey = bincode::deserialize(&account.pubkey).unwrap();
                        let account: Account = bincode::deserialize(&account.data).unwrap();
                        let init_token_balance: u128 = unpack_token_account(&init_token_acc.data).amount as u128;
                        swap_start_amount /= 2;
                        if swap_start_amount < min_swap_amount {
                            swap_start_amount = init_token_balance;
                        }
                           let arb = arbitrager.brute_force_search(
                                start_mint_idx,
                                init_token_balance,
                                swap_start_amount,
                                vec![start_mint_idx],
                                vec![],
                                0
                            );
                            println!("arb is {:?}", arb);

                        let pk = Pubkey::try_from(acc.account.unwrap().pubkey.as_slice())
                            .expect("valid pubkey from account")
                            .to_string();
                        if let Some(timestamp) = entry.0 {
                            info!("received account {} at {}", pk, timestamp);
                        } else {
                            entry.1.push(pk);
                        }
                    }
                    Some(UpdateOneof::BlockMeta(block)) => {
                        let entry = messages.entry(block.slot).or_default();
                        entry.0 = block.block_time.map(|obj| {
                            DateTime::from_naive_utc_and_offset(
                                NaiveDateTime::from_timestamp_opt(obj.timestamp, 0).unwrap(),
                                Utc,
                            )
                        });
                        if let Some(timestamp) = entry.0 {
                            for sig in &entry.1 {
                                info!("received txn {} at {}", sig, timestamp);
                            }
                        }

                        // remove outdated
                        while let Some(slot) = messages.keys().next().cloned() {
                            if slot < block.slot - 20 {
                                messages.remove(&slot);
                            } else {
                                break;
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
    Ok(())

}
#[tokio::main]

async fn main() {
    let mut page_config = Arc::new(ShardedDb::new(Mutex::new(HashMap::new())));

    let args = Args::parse();
    let cluster = match args.cluster.as_str() {
        "localnet" => Cluster::Localnet,
        "mainnet" => Cluster::Mainnet,
        _ => panic!("invalid cluster type"),
    };
    // ** setup RPC connection
    let connection_url = match cluster {
        Cluster::Mainnet => "https://jarrett-solana-7ba9.mainnet.rpcpool.com/8d890735-edf2-4a75-af84-92f7c9e31718",
        _ => cluster.url(),
    };
    println!("using connection: {}", connection_url);

    let connection = RpcClient::new_with_commitment(connection_url, CommitmentConfig::recent());

    let owner_kp_path = args.keypair.as_str();

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

    let r_dir = PoolDir {
        pool_type: PoolType::RaydiumPoolType,
        dir_path: "../pools/raydium".to_string(),
    };
    pool_dirs.push(r_dir);
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

    let serum_dir = PoolDir {
        pool_type: PoolType::SerumPoolType,
        dir_path: "../pools/serum/".to_string(),
    };
    pool_dirs.push(serum_dir);
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

    println!("extracting pool + mints...");
    for pool_dir in pool_dirs {
        debug!("pool dir: {:#?}", pool_dir);
        let pool_paths = read_json_dir(&pool_dir.dir_path);

        for pool_path in pool_paths {
            let json_str = std::fs::read_to_string(&pool_path).unwrap();
            let pool = pool_factory(&pool_dir.pool_type, &json_str);

            let pool_mints = pool.get_mints();
            if pool_mints.len() != 2 {
                // only support 2 mint pools
                warn!("skipping pool with mints != 2: {:?}", pool_path);
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

            // get accounts which need account println to be updated (e.g. pool src/dst amounts for xy=k)
            let update_accounts = pool.get_update_accounts();
            update_pks_lengths.push(update_accounts.len());
            update_pks.push(update_accounts);

            let mint0_idx = mint_idxs[0];
            let mint1_idx = mint_idxs[1];

            all_mint_idxs.push(mint0_idx);
            all_mint_idxs.push(mint1_idx);

            // record graph edges if they dont already exist
            if !graph_edges[mint0_idx].contains(&mint1_idx) {
                graph_edges[mint0_idx].insert(mint1_idx);
            }
            if !graph_edges[mint1_idx].contains(&mint0_idx) {
                graph_edges[mint1_idx].insert(mint0_idx);
            }

            pools.push(pool);
        }
    }
    let mut update_pks = update_pks.concat();

    println!("added {:?} mints", token_mints.len());
    println!("added {:?} pools", pools.len());

    // !
    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
    let start_mint = usdc_mint;
    let owner: &Keypair = rc_owner.borrow();
    let src_ata = derive_token_address(&owner.pubkey(), &start_mint);

    // slide it in there
    update_pks.push(src_ata);
    let pk_strings = update_pks.iter().map(|pk| pk.to_string()).collect::<Vec<String>>();
    std::fs::write("update_pks.json", serde_json::to_string(&pk_strings).unwrap()).unwrap();
    println!("getting pool amounts...");
    let mut update_accounts = vec![];
    for token_addr_chunk in update_pks.chunks(99) {
        let accounts = connection.get_multiple_accounts(token_addr_chunk).unwrap();
        update_accounts.extend(accounts);
        println!("got {:?} accounts", update_accounts.clone().len());
    }
    let mut update_accounts = update_accounts
        .into_iter()
        .filter(|s| s.is_some())
        .collect::<Vec<Option<Account>>>();


        let src_ata = derive_token_address(&rc_owner.pubkey(), &usdc_mint);

        let init_token_acc = connection.get_account(&src_ata).unwrap();
        let init_token_balance: u128 = unpack_token_account(&init_token_acc.data).amount as u128;
    println!("update accounts is {:?}", update_accounts.len());
    // slide it out here
    println!(
        "init token acc: {:?}, balance: {:#}",
        init_token_acc, init_token_balance
    );
    println!("starting balance = {}", init_token_balance);

    println!("setting up exchange graph...");
    let mut graph = PoolGraph::new();
    let mut pool_count = 0;
    let mut account_ptr = 0;

    let mut filter_map: HashMap<String, SubscribeRequestFilterAccounts> = HashMap::new();
    for pk in update_pks {
        let maybe = connection.get_account(&pk);
        if maybe.is_err() {
            continue;
        }
        let account = maybe.unwrap();
        let mut filters: Vec<SubscribeRequestFilterAccountsFilter> = vec![];
        
        filter_map.insert(
            pk.to_string(),
            
            SubscribeRequestFilterAccounts {
                account: vec![pk.to_string()],
                owner: vec![account.owner.to_string()],
                filters: filters
            },
        );
    }
    println!("filter map is {:?}", filter_map.len());

    for mut pool in pools.iter_mut() {
        // update pool
        let length = update_pks_lengths[pool_count];
        //range end index 518 out of range for slice of length 517
        if account_ptr + length > update_accounts.len() {
            break;
        }
        let _account_slice = &update_accounts[account_ptr..account_ptr + length].to_vec();
       

        pool.set_update_accounts(_account_slice.to_vec(), cluster.clone());
        let mut pc = page_config.lock().unwrap();
        let mut humbug = 0;
        account_ptr += length;
        // add pool to graph
        let idxs = &all_mint_idxs[pool_count * 2..(pool_count + 1) * 2].to_vec();
        let idx0 = PoolIndex(idxs[0]);
        let idx1 = PoolIndex(idxs[1]);
let pool = pool.clone();
        let mut pool_ptr = PoolQuote::new(Rc::new(pool));
        add_pool_to_graph(&mut graph, idx0, idx1, &mut pool_ptr.clone());
        add_pool_to_graph(&mut graph, idx1, idx0, &mut pool_ptr);

        pool_count += 1;
    }

    println!("searching for arbitrages...");
    let min_swap_amount = 10_u128.pow(3_u32); // scaled! -- 1 USDC

    let src_ata = derive_token_address(&rc_owner.pubkey(), &usdc_mint);

    let init_token_acc = connection.get_account(&src_ata).unwrap();
    let init_token_balance: u128 = unpack_token_account(&init_token_acc.data).amount as u128;
    let mut swap_start_amount = init_token_balance; // scaled!
    println!("swap start amount = {}", swap_start_amount); // track what arbs we did with a larger size
    let init_token_acc = connection.get_account(&src_ata).unwrap();
    let init_token_balance: u128 = unpack_token_account(&init_token_acc.data).amount as u128;

    
    println!("searching for arbitrages...");
    let min_swap_amount = 10_u128.pow(4_u32); // scaled! -- 1 USDC
    let mut swap_start_amount = init_token_balance; // scaled!
    println!("swap start amount = {}", swap_start_amount);
    // Spawn as tokio task
  

    
        yellowstone(&mut page_config, token_mints, graph_edges, graph, &cluster, connection_url, filter_map, mint2idx).await;

   
}
