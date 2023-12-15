use std::collections::{HashMap, HashSet};

use crate::monitor::pools;
use crate::utils::PoolGraph;

use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::Cluster;
use anchor_lang::prelude::Account;
use solana_sdk::commitment_config::CommitmentConfig;

use std::rc::Rc;
use std::str::FromStr;
use std::vec;

use crate::monitor::setup::add_pool_to_graph;
use crate::identify::arbitrager::Arbitrager;
use crate::utils::{PoolIndex, PoolQuote};


pub async fn search(graph: PoolGraph) {
    println!("searching for arbitrages...");
    let min_swap_amount = 10_u128.pow(3_u32); // scaled! -- 1 USDC
    // TODO:
    let mut swap_start_amount = init_token_balance; // scaled!
    println!("swap start amount = {}", swap_start_amount); // track what arbs we did with a larger size
    let update_pks = std::fs::read_to_string("./data/update_pks.json").unwrap();
    let update_pks: HashMap<String, Vec<String>> = serde_json::from_str(&update_pks).unwrap();
    loop {
        let mut token_mints = vec![];

        let mut update_pks_lengths = vec![];
        let mut all_mint_idxs = vec![];

        let mut mint2idx = HashMap::new();
        let mut graph_edges = vec![];
        let mut graph = PoolGraph::new();
        let mut pool_count = 0;
        for pool in pools.iter_mut() {
            let accounts = pool.get_update_accounts();
            let pc = page_config.lock().unwrap();
            let name = pool.get_name();
            for acc in accounts {
                for (key, _val) in update_pks.clone() {
                    if name.to_lowercase() == key.to_lowercase() {
                        let data: Account = pc.get(&acc.to_string()).unwrap().clone();
                        pool.set_update_accounts2(
                            Pubkey::from_str(&acc.to_string()).unwrap(),
                            &data.data,
                            Cluster::Mainnet,
                        );
                    }
                }
            }

            let pool_mints = pool.get_mints();

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
            let idxs = &all_mint_idxs[pool_count * 2..(pool_count + 1) * 2].to_vec();
            let idx0 = PoolIndex(idxs[0]);
            let idx1 = PoolIndex(idxs[1]);
            let pool = pool.clone();
            let mut pool_ptr = PoolQuote::new(Rc::new(pool));

            // WHY IS THIS HERE?
            add_pool_to_graph(&mut graph, idx0, idx1, &mut pool_ptr.clone());
            add_pool_to_graph(&mut graph, idx1, idx0, &mut pool_ptr);

            pool_count += 1;
        }

        let connection = RpcClient::new_with_commitment(connection_url, CommitmentConfig::processed());
        let arbitrager = Arbitrager {
            token_mints,
            graph_edges,
            graph,
            cluster: Cluster::Mainnet,
            connection,
            config,
        };
        // PROFIT OR REVERT instruction
        let start_mint_idx: usize = *mint2idx.get(&start_mint).unwrap();

        arbitrager
            .brute_force_search(
                start_mint_idx,
                swap_start_amount,
                swap_start_amount,
                vec![start_mint_idx],
                vec![],
            )
            .await;
        swap_start_amount /= 2;
        if swap_start_amount < min_swap_amount {
            swap_start_amount = init_token_balance;
        }
    }
}