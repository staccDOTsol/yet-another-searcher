use client::serialize::token::unpack_token_account;
use rand::Rng;
use solana_client::nonblocking::rpc_client::RpcClient;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::read_keypair_file;
use anchor_client::solana_sdk::signature::{Keypair, Signer};
use solana_client::rpc_config::RpcSendTransactionConfig;
use tmp::accounts as tmp_accounts;
use tmp::instruction as tmp_ix;
use futures::stream::FuturesUnordered;
use futures::stream::StreamExt;
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
use tokio::runtime::Runtime;
use core::panic;
use std::cell::RefCell;
use redb::{ReadableTable, TableDefinition};
use solana_sdk::program_pack::Pack;
 // 0.2.23


// Create the runtime
use client::execute::process::{Arbitrager, get_arbitrage_instructions, calculate_recent_fee, get_address_lookup_table_accounts, create_and_or_extend_luts};





use serde::{Deserialize};

use std::sync::{Arc, Mutex};


use {
    clap::{Parser, ValueEnum},
    futures::{sink::SinkExt},
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
// use spl_token unpack_token_account


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
    #[clap(short, long)]
    pub yy: bool,
    

}
fn add_pool_to_graph<'a>(
    graph: &mut PoolGraph,
    idx0: PoolIndex,
    idx1: PoolIndex,
    quote:&mut PoolQuote
) {
    let edges = graph
        .0
        .entry(idx0)
        .or_insert_with(|| PoolEdge(HashMap::new()));
    let quotes = edges.0.entry(idx1).or_insert_with(|| vec![]);
    quotes.push(quote.clone());
    
}
async fn yellowstone( mut og_pools: &mut  Vec<Box< dyn PoolOperations>>,
    arbitrager: Arc<Arbitrager>,
    connection_url: &str,
     accounts: HashMap<String, SubscribeRequestFilterAccounts>,
     owner: Arc<Keypair>,
     start_mint_idx: usize
     )  {

        let token_mints = arbitrager.clone().token_mints.clone();
                        
        let connection = solana_client::rpc_client::RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::finalized());

        let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();

        let start_mint: Pubkey = usdc_mint;
        
    let _min_swap_amount = 10_u128.pow(3_u32); // scaled! -- 1 USDC
    let _owner_kp_path = "/home/ubuntu/.config/solana/id.json";
    let rc_owner = owner;
    let src_ata = derive_token_address(&rc_owner.pubkey(), &start_mint);

let rc_owner_signer: &dyn solana_sdk::signature::Signer = &*rc_owner;
let signers = [rc_owner_signer];
    let init_token_acc = connection.get_account(&src_ata);
    if init_token_acc.is_err() {
        println!("init token acc is err");
        return ;
    }
    let init_token_acc = init_token_acc.unwrap();
    let init_token_balance: u128 = unpack_token_account(&init_token_acc.data).amount as u128 - 1000;
    let swap_start_amount = init_token_balance; // scaled!
    println!("swap start amount = {}", swap_start_amount); // track what arbs we did with a larger size
    let _init_token_acc = connection.get_account(&src_ata);

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

}
#[tokio::main(worker_threads = 23)]

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut page_config = ShardedDb::new(Mutex::new(HashMap::new()));

    let args = Args::parse();
    let cluster = match args.cluster.as_str() {
        "localnet" => Cluster::Localnet,
        "mainnet" => Cluster::Mainnet,
        _ => panic!("invalid cluster type"),
    };
    // ** setup RPC connection
    let connection_url: &str = "https://jarrett-solana-7ba9.mainnet.rpcpool.com/8d890735-edf2-4a75-af84-92f7c9e31718";
    
    println!("using connection: {}", connection_url);

    let connection = Arc::new(RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::finalized()));

    let owner_kp_path = "/home/ubuntu/.config/solana/id.json";

    // setup anchor things
    let owner = read_keypair_file(owner_kp_path).unwrap();
    let rc_owner = Arc::new(owner);

    let provider = anchor_client::Client::new_with_options(
        Cluster::Custom(
            connection_url.to_string(),
            connection_url.to_string(),
        ),
        rc_owner.clone(),
        solana_sdk::commitment_config::CommitmentConfig::finalized(),
    );
    let program = provider.program(*ARB_PROGRAM_ID).unwrap();
    let program_async = program.async_rpc();
    // setup anchor things
    let owner2 = read_keypair_file(owner_kp_path).unwrap();
    let _rc_owner2 = Arc::new(owner2);
    let provider = Client::new_with_options(
        cluster.clone(),
        rc_owner.clone(),
        CommitmentConfig::finalized(),
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
    //pool_dirs.push(serum_dir);

    // ** json pool -> pool object
    let mut token_mints = vec![];
    let mut pools = vec![];

    let mut update_pks = HashMap::new();
    let mut all_mint_idxs = vec![];

    let mut mint2idx = HashMap::new();
    let mut graph_edges = vec![];

    println!("setting up exchange graph...");
    let mut graph = PoolGraph::new();
    let _pool_count = 0;
    let _account_ptr = 0;
    println!("extracting pool + mints...");
    let mut update_accounts: Vec<Account> = vec![];
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
let step = 70;
let mut indexy = 0;
let mut maybe_pools = vec![];
for pool_dir in pool_dirs.clone() {
    debug!("pool dir: {:#?}", pool_dir);
    let pool_paths = read_json_dir(&pool_dir.dir_path);
    for pool_path in &mut pool_paths.clone() {
      
        let json_str = std::fs::read_to_string(&pool_path).unwrap();
        let mut pool = pool_factory(&pool_dir.pool_type, &json_str);

        let pool_mints = pool.get_mints();
        if pool_mints.len() != 2 {
            // only support 2 mint pools
            println!("skipping pool with mints != 2: {:?}", pool_path);
            continue;
        }
        
        maybe_pools.push(pool.clone());
        
    }
}

let maybe_pools = maybe_pools.clone();
    for chunk in tuas.chunks(100) {
        let fut = program_async.get_multiple_accounts_with_commitment(&chunk, CommitmentConfig::finalized());
        futures.push(fut);
    }
let mut cycles = 0;
while !futures.is_empty() {
    println!("futures is {:?}", futures.len());
    let chunk_futures = futures.split_off(std::cmp::min(futures.len(), futures.len() - step));
    let results = join_all(chunk_futures).await;
    if futures.len() < step {
        break 
    }
    cycles += 1;
    for (i, result) in results.into_iter().enumerate() {
            println!("futures {:?}", (cycles * step + i) * 100);
            if let Ok(account_info) = result {
              
                let accounts = account_info.value;
                for account in accounts {
                    indexy += 1;
                
                    if account.is_none() {
                        continue;
                    }
                    let account = account.unwrap();
                    update_pks.insert(
                        tupdate_pks[indexy].clone(),
                        account.clone(),
                    );


                }
            }
        }
    }
    println!("update accounts is {:?}", update_accounts.len());
    println!("update pks is {:?}", update_pks.len());
    println!("maybe pools is {:?}", maybe_pools.len());
    let update_pks_set: HashSet<_> = update_pks.iter().map(|(pubkey, _)| pubkey.clone()).collect();

    let mut indexy = 0;
    for mut pool in maybe_pools {
        indexy += 1;
        let uas = pool.get_update_accounts();
        let mut counter = 0;
        let mut two_keys = vec![];
        for pubkey in uas {
            // get both update accounts
            if update_pks_set.contains(&pubkey) {

                if let Some(account) = update_pks.iter().find(|(pk, _)| pk == &&pubkey) {

                    two_keys.push(Some(account.1));
                }
            }
        }
        if two_keys.len() != 2 {
            continue;
        }

        pool.set_update_accounts(two_keys, cluster.clone());
        
            let pool_mints = pool.get_mints();
                pools.push(pool.clone());
                println!("{} / {}", indexy, pools.len());
            
    }
    
    println!("pool is {:?}", pools.clone().len());

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
        let mut pool_ptr = PoolQuote(Arc::new(RefCell::new(pool.clone_box())));
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
    println!("graph edges is {:?}", graph_edges.len());
    // !
    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
    let start_mint = usdc_mint;
    let start_mint_idx = *mint2idx.get(&usdc_mint).unwrap();

    let owner: &Keypair = &rc_owner.borrow();
    let _src_ata = derive_token_address(&owner.pubkey(), &start_mint);

    // slide it in there
    println!("getting pool amounts...");


let update_accounts = update_pks.clone().into_iter().map(|(_, account)| {
    account
}).collect::<Vec<Account>>();
        println!("added {:?} update accounts", update_accounts.clone().len());

        let src_ata = derive_token_address(&rc_owner.pubkey(), &usdc_mint);

        let connection = Arc::new(RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::finalized()));

        let init_token_acc = connection.clone().get_account(&src_ata).await.unwrap();
        let init_token_balance: u128 = unpack_token_account(&init_token_acc.data).amount as u128 - 1000;
    println!("update accounts is {:?}", update_accounts.len());
    // slide it out here
    println!(
        "init token acc: {:?}, balance: {:#}",
        init_token_acc, init_token_balance
    );
    println!("starting balance = {}", init_token_balance);

    let mut filter_map: HashMap<String, SubscribeRequestFilterAccounts> = HashMap::new();
    let update_accounts_chunks = update_accounts.chunks(2666);
    let owners = update_accounts_chunks.clone().map(|chunk| {
        chunk
            .iter()
            .map(|maybe| {
                maybe.owner.to_string()
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
        let update_pks = update_pks.clone()
        .iter()
        .map(|(pk, _)| {
            pk.to_string()
        })
        .collect::<Vec<String>>();
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
    let init_token_balance: u128 = unpack_token_account(&init_token_acc.data).amount as u128 - 1000;
    let swap_start_amount = init_token_balance; // scaled!
    println!("swap start amount = {}", swap_start_amount); // track what arbs we did with a larger size
    let init_token_acc = connection.clone().get_account(&src_ata).await.unwrap();
    let init_token_balance: u128 = unpack_token_account(&init_token_acc.data).amount as u128 - 1000;

    
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
        connection: Arc::new(solana_client::nonblocking::rpc_client::RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::finalized())),
        state: Arc::new(Mutex::new(HashSet::new() as HashSet<Vec<usize>>)),
        blacklist: Arc::new(Mutex::new(Vec::new())),
        };

    // for edge in graph_edges, do edge_weight and add to weights
    
    println!("graph edges len {} ", graph_edges.clone().len());
    /*
    for edge_set in graph_edges.clone() {
        let arbitrager = &arbitrager.clone();
        for mint_idx in edge_set {
            let future = edge_weight(arbitrager.clone(), mint_idx, start_mint_idx);
            futures.push(future);
            println!("futures len {} ", futures.len());
        }
    }

    
    // batch 23 futures at once
    let results = join_all(futures).await;

    for result in results {
        let (mint_idx, start_mint_idx, weight) = result;
        graph
        .0
        .entry(PoolIndex(start_mint_idx))
        .or_insert_with(|| PoolEdge(HashMap::new()))
        .0
        .entry(PoolIndex(mint_idx))
        .or_insert_with(|| (vec![(0, 0)], vec![]))
        .0
        .extend(weight.clone());
        weights.extend(weight);
        println!("weights len {} ", weights.len()); 
    }
    weights.sort_by(|a, b: &(usize, u128)| b.1.cmp(&a.1));
   */
    tokio::spawn(async move {
            
        let arbitrager = Arc::new(arbitrager.clone());
            yellowstone(&mut pools.clone(), (arbitrager.clone()), connection_url, filter_map, (rc_owner.clone()), start_mint_idx).await

    });
    let mut arbitrager = Arbitrager {
        token_mints: token_mints.clone(),
        graph_edges: graph_edges.clone(),   
        graph: graph.clone(),
        cluster: Cluster::Custom(
            connection_url.to_string(),
            connection_url.to_string()),
        connection: Arc::new(solana_client::nonblocking::rpc_client::RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::finalized())),
        state: Arc::new(Mutex::new(HashSet::new())),

        blacklist: Arc::new(Mutex::new(Vec::new())),
    };
                    doit(
                        &arbitrager.clone().token_mints,
                        &arbitrager.clone().graph_edges,
                        &arbitrager.clone().graph,
                        start_mint_idx,
                        Arc::new(read_keypair_file(owner_kp_path).unwrap()),
                        Arc::new(read_keypair_file(owner_kp_path).unwrap()),
                        mint2idx.clone(),
                    )
                    .await;
                

    Ok(())
}
async fn doit( token_mints: &Vec<Pubkey>,
    graph_edges: &Vec<HashSet<usize>>,
    graph: &PoolGraph,
    start_mint_idx: usize,
    owner: Arc<Keypair>,
    rc_owner: Arc<Keypair>,
    mint2idx: HashMap<Pubkey, usize>,
) -> ! {
    println!("starting doit...");

    let mut usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
    let start_mint = usdc_mint;
    
    let mut usdc_mints = vec![usdc_mint.clone()];//, Pubkey::from_str("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263").unwrap(), Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap()];
    let mut mint_idxs = vec![mint2idx.get(&usdc_mint).unwrap().clone()];//,  mint2idx.get(&Pubkey::from_str("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263").unwrap()).unwrap().clone(),  mint2idx.get(&Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap()).unwrap().clone()];
    // select a random number 1-3
//    let onetothree = rand::thread_rng().gen_range(0..0);
    let start_mint_idx = mint_idxs[0];
    usdc_mint = usdc_mints[0].clone();
    let owner: &Keypair = &rc_owner.borrow();
    let _src_ata = derive_token_address(&owner.pubkey(), &start_mint);

    // slide it in there
    println!("getting pool amounts...");

    let connection_url = "https://jarrett-solana-7ba9.mainnet.rpcpool.com/8d890735-edf2-4a75-af84-92f7c9e31718";
    



        let src_ata = derive_token_address(&rc_owner.pubkey(), &usdc_mint);

        let connection = Arc::new(RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::finalized()));
        let mut arbitrager: Arbitrager = Arbitrager {
            token_mints: token_mints.clone(),
            graph_edges: graph_edges.clone(),
            graph: graph.clone(),
            cluster: Cluster::Custom(
                connection_url.to_string(),
                connection_url.to_string()),
            connection: Arc::new(RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::finalized())),
            state: Arc::new(Mutex::new(HashSet::new())),

        blacklist: Arc::new(Mutex::new(Vec::new())),
            };
            
            
        let init_token_acc = connection.clone().get_account(&src_ata).await.unwrap();
        let init_token_balance: u128 = unpack_token_account(&init_token_acc.data).amount as u128 - 1000;
    let graph_edges = arbitrager.clone().graph_edges.clone(); 
    let graph = arbitrager.clone().graph.clone();
    let token_mints = arbitrager.clone().token_mints.clone();

// Create a FuturesUnordered to hold the futures

let mut b: Arbitrager =Arbitrager {
    token_mints: token_mints.clone(),
    graph_edges: graph_edges.clone(),
    graph: graph.clone(),
    cluster: Cluster::Custom(
        connection_url.to_string(),
        connection_url.to_string()),
    connection,
    state: Arc::new(Mutex::new(HashSet::new())),

    blacklist: Arc::new(Mutex::new(Vec::new())),
};
loop {

            let mut arbitrager = b.clone();
            let token_mints = arbitrager.clone().token_mints.clone();
            let graph_edges = arbitrager.clone().graph_edges.clone();
            let graph = arbitrager.clone().graph.clone();
            
            let mut c = Arbitrager {
                token_mints: token_mints.clone(),
                graph_edges: graph_edges.clone(),
                graph: graph.clone(),
                cluster: Cluster::Custom(
                    connection_url.to_string(),
                    connection_url.to_string()),
                connection: Arc::new(RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::finalized())),
                state: Arc::new(Mutex::new(HashSet::new())),

        blacklist: Arc::new(Mutex::new(Vec::new())),
                };
   

                  let result = c.find_yield(
                        start_mint_idx,
                        7,
                        init_token_balance - 10000
                    ).await;
                      
            if result.is_none() {
                continue;
            }
            let result = result.unwrap();
let ( arb_path, arb_pools, arbin_amounts) = (result.nodes, result.pool_idxs, result.yields);

//println!("arbkey: {:?}", arb_key);/* 
let usdc_mint = usdc_mint.clone();
let src_ata = src_ata.clone();
let init_token_balance = init_token_balance.clone();
let arb_path = arb_path.clone();
let pubkey = rc_owner.clone().pubkey();
let owner2 = rc_owner.clone();
let usdc_mint = Arc::new(usdc_mint.clone());
let arb_pools = Arc::new(arb_pools.clone());
let a = Arc::new(b.clone());
println!("arbin amounts: {:?}", arbin_amounts);
let ixs = get_arbitrage_instructions(
Arc::new(a.token_mints.clone()),
*usdc_mints.get(0).unwrap(),
init_token_balance.clone(),
arb_path.clone(),
arbin_amounts,
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
let connection = solana_client::rpc_client::RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::finalized());
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
if ixs.len() < 4 {
println!("ixs len is less than 4");
continue;
}

    let tx = VersionedTransaction::try_new( VersionedMessage::V0(v0::Message::try_compile(
        &rc_owner.pubkey(),
        &ixs,
        &lutties,
        blockhash.unwrap(),
        ).unwrap()), &signers);
    if tx.is_err() {
        println!("tx is err {} ", tx.err().unwrap());
        continue;
    }
    let tx = tx.unwrap();
    let connection = solana_client::rpc_client::RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::finalized());
    let sig = connection.send_transaction_with_config(&tx, 
        RpcSendTransactionConfig {
            skip_preflight: true,
            ..RpcSendTransactionConfig::default()
        });
    if sig.is_err() {
        print!("sig is err {} ", sig.err().unwrap());
        continue;
    }
    let sig = sig.unwrap();
    println!("https://solscan.io/tx/{:?}", sig);
}

}