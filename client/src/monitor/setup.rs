use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_sdk::pubkey::*;
use anchor_client::solana_sdk::signature::Keypair;
use solana_sdk::signature::Signer;
use anchor_client::Cluster;
use std::borrow::Borrow;
use log::{debug, warn};
use solana_sdk::account::Account;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::rc::Rc;
use std::vec;
use std::sync::{Arc, Mutex};

type ShardedDb = Arc<Mutex<HashMap<String, Account>>>;

use super::pools::pool::{pool_factory, PoolDir, PoolType};
use crate::serialize::token::unpack_token_account;
use crate::utils::{
    derive_token_address, read_json_dir, PoolEdge, PoolGraph, PoolIndex, PoolQuote,
};

pub fn get_pool_directories() -> Vec<PoolDir> {
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

    return pool_dirs;
}

pub fn add_pool_to_graph<'a>(
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

/*
build graph works asynchronously with our http server receiving update_pks

0: compiles list of all accounts
1: requests account data from node
2a: for each account, adds to graph
2b: sends to rest api
GOTO 1 until all pools complete

On rest api:
0: start listening on handler, filter requests based on a map of pools
1a: receive request from shyft, if in map update value in graph
1b: receive new pool from build graph function: add to map used to filter

the map is used to filter which pools we're monitoring and allows us to insure we
dont miss any updates or become out of sync with the network. we populate pool in graph
and then send to http handler to add to the pools its monitoring
*/
pub fn build_graph(pool_dirs: Vec<PoolDir>,
    connection: RpcClient,
    cluster: Cluster,
    page_config: &Arc<Arc<Mutex<HashMap<String, Account>>>>,
    rc_owner: Rc<Keypair>) -> PoolGraph {
    // ** json pool -> pool object
    let mut token_mints = vec![];
    let mut pools = vec![];

    let mut update_pks = vec![];
    let mut update_pks_lengths = vec![];
    let mut all_mint_idxs = vec![];

    let mut mint2idx = HashMap::new();
    let mut graph_edges = vec![];
    // #[derive(Parser, Debug)]
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

    println!("setting up exchange graph...");
    let mut graph = PoolGraph::new();
    let mut pool_count = 0;
    let mut account_ptr = 0;

    for pool in pools.iter_mut() {
        // update pool
        let length = update_pks_lengths[pool_count];
        //range end index 518 out of range for slice of length 517
        if account_ptr + length > update_accounts.len() {
            break;
        }
        let _account_slice = &update_accounts[account_ptr..account_ptr + length].to_vec();

        pool.set_update_accounts(_account_slice.to_vec(), cluster.clone());

        // TODO: FIGURE OUT WHAT IS THIS DOING HERE
  
        let mut pc = page_config.lock().unwrap();
        let mut humbug = 0;
        for acc in _account_slice.to_vec() {
            pc.insert(update_pks[account_ptr + humbug].to_string(), acc.unwrap());
            humbug += 1;
        }
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

    return graph;
}
