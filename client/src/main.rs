use solana_client::nonblocking::rpc_client::RpcClient;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::read_keypair_file;
use anchor_client::solana_sdk::signature::{Keypair, Signer};
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
use redb::{ReadableTable, TableDefinition};
use solana_sdk::program_pack::Pack;
 // 0.2.23


// Create the runtime
use client::execute::process::Arbitrager;





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
    #[clap(short, long)]
    pub yy: bool,
    

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
async fn yellowstone( mut og_pools: &mut  Vec<Box< dyn PoolOperations>>,
    arbitrager: Arc<Arbitrager>,
    connection_url: &str,
     accounts: HashMap<String, SubscribeRequestFilterAccounts>,
     owner: Arc<Keypair>,
     start_mint_idx: usize
     )  {

        let token_mints = arbitrager.clone().token_mints.clone();
                        
        let connection = solana_client::rpc_client::RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::confirmed());

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
    let init_token_balance: u128 = spl_token::state::Account::unpack(&init_token_acc.data).unwrap().amount as u128;
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
#[tokio::main(worker_threads = 24)]

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
        solana_sdk::commitment_config::CommitmentConfig::confirmed(),
    );
    let program = provider.program(*ARB_PROGRAM_ID).unwrap();
    let program_async = program.async_rpc();
    // setup anchor things
    let owner2 = read_keypair_file(owner_kp_path).unwrap();
    let _rc_owner2 = Arc::new(owner2);
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
        if futures.len() > 22 {
            println!("futures length is {:?} update_accounts length is {:?}", futures.len(), update_accounts.len());
                
            let results = join_all(futures).await;
            futures = vec![];
            for result in results {
                if result.is_err() {
                    continue;
                }
                let account_infos = result.unwrap().value;
                for i in 0..account_infos.len(){
                   
                    let account = account_infos[i].clone();
                    update_accounts.push(account);
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
                   
                    let account = account_infos[i].clone();
                    update_accounts.push(account);
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
                    updateuauas.push(update_accounts[index].clone());
                    updatepkpks.push(index);
                    let acc = update_accounts[index].clone();
                    if acc.is_none() {
                        continue;
                    }
                    let acc = acc.unwrap();
                  //  pool.set_update_accounts2(update_pk, &acc.data, cluster.clone()).await;

                    
                }
                
                index += 1;
            }
            pool.set_update_accounts(updateuauas.clone(), cluster.clone());
            
            if !pool.can_trade(&pool_mints[0], &pool_mints[1]) {
                continue;
            }
            // ** record pool
            pools.push( pool.clone());
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

    let owner: &Keypair = &rc_owner.borrow();
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
        token_mints: token_mints.clone(),
        graph_edges: graph_edges.clone(),
        graph: graph.clone(),
        cluster: Cluster::Custom(
            connection_url.to_string(),
            connection_url.to_string()),
        connection
    };
    let mut twofutures = vec![];
        twofutures.push(tokio::task::spawn(async move {
            
        let arbitrager = Arc::new(arbitrager.clone());
            yellowstone(&mut pools.clone(), (arbitrager.clone()), connection_url, filter_map, (rc_owner.clone()), start_mint_idx).await
        }));


        
    let arbitrager =Arbitrager {
        token_mints: token_mints.clone(),
        graph_edges: graph_edges.clone(),
        graph: graph.clone(),
        cluster: Cluster::Custom(
            connection_url.to_string(),
            connection_url.to_string()),
        connection: Arc::new(solana_client::nonblocking::rpc_client::RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::confirmed()))
    };
    for i in 0..20 {
        let arbitrager = Arc::new(arbitrager.clone());
        
        twofutures.push(tokio::task::spawn(async move {
            doit(
                &arbitrager.clone().token_mints,
                &arbitrager.clone().graph_edges,
                &arbitrager.clone().graph,
                start_mint_idx,
                Arc::new(read_keypair_file(owner_kp_path).unwrap()),
                Arc::new(read_keypair_file(owner_kp_path).unwrap()),
            )
            .await
        }));
    }
    let results = join_all(twofutures).await;
    for result in results {
        if result.
        is_err() {
            println!("result is err");
            continue;
        }
        let result = result.unwrap();
        
    }

Ok(())
   
}

async fn doit( token_mints: &Vec<Pubkey>,
    graph_edges: &Vec<HashSet<usize>>,
    graph: &PoolGraph,
    start_mint_idx: usize,
    owner: Arc<Keypair>,
    rc_owner: Arc<Keypair>,
) {

    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
    let start_mint = usdc_mint;

    let owner: &Keypair = &rc_owner.borrow();
    let _src_ata = derive_token_address(&owner.pubkey(), &start_mint);

    // slide it in there
    println!("getting pool amounts...");

    let connection_url = "https://jarrett-solana-7ba9.mainnet.rpcpool.com/8d890735-edf2-4a75-af84-92f7c9e31718";
    



        let src_ata = derive_token_address(&rc_owner.pubkey(), &usdc_mint);

        let connection = Arc::new(RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::confirmed()));
        let mut arbitrager: Arbitrager = Arbitrager {
            token_mints: token_mints.clone(),
            graph_edges: graph_edges.clone(),
            graph: graph.clone(),
            cluster: Cluster::Custom(
                connection_url.to_string(),
                connection_url.to_string()),
            connection: Arc::new(RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::confirmed())),
            };
            
            
        let init_token_acc = connection.clone().get_account(&src_ata).await.unwrap();
        let init_token_balance: u128 = spl_token::state::Account::unpack(&init_token_acc.data).unwrap().amount as u128;
    let graph_edges = arbitrager.clone().graph_edges.clone();
    let graph = arbitrager.clone().graph.clone();
    let token_mints = arbitrager.clone().token_mints.clone();
let arb = arbitrager.optimized_search(
        start_mint_idx,
        init_token_balance,
        [start_mint_idx].to_vec(),
        token_mints,
        graph_edges,
        graph,
    ).await;

}

