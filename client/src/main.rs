use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::read_keypair_file;
use anchor_client::solana_sdk::signature::{Keypair, Signer};
use boilerplate::Boilerplate;
use client::serialize::token::unpack_token_account;
use core::panic;
use redb::{Database, Error, ReadableTable, TableDefinition};
use solana_sdk::program_pack::Pack;
use tokio::runtime::Runtime; // 0.2.23
use tokio::sync::mpsc;

// Create the runtime
use client::identify::arb::Arbitrager;

use std::fs::File;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use serde;
use serde::{Deserialize, Serialize};

use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
struct Config {
    msg: String,
}
#[derive(Debug)]
struct AppState {
    home_html: Mutex<HomeHtml>,
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
use solana_sdk::signature::Signature;
use solana_sdk::signers::Signers;
use solana_sdk::transaction::Transaction;
use structopt::StructOpt;

use flash_loan_sdk::instruction::{flash_borrow, flash_repay};
use flash_loan_sdk::{available_liquidity, flash_loan_fee, get_reserve, FLASH_LOAN_ID};

use anchor_client::{Client, Cluster};

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::io::Write;
use std::rc::Rc;
use std::str::FromStr;

use std::borrow::Borrow;
use std::vec;

use clap::Parser;

use log::{debug, info, warn};
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

#[derive(Debug)]
pub enum ServerError {
    BadRequest(String),
    Internal(Error),
    NotAcceptable(String),
    NotFound(String),
}

pub type ServerResult<T> = Result<T, ServerError>;

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message).into_response(),
            Self::Internal(error) => {
                eprintln!("error serving request: {error}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    StatusCode::INTERNAL_SERVER_ERROR
                        .canonical_reason()
                        .unwrap_or_default(),
                )
                    .into_response()
            }
            Self::NotAcceptable(content_type) => (
                StatusCode::NOT_ACCEPTABLE,
                format!("inscription content type `{content_type}` is not acceptable"),
            )
                .into_response(),
            Self::NotFound(message) => (
                StatusCode::NOT_FOUND,
                [(header::CACHE_CONTROL, HeaderValue::from_static("no-store"))],
                message,
            )
                .into_response(),
        }
    }
}

pub trait OptionExt<T> {
    fn ok_or_not_found<F: FnOnce() -> S, S: Into<String>>(self, f: F) -> ServerResult<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_not_found<F: FnOnce() -> S, S: Into<String>>(self, f: F) -> ServerResult<T> {
        match self {
            Some(value) => Ok(value),
            None => Err(ServerError::NotFound(f().into() + " not found")),
        }
    }
}

impl From<Error> for ServerError {
    fn from(error: Error) -> Self {
        Self::Internal(error)
    }
}
/*
parsed: {
  pubkey: 'EaXdHx7x3mdGA38j5RSmKYSXMzAFzzUXCLNBEDXDn1d5',
  lamports: 457104960,
  owner: 'srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX',
  executable: false,
  rent_epoch: 0,
  data: ['AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA'... 77400 more characters,
    'base64'
  ],
  write_version: 939620503313
     */
#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct ShyftInfo {
    pub pubkey: String,
    pub lamports: u64,
    pub owner: String,
    pub executable: bool,
    pub rent_epoch: u64,
    pub data: Vec<String>,
    pub write_version: u64,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct SetJson {
    pub status_code: u8,
}

#[derive(Debug, PartialEq)]
enum Command {
    Get { key: String },
    Set { key: String, val: Account },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct VecHomeHtml {
    pub account: Vec<HomeHtml2>,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct HomeHtml2 {
    pub account: HomeHtml,
}
#[derive(Boilerplate, Deserialize, Serialize, Debug, Clone)]
pub(crate) struct HomeHtml {
    pub parsed: ShyftInfo,
}
#[axum_macros::debug_handler]

async fn home(
    Extension(page_config): Extension<Arc<ShardedDb>>,
    b: String,
) -> ServerResult<impl IntoResponse> {
    // if b iz string

    let body = serde_json::from_str::<Vec<HomeHtml2>>(&b).unwrap();
    for i in 0..body.len() {
        page_config.lock().unwrap().insert(
            body[i].account.parsed.pubkey.clone(),
            Account {
                lamports: body[i].account.parsed.lamports,
                data: base64::decode(&body[i].account.parsed.data[0]).unwrap(),
                owner: Pubkey::from_str(&body[i].account.parsed.owner).unwrap(),
                executable: body[i].account.parsed.executable,
                rent_epoch: body[i].account.parsed.rent_epoch,
            },
        );
    }
    Ok(Json(SetJson { status_code: 200 }).into_response())
}
#[tokio::main]

async fn main() {
    let page_config = Arc::new(ShardedDb::new(Mutex::new(HashMap::new())));

    let router = Router::new()
        .route("/", post(home))
        .layer(Extension(page_config.clone()))
        .layer(
            CorsLayer::new()
                .allow_methods([http::Method::POST])
                .allow_origin(Any),
        );
    let handle = Handle::new();

    let addr = ("0.0.0.0", 3000)
        .to_socket_addrs()
        .unwrap()
        .next()
        .ok_or_else(|| anyhow::anyhow!("failed to get socket addrs"))
        .unwrap();
    println!("addr = {}", addr);

    // Spawn as tokio task
    tokio::spawn(async move {
        let server = axum_server::Server::bind(addr)
            .handle(handle)
            .serve(router.into_make_service());

        server.await
    });
    let args = Args::parse();
    let cluster = match args.cluster.as_str() {
        "localnet" => Cluster::Localnet,
        "mainnet" => Cluster::Mainnet,
        _ => panic!("invalid cluster type"),
    };

    env_logger::init();

    let owner_kp_path = match cluster {
        Cluster::Localnet => "../../mainnet_fork/localnet_owner.key",
        Cluster::Mainnet => "/Users/stevengavacs/.config/solana/id.json",
        _ => panic!("shouldnt get here"),
    };

    // ** setup RPC connection
    let connection_url = match cluster {
        Cluster::Mainnet => "https://rpc.shyft.to?api_key=jdXnGbRsn0Jvt5t9",
        _ => cluster.url(),
    };
    println!("using connection: {}", connection_url);

    let connection = RpcClient::new_with_commitment(connection_url, CommitmentConfig::recent());

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
    /*
    let mercurial_dir = PoolDir {
        pool_type: PoolType::MercurialPoolType,
        dir_path: "../pools/mercurial".to_string(),
    };
    pool_dirs.push(mercurial_dir);
     */

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
    let start_mint_idx = *mint2idx.get(&start_mint).unwrap();

    let owner: &Keypair = rc_owner.borrow();
    let owner_start_addr = derive_token_address(&owner.pubkey(), &start_mint);

    // slide it in there
    update_pks.push(owner_start_addr);

    println!("getting pool amounts...");
    let mut update_accounts = vec![];
    for token_addr_chunk in update_pks.chunks(99) {
        let accounts = connection.get_multiple_accounts(token_addr_chunk).unwrap();
        update_accounts.push(accounts);
    }
    let mut update_accounts = update_accounts
        .concat()
        .into_iter()
        .filter(|s| s.is_some())
        .collect::<Vec<Option<Account>>>();

    println!("update accounts is {:?}", update_accounts.len());
    // slide it out here
    let init_token_acc = update_accounts.pop().unwrap().unwrap();
    let init_token_balance = unpack_token_account(&init_token_acc.data).amount as u128;
    println!(
        "init token acc: {:?}, balance: {:#}",
        init_token_acc, init_token_balance
    );
    println!("starting balance = {}", init_token_balance);

    println!("setting up exchange graph...");
    let mut graph = PoolGraph::new();
    let mut pool_count = 0;
    let mut account_ptr = 0;

    for mut pool in pools.into_iter() {
        // update pool
        let length = update_pks_lengths[pool_count];
        //range end index 518 out of range for slice of length 517
        if account_ptr + length > update_accounts.len() {
            break;
        }
        let _account_slice = &update_accounts[account_ptr..account_ptr + length].to_vec();
        account_ptr += length;

        pool.set_update_accounts(_account_slice.to_vec(), cluster.clone());

        // add pool to graph
        let idxs = &all_mint_idxs[pool_count * 2..(pool_count + 1) * 2].to_vec();
        let idx0 = PoolIndex(idxs[0]);
        let idx1 = PoolIndex(idxs[1]);

        let mut pool_ptr = PoolQuote::new(Rc::new(pool));
        add_pool_to_graph(&mut graph, idx0, idx1, &mut pool_ptr.clone());
        add_pool_to_graph(&mut graph, idx1, idx0, &mut pool_ptr);

        pool_count += 1;
    }

    let arbitrager = Arbitrager {
        token_mints,
        graph_edges,
        graph,
        cluster,
        owner: rc_owner2,
        connection,
    };

    println!("searching for arbitrages...");
    let min_swap_amount = 10_u128.pow(3_u32); // scaled! -- 1 USDC
    let mut swap_start_amount = init_token_balance; // scaled!
    println!("swap start amount = {}", swap_start_amount);
    let mut sent_arbs = HashSet::new(); // track what arbs we did with a larger size

    let src_ata = derive_token_address(&owner.pubkey(), &usdc_mint);
    // PROFIT OR REVERT instruction
    let ix = spl_token::instruction::transfer(
        &spl_token::id(),
        &src_ata,
        &src_ata,
        &owner.pubkey(),
        &[],
        init_token_balance as u64,
    );
    arbitrager.brute_force_search(
        start_mint_idx,
        swap_start_amount,
        swap_start_amount,
        vec![start_mint_idx],
        vec![],
        &mut sent_arbs,
        ix.unwrap(),
        &page_config,
    );
}
