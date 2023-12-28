use client::serialize::token::unpack_token_account;


use solana_client::nonblocking::rpc_client::RpcClient;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::read_keypair_file;
use anchor_client::solana_sdk::signature::{Keypair, Signer};
use solana_client::rpc_config::RpcSendTransactionConfig;




use futures::stream::{StreamExt};

use rand::{seq::SliceRandom};




use std::sync::Arc;

use log::{debug};


use solana_program::address_lookup_table_account::AddressLookupTableAccount;

use solana_program::message::{VersionedMessage, v0};
use solana_sdk::compute_budget::ComputeBudgetInstruction;

use core::panic;
use std::cell::RefCell;
use redb::{ReadableTable, TableDefinition};

 // 0.2.23


// Create the runtime
use client::execute::process::{Arbitrager, get_arbitrage_instructions, calculate_recent_fee, get_address_lookup_table_accounts, create_and_or_extend_luts, Edge};







use std::sync::{Mutex};


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


use std::str::FromStr;

use std::borrow::{Borrow};
use std::vec;




type ShardedDb = Arc<Mutex<HashMap<String, Arc<Mutex<PoolQuote>>>>>;

use client::constants::*;
use client::monitor::pools::pool::{pool_factory, PoolDir, PoolOperations, PoolType};
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
    let quotes = edges.0.entry(idx1).or_default();
    quotes.push(quote.clone());
    
}
async fn yellowstone( og_pools: &mut  Vec<Box< dyn PoolOperations>>,
    _arbitrager: Arc<Arbitrager>,
    connection_url: &str,
     accounts: HashMap<String, SubscribeRequestFilterAccounts>,
     owner: Arc<Keypair>,
     _start_mint_idx: usize
     )  {

        // let_token_mints = arbitrager.clone().token_mints.clone();
                        
        let connection = solana_client::rpc_client::RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::finalized());

        let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();

        let start_mint: Pubkey = usdc_mint;
        
    // let_min_swap_amount = 10_u128.pow(3_u32); // scaled! -- 1 USDC
    // let_owner_kp_path = "/home/ubuntu/.config/solana/id.json";
    let rc_owner = owner;
    let src_ata = derive_token_address(&rc_owner.pubkey(), &start_mint);

let _rc_owner_signer: &dyn solana_sdk::signature::Signer = &*rc_owner;
// let_signers = [rc_owner_signer];
    let init_token_acc = connection.get_account(&src_ata);
    if init_token_acc.is_err() {
        println!("init token acc is err");
        return ;
    }
    let init_token_acc = init_token_acc.unwrap();
    let init_token_balance: u128 = unpack_token_account(&init_token_acc.data).amount as u128 - 1000;
    let swap_start_amount = init_token_balance; // scaled!
    println!("swap start amount = {}", swap_start_amount); // track what arbs we did with a larger size
    // let_init_token_acc = connection.get_account(&src_ata);

    // let_min_swap_amount = 10_u128.pow(4_u32); // scaled! -- 1 USDC
    let mut client = GeyserGrpcClient::connect("https://jarrett-solana-7ba9.mainnet.rpcpool.com", Some("8d890735-edf2-4a75-af84-92f7c9e31718".to_string()), None).unwrap();
    let (mut subscribe_tx, mut stream) = client.subscribe().await.unwrap();
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
    // let_page_config = ShardedDb::new(Mutex::new(HashMap::new()));

    let args = Args::parse();
    let cluster = match args.cluster.as_str() {
        "localnet" => Cluster::Localnet,
        "mainnet" => Cluster::Mainnet,
        _ => panic!("invalid cluster type"),
    };
    // ** setup RPC connection
    let connection_url: &str = "https://jarrett-solana-7ba9.mainnet.rpcpool.com/8d890735-edf2-4a75-af84-92f7c9e31718";
    
    println!("using connection: {}", connection_url);

    // let_connection = Arc::new(RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::finalized()));

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
    let _owner2 = read_keypair_file(owner_kp_path).unwrap();
    // let_rc_owner2 = Arc::new(owner2);
    let _provider = Client::new_with_options(
        cluster.clone(),
        rc_owner.clone(),
        CommitmentConfig::finalized(),
    );
    // let_program = provider.program(*ARB_PROGRAM_ID);

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
    // let_serum_dir = PoolDir {
       // pool_type: PoolType::SerumPoolType,
       // dir_path: "../pools/openbook/".to_string(),
   // };
    //pool_dirs.push(serum_dir);

    // ** json pool -> pool object
    let mut token_mints = vec![];
    let mut pools = vec![];

    let mut update_pks = vec![];
    let mut all_mint_idxs = vec![];

    let mut mint2idx = HashMap::new();
    let mut graph_edges = vec![];

    let _usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();

    // let_start_mint: Pubkey = usdc_mint;
    println!("setting up exchange graph...");
    let mut graph = PoolGraph::new();
    // let_pool_count = 0;
    // let_account_ptr = 0;
    println!("extracting pool + mints...");
    let mut update_accounts = vec![];
    let mut tuas = vec![];
    let mut tuastopools = HashMap::new();
    let mut tupdate_pks = vec![];
    for pool_dir in pool_dirs.clone() {
        debug!("pool dir: {:#?}", pool_dir);
        let mut pool_paths = read_json_dir(&pool_dir.dir_path);
        let mut max = 10000000;
        if max > pool_paths.len() {
            max = pool_paths.len();
        }
        pool_paths.shuffle(&mut rand::thread_rng());
        pool_paths = pool_paths[..max].to_vec();


        for pool_path in &mut pool_paths.clone() {

            let json_str = std::fs::read_to_string(&pool_path).unwrap();
            let pool = pool_factory(&pool_dir.pool_type, &json_str);

           
            let uas = pool.get_update_accounts();
            tuas.push(uas.clone()[0]);
            tuas.push(uas.clone()[1]);
            tuastopools.insert(uas.clone()[0], pool.clone_box());
            tuastopools.insert(uas.clone()[1], pool.clone_box());
            tupdate_pks.extend(uas.clone().into_iter());
        }

    }
    println!("getting update accounts... tuas length is {:?}", tuas.len());

// let_step = 70;
// let_indexy = 0;
let mut maybe_pools = vec![];
for pool_dir in pool_dirs.clone() {
    debug!("pool dir: {:#?}", pool_dir);
    let pool_paths = read_json_dir(&pool_dir.dir_path);
    for pool_path in &mut pool_paths.clone() {
      
        let json_str = std::fs::read_to_string(&pool_path).unwrap();
        let pool = pool_factory(&pool_dir.pool_type, &json_str);

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

async fn process_data(
    tuastopools: &HashMap<Pubkey, Box<dyn PoolOperations>>,
    tuas: &[Pubkey], program_async: &RpcClient, maybe_pools: &[Box<dyn PoolOperations>], pools: &mut Vec<Box<dyn PoolOperations>>, update_accounts: &mut Vec<solana_sdk::account::Account>, update_pks: &mut Vec<Pubkey>
,graph_edges: &mut Vec<HashSet<Edge>>, graph: &mut PoolGraph, mint2idx: &mut HashMap<Pubkey, usize>, token_mints: &mut Vec<Pubkey>, all_mint_idxs: &mut Vec<usize>
) {
let _chunkindex = 0;
    for chunk in tuas.chunks(100) {
        
       let result = program_async.get_multiple_accounts_with_commitment(chunk, CommitmentConfig::confirmed()).await;
       
        match result {
            Ok(response) => {
                let mut two_keys = Vec::new();
                let mut count = 0;
                let accounts = response.value;
                for (pubkey, account) in chunk.iter().zip(accounts.iter()) {
                    match account {
                        Some(account) => {
                            let two_key = (*pubkey, account.clone());
                            two_keys.push(two_key);
                            count+=1;
                            if two_keys.len() == 2 {
                            process_accounts(tuastopools, graph_edges, graph, mint2idx, token_mints, all_mint_idxs, &two_keys, maybe_pools, pools, update_accounts, update_pks).await;
                            two_keys = Vec::new();
                            count = 0;
                            }
                            if count == 2 && two_keys.len() < 2 {
                                two_keys = Vec::new();
                                count = 0;
                            }
                        }
                        None => {
                            two_keys = Vec::new();
                            count = 0;
                        }
                    }

                }
            },
            Err(_) => {
                println!("error getting accounts");
            }
        }
    }
    
}

async fn process_accounts(tuastopools: &HashMap<Pubkey, Box<dyn PoolOperations>>, graph_edges: &mut Vec<HashSet<Edge>>, graph: &mut PoolGraph, mint2idx: &mut HashMap<Pubkey, usize>, token_mints: &mut Vec<Pubkey>, all_mint_idxs: &mut Vec<usize>,
    two_keys: &[(Pubkey, solana_sdk::account::Account)], _maybe_pools: &[Box<dyn PoolOperations>], pools: &mut Vec<Box<dyn PoolOperations>>, update_accounts: &mut Vec<solana_sdk::account::Account>, update_pks: &mut Vec<Pubkey>) {
        
                                let pool: &Box<dyn PoolOperations> = tuastopools.get(&two_keys[0].0).unwrap();
                        let mut pool = pool.clone_box();
            
                        pool.set_update_accounts(vec![Some(two_keys[0].1.clone()), Some(two_keys[1].1.clone())], Cluster::Mainnet);
                          
                        let mints = pool.get_mints();
            
                        if pool.can_trade(&mints[0], &mints[1]) {
                        
                            update_accounts.push(two_keys[0].1.clone());
                            update_accounts.push(two_keys[1].1.clone());
                            update_pks.push(two_keys[0].0);
                            update_pks.push(two_keys[1].0);
                            
            
                        
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
                    let mint1_idx = *mint2idx.get(&mints.clone()[1]).unwrap();
                    let idx0: PoolIndex = PoolIndex(mint0_idx);
                    let idx1: PoolIndex = PoolIndex(mint1_idx);
                    let mut pool_ptr = PoolQuote::new(
                        Arc::new(RefCell::new(pool.clone_box())));
                    add_pool_to_graph(graph, idx0, idx1, &mut pool_ptr.clone());
                    add_pool_to_graph(graph, idx1, idx0, &mut pool_ptr);

                    all_mint_idxs.push(mint0_idx);
                    all_mint_idxs.push(mint1_idx);
                    let edge0: Edge = Edge {
                        from: mint0_idx,
                        to: mint1_idx,
                        mint0idx: mints.clone()[0],
                        mint1idx: mints.clone()[1],
                    };
                    let edge1: Edge = Edge {
                        from: mint1_idx,
                        to: mint0_idx,
                        mint0idx: mints.clone()[1],
                        mint1idx: mints.clone()[0],
                    };
                    // record graph edges if they dont already exist
                    if !graph_edges[mint0_idx].
                    iter()
                    .any(|edge: &Edge | edge.mint0idx == mints.clone()[0]) {
                        graph_edges[mint0_idx].insert(edge0);
                    }
                    if !graph_edges[mint1_idx].
                    iter()
                    .any(|edge: &Edge | edge.mint1idx == mints.clone()[1]) {

                        graph_edges[mint1_idx].insert(edge1);
                    }
                    pools.push(pool.clone_box());
                    if pools.clone().len() % 1000 == 0 {
                        println!("added pool {:?} from {:?} total {:?}", pool.clone().get_own_addr(), pool.clone().get_name(), pools.clone().len());
                    }
                        
                    }
                }   
        
                process_data(&tuastopools, &tuas, &program_async, &maybe_pools, &mut pools, &mut update_accounts, &mut update_pks, &mut graph_edges, &mut graph, &mut mint2idx, &mut token_mints, &mut all_mint_idxs).await;


    // let_indexy = 0;
  
    println!("pool is {:?}", pools.clone().len());

    println!("graph edges is {:?}", graph_edges.len());
    // !
    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
    let _start_mint = usdc_mint;
    let start_mint_idx = *mint2idx.get(&usdc_mint).unwrap();

    let _owner: &Keypair = rc_owner.borrow();
    // let_src_ata = derive_token_address(&owner.pubkey(), &start_mint);

    // slide it in there
    println!("getting pool amounts...");


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
        .map(|pk| {
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
    // let_min_swap_amount = 10_u128.pow(3_u32); // scaled! -- 1 USDC

    let src_ata = derive_token_address(&rc_owner.pubkey(), &usdc_mint);

    let init_token_acc = connection.clone().get_account(&src_ata).await.unwrap();
    let init_token_balance: u128 = unpack_token_account(&init_token_acc.data).amount as u128 - 1000;
    let swap_start_amount = init_token_balance; // scaled!
    println!("swap start amount = {}", swap_start_amount); // track what arbs we did with a larger size
    let init_token_acc = connection.clone().get_account(&src_ata).await.unwrap();
    let init_token_balance: u128 = unpack_token_account(&init_token_acc.data).amount as u128 - 1000;

    
    println!("searching for arbitrages...");
    // let_min_swap_amount = 10_u128.pow(4_u32); // scaled! -- 1 USDC
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
        best_paths: Vec::new()
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
            yellowstone(&mut pools.clone(), arbitrager.clone(), connection_url, filter_map, rc_owner.clone(), start_mint_idx).await

    });
    let arbitrager = Arbitrager {
        token_mints: token_mints.clone(),
        graph_edges: graph_edges.clone(),   
        graph: graph.clone(),
        cluster: Cluster::Custom(
            connection_url.to_string(),
            connection_url.to_string()),
        connection: Arc::new(solana_client::nonblocking::rpc_client::RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::finalized())),
        state: Arc::new(Mutex::new(HashSet::new())),

        blacklist: Arc::new(Mutex::new(Vec::new())),
        best_paths: Vec::new()
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
    graph_edges: &Vec<HashSet<Edge>>,
    graph: &PoolGraph,
    _start_mint_idx: usize,
    _owner: Arc<Keypair>,
    rc_owner: Arc<Keypair>,
    mint2idx: HashMap<Pubkey, usize>,
) -> ! {
    println!("starting doit...");

    let mut usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
    let _start_mint = usdc_mint;
    
    let usdc_mints = [usdc_mint];//, Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(), Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap()];
    let mint_idxs = [*mint2idx.get(&usdc_mint).unwrap()];//,  mint2idx.get(&Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap()).unwrap().clone(),  mint2idx.get(&Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap()).unwrap().clone()];
    // select a random number 1-3
//    let onetothree = rand::thread_rng().gen_range(0..0);
    let start_mint_idx = mint_idxs[0];
    usdc_mint = usdc_mints[0];
    let _owner: &Keypair = rc_owner.borrow();
    // let_src_ata = derive_token_address(&owner.pubkey(), &start_mint);

    // slide it in there
    println!("getting pool amounts...");

    let connection_url = "https://jarrett-solana-7ba9.mainnet.rpcpool.com/8d890735-edf2-4a75-af84-92f7c9e31718";
    



        let src_ata = derive_token_address(&rc_owner.pubkey(), &usdc_mint);

        let connection = Arc::new(RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::finalized()));
        let arbitrager: Arbitrager = Arbitrager {
            token_mints: token_mints.clone(),
            graph_edges: graph_edges.clone(),
            graph: graph.clone(),
            cluster: Cluster::Custom(
                connection_url.to_string(),
                connection_url.to_string()),
            connection: Arc::new(RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::finalized())),
            state: Arc::new(Mutex::new(HashSet::new())),

        blacklist: Arc::new(Mutex::new(Vec::new())),
        best_paths: Vec::new()
            };
            
            
        let init_token_acc = connection.clone().get_account(&src_ata).await.unwrap();
        let init_token_balance: u128 = unpack_token_account(&init_token_acc.data).amount as u128 - 1000;
    let graph_edges = arbitrager.clone().graph_edges.clone(); 
    let graph = arbitrager.clone().graph.clone();
    let token_mints = arbitrager.clone().token_mints.clone();

// Create a FuturesUnordered to hold the futures

let b: Arbitrager =Arbitrager {
    token_mints: token_mints.clone(),
    graph_edges: graph_edges.clone(),
    graph: graph.clone(),
    cluster: Cluster::Custom(
        connection_url.to_string(),
        connection_url.to_string()),
    connection,
    state: Arc::new(Mutex::new(HashSet::new())),

    blacklist: Arc::new(Mutex::new(Vec::new())),
    best_paths: Vec::new()
};
loop {

            let arbitrager = b.clone();
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
        best_paths: Vec::new()
                };
   

                  let result = c.find_yield(
                        start_mint_idx,
                        9,
                        init_token_balance - 10000
                    ).await;
                      
            if result.is_none() {
                continue;
            }
            let result = result.unwrap();
let ( arb_path, arb_pools, arbin_amounts) = (result.nodes, result.pool_idxs, result.yields);

//println!("arbkey: {:?}", arb_key);/* 
let usdc_mint = usdc_mint;
let src_ata = src_ata;
let init_token_balance = init_token_balance;
let arb_path = arb_path.clone();
let pubkey = rc_owner.clone().pubkey();
let owner2 = rc_owner.clone();
let _usdc_mint = Arc::new(usdc_mint);
let arb_pools = Arc::new(arb_pools.clone());
let a = Arc::new(b.clone());
println!("arbin amounts: {:?}", arbin_amounts);
let ixs = get_arbitrage_instructions(
Arc::new(a.token_mints.clone()),
*usdc_mints.get(0).unwrap(),
init_token_balance,
arb_path.clone(),
arbin_amounts,
arb_pools.clone(),
owner2.clone(),
).await;

let mut ixs = ixs.0.concat();

println!("ixs: {:?}", ixs.len());
// let_hydra_ata = derive_token_address(&Pubkey::from_str("2bxwkKqwzkvwUqj3xYs4Rpmo1ncPcA1TedAPzTXN1yHu").unwrap(), &usdc_mint);
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