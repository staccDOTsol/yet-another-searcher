use client::serialize::token::unpack_token_account;
use solana_client::rpc_filter::{Memcmp, RpcFilterType};
use solana_sdk::signature::Signature;
use solana_transaction_status::{UiTransactionEncoding, EncodedTransactionWithStatusMeta};
use switchboard_solana::{Instruction, AccountMeta};
use anchor_lang::*;
use anchor_client::*;
use marginfi::state::marginfi_account::{MarginfiAccount};
use marginfi::state::marginfi_group::{Bank, BankVaultType};
use marginfi::utils::find_bank_vault_authority_pda;
use solana_client::nonblocking::rpc_client::RpcClient;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::read_keypair_file;
use anchor_client::solana_sdk::signature::{Keypair, Signer};
use solana_client::rpc_config::RpcSendTransactionConfig;
use std::{
    collections::HashMap,
    env, fmt,
    sync::{Arc, Mutex},
    time::Duration,
};
use serde_json::json;


use futures::stream::{StreamExt};

use rand::{seq::SliceRandom};
use yellowstone_grpc_proto::geyser::{SubscribeRequestFilterTransactions, SubscribeUpdateTransaction};




use std::mem::size_of;

use log::{debug};


use solana_program::address_lookup_table_account::AddressLookupTableAccount;

use solana_program::message::{VersionedMessage, v0};
use solana_sdk::compute_budget::ComputeBudgetInstruction;

use core::panic;
use std::cell::RefCell;
use redb::{ReadableTable, TableDefinition};

 // 0.2.23


// Create the runtime
use client::execute::process::{Arbitrager, get_arbitrage_instructions,  get_address_lookup_table_accounts, create_and_or_extend_luts, Edge};









use {
    clap::{Parser, ValueEnum},
    futures::{sink::SinkExt},
    log::{error},
    std::{
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

pub async fn calculate_recent_fee(
) -> u64 {
let request = reqwest::Client::new()
                                                                            .post("https://jarrett-solana-7ba9.mainnet.rpcpool.com/8d890735-edf2-4a75-af84-92f7c9e31718")
.body(json!(
    {
        "jsonrpc": "2.0",
        "id": "1",
        "method": "getPriorityFeeEstimate",
        "params": 
    [{
        "accountKeys": ["JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"],
        "options": {
            "priority_level": "HIGH"
        }
    }]
}
).to_string())
.send().await.unwrap().text().await.unwrap();
let request = serde_json::from_str::<serde_json::Value>(&request).unwrap();
request["result"]["priorityFeeEstimate"].as_f64().unwrap_or(1200.0) as u64 * 10
}
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
            pubkey: Pubkey::try_from(account.clone().data).expect("valid pubkey"),
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






#[allow(dead_code)]
pub struct TransactionPretty {
    slot: u64,
    signature: Signature,
    is_vote: bool,
    tx: EncodedTransactionWithStatusMeta,
}

impl fmt::Debug for TransactionPretty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct TxWrap<'a>(&'a EncodedTransactionWithStatusMeta);
        impl<'a> fmt::Debug for TxWrap<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let serialized = serde_json::to_string(self.0).expect("failed to serialize");
                fmt::Display::fmt(&serialized, f)
            }
        }

        f.debug_struct("TransactionPretty")
            .field("slot", &self.slot)
            .field("signature", &self.signature)
            .field("is_vote", &self.is_vote)
            .field("tx", &TxWrap(&self.tx))
            .finish()
    }
}

impl From<SubscribeUpdateTransaction> for TransactionPretty {
    fn from(SubscribeUpdateTransaction { transaction, slot }: SubscribeUpdateTransaction) -> Self {
        let tx = transaction.expect("should be defined");
        Self {
            slot,
            signature: Signature::try_from(tx.signature.as_slice()).expect("valid signature"),
            is_vote: tx.is_vote,
            tx: yellowstone_grpc_proto::convert_from::create_tx_with_meta(tx)
                .expect("valid tx with meta")
                .encode(UiTransactionEncoding::Base64, Some(u8::MAX), true)
                .expect("failed to encode"),
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
#[tokio::main(worker_threads = 23)]
async fn yellowstone( marginfi_account: (Pubkey, MarginfiAccount), banks: Vec<((Pubkey, Bank))>
     )  {

        let connection_url: &str = "https://jarrett-solana-7ba9.mainnet.rpcpool.com/8d890735-edf2-4a75-af84-92f7c9e31718";
    
        // let_token_mints = arbitrager.clone().token_mints.clone();
                        
        let connection = solana_client::rpc_client::RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::finalized());

        let usdc_mint = Pubkey::from_str("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263").unwrap();

        let start_mint: Pubkey = usdc_mint;
        
    let args = Args::parse();
    let cluster = match args.cluster.as_str() {
        "localnet" => Cluster::Localnet,
        "mainnet" => Cluster::Mainnet,
        _ => panic!("invalid cluster type"),
    };
    // ** setup RPC connection
    println!("using connection: {}", connection_url);

    // let_connection = Arc::new(RpcClient::new_with_commitment(connection_url.to_string(), CommitmentConfig::finalized()));

    let owner_kp_path = "/home/azureuser/7i.json";

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

    let _usdc_mint = Pubkey::from_str("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263").unwrap();

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
        let mut max = 100000000;
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
    let usdc_mint = Pubkey::from_str("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263").unwrap();
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

    let mut filter_map: HashMap<String, SubscribeRequestFilterTransactions> = HashMap::new();

    let filters: Vec<SubscribeRequestFilterAccountsFilter> = vec![];
    filter_map.insert( "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string(),
                
    SubscribeRequestFilterTransactions {
        vote: None,
        failed: None,
        signature: None,
        account_include: vec!["675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string()],
        account_exclude: vec![],
        account_required: vec![],
    });
    
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
        
        anindex += 100;
    }

    println!("filter map is {:?}", filter_map.len());
    println!("searching for arbitrages...");
    println!("Found {} banks", banks.len());
    println!("Bank 0 {:?}", banks[0]);

    let wsol = banks.iter().find(|bank| bank.1.mint.to_string() == "So11111111111111111111111111111111111111112");
    let bonk = banks.iter().find(|bank| bank.1.mint.to_string() == "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263");
    let usdc = banks.iter().find(|bank| bank.1.mint.to_string() == "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    
    let bank = banks.iter().find(|bank| bank.1.mint.to_string() == "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263");

    
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
    

    // for edge in graph_edges, do edge_weight and add to weights
    
    println!("graph edges len {} ", graph_edges.clone().len());
    // let_min_swap_amount = 10_u128.pow(3_u32); // scaled! -- 1 USDC
    // let_owner_kp_path = "/home/azureuser/7i.json";
    let src_ata = derive_token_address(&rc_owner.pubkey(), &start_mint);

let _rc_owner_signer: &dyn solana_sdk::signature::Signer = &*rc_owner;
// let_signers = [rc_owner_signer];
    let init_token_acc = connection.get_account(&src_ata).await;
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
            accounts: HashMap::new(),
            transactions: filter_map,
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
                    Some(UpdateOneof::Transaction(tx)) => {
                        let tx: TransactionPretty = tx.into();
                        let tx =  tx.tx.transaction.decode().unwrap();
                        let accounts = tx.message.static_account_keys();
                        for pubkey in accounts {
                            let account = connection.get_account(&pubkey).await;
                            if account.is_err() {
                                continue;
                            }
                            let account = account.unwrap();
                        for pool in pools.iter_mut() {
                            let update_accounts = pool.get_update_accounts();
                            for update_account in update_accounts {
                                if update_account == *pubkey {
                                    println!("found update account {} with data {:?}", update_account, account.data.len());
                                    pool.set_update_accounts2(*pubkey, &account.data, Cluster::Mainnet);
                                    let mints = pool.get_mints();                      
                                println!("starting doit...");

                                let mut usdc_mint = Pubkey::from_str("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263").unwrap();
                                let _start_mint = usdc_mint;
                               

                                let mut end_mint = mints[0];
                                if mints[0] == usdc_mint {
                                    end_mint = mints[1];
                                }
                                
                                let usdc_mints = [usdc_mint];//, Pubkey::from_str("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263").unwrap(), Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap()];
                                let mint_idxs = [*mint2idx.get(&usdc_mint).unwrap()];//,  mint2idx.get(&Pubkey::from_str("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263").unwrap()).unwrap().clone(),  mint2idx.get(&Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap()).unwrap().clone()];
                                // select a random number 1-3
                            //    let onetothree = rand::thread_rng().gen_range(0..0);
                                let start_mint_idx = mint_idxs[0];
                                let end_mint_idx = *mint2idx.get(&end_mint).unwrap();
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
                                        let arbitrager = b.clone();
                                        let token_mints = arbitrager.clone().token_mints.clone();
                                        let graph_edges = arbitrager.clone().graph_edges.clone();
                                        let graph = arbitrager.clone().graph.clone();
                                        
                                        let c = Arbitrager {
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
                                                    end_mint_idx,
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
                            let recent_fees = calculate_recent_fee().await;
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

            let borrow_ix: Instruction  = make_bank_borrow_ix(
                rc_owner.clone().pubkey(),
                &bank.unwrap(),
                &marginfi_account,
                src_ata,
                (init_token_balance as u128 * 10 as u128) as u64
            ).await;
        ixs.insert(0, borrow_ix);
        let mut len_ixs = ixs.len();
       ixs.insert(0, Instruction {
        program_id: marginfi::id(),
        accounts: marginfi::accounts::LendingAccountStartFlashloan {
            marginfi_account: marginfi_account.0,
            signer: rc_owner.clone().pubkey(),
            ixs_sysvar: solana_sdk::sysvar::instructions::id(),
        }
        .to_account_metas(Some(true)),
        data: marginfi::instruction::LendingAccountStartFlashloan { end_index: (len_ixs+1) as u64 }.data(),
    });
                            ixs.insert(
                            0, priority_fee_ix
                            );

                let mut account_metas = marginfi::accounts::LendingAccountEndFlashloan {
                    marginfi_account: marginfi_account.0,
                     signer: rc_owner.clone().pubkey(),
                }
                .to_account_metas(Some(true));
                account_metas.extend(
                    load_observation_account_metas(vec![*wsol.unwrap(), *usdc.unwrap(), *bonk.unwrap()], marginfi_account.clone())
                        .await,
                );
                ixs.push(make_bank_deposit_ix(
                    rc_owner.clone().pubkey(),
                    src_ata.clone(),
                    &bank.unwrap(),
                    &marginfi_account,
                    (init_token_balance as u128 * 10 as u128) as u64
                ).await);
                ixs.push(Instruction {
                    program_id: marginfi::id(),
                    accounts: account_metas,
                    data: marginfi::instruction::LendingAccountEndFlashloan {}.data(),
                });
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
fn main() {
    // let_page_config = ShardedDb::new(Mutex::new(HashMap::new()));

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
    // setup anchor things
    let owner_kp_path = "/home/azureuser/7i.json";
    let owner = read_keypair_file(owner_kp_path).unwrap();
    let rc_owner = Arc::new(owner);
  let (marginfi_account, banks) = get_marginfi(&rc_owner);

            yellowstone(marginfi_account, banks);

}



 fn get_marginfi(rc_owner: &Keypair) -> ((Pubkey, MarginfiAccount), Vec<(Pubkey, Bank)>) {
    let client = Client::new(
        anchor_client::Cluster::Custom("https://jarrett-solana-7ba9.mainnet.rpcpool.com/8d890735-edf2-4a75-af84-92f7c9e31718".to_string(), "https://jarrett-solana-7ba9.mainnet.rpcpool.com/8d890735-edf2-4a75-af84-92f7c9e31718".to_string()),
        rc_owner.clone()
    );

    let program = client.program(marginfi::id()).unwrap();

    let accounts = program.accounts::<MarginfiAccount>(vec![]).unwrap();
    let mut account  = accounts[0];
    for acc in accounts {
        if acc.0 == Pubkey::from_str("EW1iozTBrCgyd282g2eemSZ8v5xs7g529WFv4g69uuj2").unwrap() {
            account = acc;
            break;
        }
    }
    let banks = program.accounts::<Bank>(vec![RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
        8 + size_of::<Pubkey>() + size_of::<u8>(),
        account.1.group.to_bytes().to_vec(),
    ))]).unwrap();
    println!("banks len is {:?}", banks.len());
    (account, banks)
}
pub async fn load_observation_account_metas(
    include_banks: Vec<(Pubkey, Bank)>,
    account: (Pubkey, MarginfiAccount),
) -> Vec<AccountMeta> {
    let marginfi_account = account.1;
    let mut bank_pks = marginfi_account
        .lending_account
        .balances
        .iter()
        .filter_map(|balance| {
            if balance.active {
                Some(balance.bank_pk)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    for bank_pk in include_banks.clone() {
        if !bank_pks.contains(&bank_pk.0) {
            bank_pks.push(bank_pk.0);
        }
    }
//    bank_pks.retain(|bank_pk| !exclude_banks.contains(bank_pk));

    let mut banks = vec![];
    for bank_pk in bank_pks.clone() {
        for bank in include_banks.clone() {
            if bank.0 == bank_pk {
                banks.push(bank.1);
            }
        }
    }

    let account_metas = banks
        .iter()
        .zip(bank_pks.iter())
        .flat_map(|(bank, bank_pk)| {
            vec![
                AccountMeta {
                    pubkey: *bank_pk,
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: bank.config.oracle_keys[0],
                    is_signer: false,
                    is_writable: false,
                },
            ]
        })
        .collect::<Vec<_>>();
    account_metas
}

pub async fn make_bank_borrow_ix(
    funding_account: Pubkey,
    bank: &(Pubkey, Bank),
    marginfi_account: &(Pubkey, MarginfiAccount),
    destination_account: Pubkey,
    amount: u64
) -> Instruction {
    let mut ix = Instruction {
        program_id: marginfi::id(),
        accounts: marginfi::accounts::LendingAccountBorrow {
            marginfi_group: marginfi_account.clone().1.group,
            marginfi_account: marginfi_account.clone().0,
            signer: funding_account ,
            bank: bank.0,
            destination_token_account: destination_account,
            bank_liquidity_vault: bank.clone().1.liquidity_vault,
            bank_liquidity_vault_authority: find_bank_vault_authority_pda(
                &bank.clone().0,
                BankVaultType::Liquidity,
            )
            .0,
            token_program: spl_token::ID,
        }
        .to_account_metas(Some(true)),
        data: marginfi::instruction::LendingAccountBorrow {
            amount
        }
        .data(),
    };
    ix.accounts.extend_from_slice(
        &load_observation_account_metas(vec![*bank], marginfi_account.clone())
            .await,
    );

    ix
}

pub async fn make_bank_deposit_ix(
    funding_account: Pubkey,
    funding_account_ata: Pubkey,
    bank: &(Pubkey, Bank),
    marginfi_account: &(Pubkey, MarginfiAccount),
    amount: u64
) -> Instruction {

    Instruction {
        program_id: marginfi::id(),
        accounts: marginfi::accounts::LendingAccountDeposit {
            marginfi_group: marginfi_account.clone().1.group,
            marginfi_account: marginfi_account.clone().0,
            signer: funding_account.clone(),
            bank: bank.clone().0,
            signer_token_account: funding_account_ata,
            bank_liquidity_vault: bank.clone().1.liquidity_vault,
            token_program: spl_token::ID,
        }
        .to_account_metas(Some(true)),
        data: marginfi::instruction::LendingAccountDeposit {
            amount
        }
        .data(),
    }
}