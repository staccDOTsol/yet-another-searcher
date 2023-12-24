use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::pubkey::Pubkey;
use redb::Database;
use redb::ReadableTable;
use redb::TableDefinition;
use client::monitor::pools::*;
const TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("my_data");
use anchor_client::solana_sdk::signature::read_keypair_file;
use anchor_client::solana_sdk::signature::{Keypair, Signer};

use anchor_client::{Client, Cluster};
use solana_program::program_pack::Pack;

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::rc::Rc;
use std::str::FromStr;

use std::borrow::Borrow;
use std::vec;

use clap::Parser;

use log::{debug, info, warn};
use solana_sdk::account::Account;

use client::constants::*;
use client::utils::{
    derive_token_address, read_json_dir, PoolEdge, PoolGraph, PoolIndex, PoolQuote,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(short, long)]
    pub cluster: String,
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

fn main() {
    let args = Args::parse();
    let cluster = match args.cluster.as_str() {
        "localnet" => Cluster::Localnet,
        "mainnet" => Cluster::Mainnet,
        _ => panic!("invalid cluster type"),
    };

    env_logger::init();

    let owner_kp_path = match cluster {
        Cluster::Localnet => "../../mainnet_fork/localnet_owner.key",
        Cluster::Mainnet => "/root/.config/solana/id.json",
        _ => panic!("shouldnt get here"),
    };

    // ** setup RPC connection
    let connection_url = match cluster {
        Cluster::Mainnet => "https://rpc.shyft.to?api_key=jdXnGbRsn0Jvt5t9",
        _ => cluster.url(),
    };
    println!("using connection: {}", connection_url);

    // setup anchor things
    let owner = read_keypair_file(owner_kp_path.clone()).unwrap();
    let owner2 = read_keypair_file(owner_kp_path.clone()).unwrap();
    let rc_owner = Rc::new(owner2);
    let rc_owner2 = Rc::new(owner);
    let provider = Client::new_with_options(
        cluster.clone(),
        rc_owner.clone(),
        CommitmentConfig::confirmed(),
    );
    let program = provider.program(*ARB_PROGRAM_ID);

    // ** define pool JSONs
    let mut pool_dirs: Vec<PoolDir> = vec![];

    let orca_dir = PoolDir {
        pool_type: PoolType::OrcaPoolType,
        dir_path: "../pools/orca".to_string(),
    };
    let r_dir = PoolDir {
        pool_type: PoolType::RaydiumPoolType,
        dir_path: "../pools/raydium".to_string(),
    };
    pool_dirs.push(r_dir);
    pool_dirs.push(orca_dir);
    let aldrin_dir = PoolDir {
        pool_type: PoolType::AldrinPoolType,
        dir_path: "../pools/aldrin".to_string(),
    };
    pool_dirs.push(aldrin_dir);

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
    println!("added {:?} mints", update_pks.len());
    println!("added {:?} mints", update_pks_lengths.len());

    println!("added {:?} mints", token_mints.len());
    println!("added {:?} pools", pools.len());

    // !
    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
    let start_mint = usdc_mint;
    let start_mint_idx = *mint2idx.get(&start_mint).unwrap();

    let owner: &Keypair = rc_owner2.borrow();
    let owner_start_addr = derive_token_address(&owner.pubkey(), &start_mint);

    println!("getting pool amounts...");
    let mut first = true;

    let connection = RpcClient::new_with_commitment(connection_url, CommitmentConfig::confirmed());
    // slide it out here
    let init_token_acc = derive_token_address(&owner.pubkey(), &usdc_mint);
    let init_token_balance =
        spl_token::state::Account::unpack(&connection.get_account(&init_token_acc).unwrap().data).unwrap().amount as u128;
    println!(
        "init token acc: {:?}, balance: {:#}",
        init_token_acc, init_token_balance
    );
    println!("starting balance = {}", init_token_balance);

    println!("setting up exchange graph...");

    let mut graph = PoolGraph::new();
    let mut pool_count = 0;
    let mut account_ptr = 0;

    let mut pool_count = 0;
    let mut account_ptr = 0;

    let json_str = std::fs::read_to_string("./update_pks.json").unwrap();
    let update_pks2: HashMap<String, Vec<String>> = serde_json::from_str(&json_str).unwrap();
    // given update_pks, find the account withing vec<string> and then return the key from the hashmap

    let mut pool_type = PoolType::OrcaPoolType;

    let mut pool_str = "".to_string();
    let db = Database::create("my_db.redb");
    if db.is_err() {
        println!("{}", "uhoh");
    }

    let db = db.unwrap();
    let read_txn = db.begin_read().unwrap();
    for mut pool in pools.into_iter() {
        for (key, value) in update_pks2.iter() {
            let pool_addr = pool.get_own_addr().to_string();

            if !key.contains(&pool_addr) {
                continue;
            }
            pool_str = std::fs::read_to_string(&key).unwrap();

            if key.contains("orca") {
                pool_type = PoolType::OrcaPoolType;
            } else if key.contains("aldrin") {
                pool_type = PoolType::AldrinPoolType;
            } else if key.contains("saber") {
                pool_type = PoolType::SaberPoolType;
            } else if key.contains("serum") {
                pool_type = PoolType::SerumPoolType;
            } else {
                panic!("pool type not found");
            }
            let tvalue = value.clone();
            update_pks = vec![];
            for v in tvalue {
                update_pks.push(Pubkey::from_str(&v).unwrap());
            }

            //range end index 518 out of range for slice of length 517
            if pool_count >= update_pks_lengths.len() {
                continue;
            }
            // update pool
            let length = update_pks_lengths[pool_count] - account_ptr;
            pool_count += 1;
            account_ptr += length;
            //range end index 518 out of range for slice of length 517

            let table = read_txn.open_table(TABLE).unwrap();
            let v1 = table.get(update_pks[0].to_string().as_str()).unwrap();

            if v1.is_none() {
                continue;
            }
            if false && pool.get_name() == "Serum" {
                let account = connection.get_account(&update_pks[2]).unwrap();
                let account1 = connection.get_account(&update_pks[1]).unwrap();
                let account0 = connection.get_account(&update_pks[0]).unwrap();
                pool.set_update_accounts(
                    vec![Some(account0), Some(account1), Some(account)],
                    cluster.clone(),
                );
            } else if false {
                let account = connection.get_account(&update_pks[1]).unwrap();
                let account2 = connection.get_account(&update_pks[0]).unwrap();
                pool.set_update_accounts(vec![Some(account2), Some(account)], cluster.clone());
            }

            pool.set_update_accounts2(update_pks[0], v1.unwrap().value(), cluster.clone());

            let v2 = table.get(update_pks[1].to_string().as_str()).unwrap();
            if v2.is_none() {
                continue;
            }
            pool.set_update_accounts2(update_pks[1], v2.unwrap().value(), cluster.clone());
            if update_pks.len() == 3 {
                let v3 = table.get(update_pks[2].to_string().as_str()).unwrap();

                pool.set_update_accounts2(update_pks[2], v3.unwrap().value(), cluster.clone());
            }
            let what = pool_count * 2..(pool_count + 1) * 2;
            if what.end > all_mint_idxs.len() {
                continue;
            }
        }
        // add pool to graph
        let idxs = &all_mint_idxs[pool_count * 2..(pool_count + 1) * 2].to_vec();
        let idx0 = PoolIndex(idxs[0]);
        let idx1 = PoolIndex(idxs[1]);

        let mut pool_ptr = PoolQuote::new(Rc::new(pool));

        add_pool_to_graph(&mut graph, idx0, idx1, &mut pool_ptr.clone());
        add_pool_to_graph(&mut graph, idx1, idx0, &mut pool_ptr);
    }

    if false {
        first = false
    }
}
