



use std::borrow::BorrowMut;
use std::collections::HashSet;


use rand::seq::SliceRandom;
use solana_address_lookup_table_program::instruction::{extend_lookup_table, create_lookup_table};
use solana_client::nonblocking::rpc_client::RpcClient;
use std::cmp::{max, Ordering, Reverse};



use rayon::prelude::*;
use futures::stream::{FuturesUnordered, StreamExt};



use futures::{FutureExt};
use anchor_client::solana_sdk::pubkey::Pubkey;

use anchor_client::solana_sdk::signature::{Keypair, Signer};
use anchor_client::{Cluster, Program};

use solana_address_lookup_table_program::state::{AddressLookupTable};


use solana_program::message::{VersionedMessage, v0};

use solana_sdk::account::Account;
use solana_sdk::address_lookup_table_account::AddressLookupTableAccount;
use solana_sdk::commitment_config::{CommitmentConfig};







use std::collections::{HashMap, BinaryHeap};


use std::sync::{Arc, Mutex};

use solana_sdk::instruction::Instruction;
use solana_sdk::transaction::{VersionedTransaction};



use std::str::FromStr;
use std::vec;



use tmp::accounts as tmp_accounts;
use tmp::instruction as tmp_ix;


use crate::monitor::pools::pool::{PoolOperations};

use crate::utils::{derive_token_address, PoolGraph, PoolIndex, PoolQuote};
#[derive(Clone)]
pub struct Arbitrager {
    pub best_paths: Vec<u128>,
    pub token_mints: Vec<Pubkey>,
    pub graph_edges: Vec<HashSet<Edge>>, // used for quick searching over the graph
    pub graph: PoolGraph,
    pub cluster: Cluster,
    // vv -- need to clone these explicitly -- vv
    pub connection: Arc<RpcClient>,
    pub state: Arc<Mutex<HashSet<Vec<usize>>>>,
    pub blacklist: Arc<Mutex<Vec<(usize, usize)>>>,

}
#[derive(Clone, Debug, Hash, Copy)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub mint0idx: Pubkey,
    pub mint1idx: Pubkey,
}
impl Eq for Edge {}
impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to
    }
}


impl Ord for Edge {
    fn cmp(&self, other: &Self) -> Ordering {
        // First, compare 'from' fields
        match self.from.cmp(&other.from) {
            Ordering::Equal => self.to.cmp(&other.to), // If 'from' fields are equal, compare 'to' fields
            other => other,
        }
    }
}

impl PartialOrd for Edge {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
#[derive(Clone)]
pub struct Path {
   pub nodes: Vec<usize>,
   edges: Vec<Edge>,  // List of edges in the path

   pub pool_idxs: Vec<PoolQuote>,
   pub last_edge: Edge,
   pub yields: Vec<u128>,
   current_node: usize,

    total_yield: u128,
    total_length: usize,
    most_recent_yield: u128,
}

impl PartialOrd for Path {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.cmp(other).into()
    }
}
impl Ord for Path {
    fn cmp(&self, other: &Self) -> Ordering {
        let our_pool_avg_non_0_weight = self.get_weight();
        let their_pool_avg_non_0_weight = other.get_weight();
        our_pool_avg_non_0_weight.partial_cmp(&their_pool_avg_non_0_weight).unwrap()
    }
}
impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        self.total_yield == other.total_yield
    }
}
impl Eq for Path {}


impl Path {
    pub fn get_weight(&self) -> u128 {
        let len_pools_non_0_weight = self.pool_idxs
            .iter()
            .filter(|pool| pool.1 > 0)
            .count();
        let our_pool_avg_non_0_weight = self.pool_idxs
            .iter()
            .map(|pool| pool.1)
            .sum::<u128>() as f64
            / len_pools_non_0_weight as f64;
        our_pool_avg_non_0_weight as u128
    }
    fn calculate_avg_yield(&self) -> u128 {
        let mut total = 0.0;
        let mut count = 0.0;
        for i in 0..self.pool_idxs.len() {
            let pool = self.pool_idxs[i].clone();
            if pool.1 != 0 {
                total += self.yields[i] as f64 / pool.1 as f64;
                count += 1.0;
            }
        }
        if total == 0.0 {
            return 0;
        }
        println!("avg yield for {:?} is {} ", self.nodes, total / count);
        return (total / count) as u128;
    }
    fn new(start_mint_idx: usize, amount: u128) -> Self {
        Self {
            nodes: vec![start_mint_idx],
            pool_idxs: vec![],
            yields: vec![amount],
            current_node: start_mint_idx,
            total_yield: amount,
            total_length: 1,
            most_recent_yield: amount,
            last_edge: Edge {
                from: start_mint_idx,
                to: start_mint_idx,
                mint0idx: Pubkey::new_from_array([0; 32]),
                mint1idx: Pubkey::new_from_array([0; 32])
            },
            edges: vec![Edge {
                from: start_mint_idx,
                to: start_mint_idx,
                mint0idx: Pubkey::new_from_array([0; 32]),
                mint1idx: Pubkey::new_from_array([0; 32])
             } ]
        }
    }

    fn last_node(&self) -> usize {
        self.nodes.last().copied().unwrap_or_default()
    }

    fn contains_node(&self, node: usize) -> bool {
        self.nodes.contains(&node)
    }

    fn extend(&mut self, edge: usize, additional_yield: u128, pool_idx: PoolQuote, poolidx_1: u128) {
        let poolidx_1 = poolidx_1 + 1;
       
        self.nodes.push(edge);
        self.pool_idxs.push(pool_idx.clone());
        self.yields.push(additional_yield);
        self.current_node = edge;
        self.total_yield += additional_yield;
        self.total_length += 1;
        self.most_recent_yield = additional_yield;

    }
    fn extend_with_edge(&mut self, to_node: usize, yield_value: u128, quote: PoolQuote, poolidx: u128, old_edge: Edge) {
        self.last_edge = old_edge;
        let poolidx = poolidx + 1;
        // Update the current node to the destination of the new edge
        self.current_node = to_node;
        // Update the path's total yield, most recent yield, etc.
        self.total_yield += yield_value;
        self.most_recent_yield = yield_value;

        // Add the new node and pool index to the path
        self.nodes.push(to_node);
        self.pool_idxs.push(quote.clone());

        // Increase the total length of the path
        self.total_length += 1;

        // Add the new yield value to the yields list
        self.yields.push(yield_value);

        // You might need additional logic depending on your application's needs
    }

}
impl Arbitrager {
    pub async fn find_yield(&self, start_mint_idx: usize, max_hops: usize, max_output: u128) -> Option<Path> {
    let mut max_heap = BinaryHeap::new();
    let graph_edges = self.graph_edges.clone();

        let path = Path::new(start_mint_idx, 1_000);
        let path2 = Path::new(start_mint_idx, max_output);

        max_heap.push(path2.clone());



    while let Some(mut path2) = max_heap.pop() {

        let mut edges = graph_edges[path.current_node].clone();
        let mut futures = futures::stream::FuturesUnordered::new();
        let mut futures2 = futures::stream::FuturesUnordered::new();
   
        let timestamp = Arbitrager::current_timestamp() % edges.len();
        let mut max = timestamp + 100;
        if max > edges.len() {
            max = edges.len();
        }
        let edges = edges.into_iter().collect::<Vec<Edge>>();
        let edges = edges[timestamp..max].to_vec();
        for edge in &edges {
            let pools = self.graph.0.get(&PoolIndex(edge.from)).and_then(|p| p.0.get(&PoolIndex(edge.to)));
            if pools.is_none() {
                continue;
            }
            let pools = pools.unwrap();
            for pool in pools {
                let pool = pool.clone();
                if pool.1 == 0 || pool.2 < Arbitrager::current_timestamp() - 1000 * 60 * 2 {
                    futures2.push(self.get_yield( *edge, start_mint_idx, 1_000, 1, edge.to));
                }
                
            }
            if path2.calculate_avg_yield() > 0 {
                futures.push(self.process_edge(*edge, &path2, start_mint_idx, path2.most_recent_yield));
            }

        }


        while let Some(result) = futures2.next().await {
            let pool = result.1;

            if pool.is_none() {
                continue;
            }
            let ownaddr = pool.clone().unwrap().get_own_addr();

            let mut pool = pool.unwrap();
            let p = pool.borrow_mut();
            let new_yield = result.0;
                if new_yield > 0 && (p.1 == 0 || (p.2 < Arbitrager::current_timestamp() * 1000 * 60 * 2)) {
                    p.set_weight(new_yield);
                    println!("new pool weight for {} is {}", ownaddr, new_yield);
                }
            }

        while let Some(result) = futures.next().await {
            if let Some((new_path, new_total_yield, _edge)) = result {
                if new_total_yield > max_output as u128 {
                        println!("new path is {:?}", new_path.clone().nodes);
                        println!("new path is {:?}", new_path.clone().yields);
                }
                if new_total_yield > max_output as u128 && new_path.total_length > 2 && new_path.last_node() == start_mint_idx {
                    return Some(new_path);
                }
                max_heap.push(new_path.clone());
            }
        }

    }

    max_heap.into_sorted_vec().into_iter().find(|p| p.last_node() == start_mint_idx)
}

    async fn process_edge(&self, edge: Edge, path: &Path, start_mint_idx: usize, max_output: u128) -> Option<(Path, u128, Edge)> {
        if path.contains_node(edge.to) {
            return None;
        }
        if path.total_length > 4 {
            return None;
        }
        let mut yield_increase = 0;
        let mut quote = None;
        let mut updated_edge = edge;
        let mut edge = edge;
        if path.total_length >= 3 {
            edge.to = start_mint_idx;
        }
        (yield_increase, quote, updated_edge) = self.compute_yield_improvement(&edge, path).await;
        if yield_increase == 0 {
            return None;
        }

        let mut new_path = path.clone();
        let quote = quote.unwrap();
        new_path.extend_with_edge(edge.to, yield_increase, quote.clone(), path.get_weight(), edge);
        if !Arbitrager::is_invalid_path(&new_path, start_mint_idx, max_output) {
            let new_avg_yield = (new_path.yields[new_path.yields.len()-1] as f64 / quote.1 as f64) as u128;
           // println!("new total yielding path is {:?} {:?} {:?} {:?}", new_path.nodes, new_path.yields, new_path.total_yield, new_path.most_recent_yield_usd);
            Some((new_path, new_avg_yield as u128, updated_edge))
        } else {
            None
        }
    }

    fn is_invalid_path( path: &Path, start_mint_idx: usize, max_output: u128) -> bool {
        // Example logic; replace with your actual conditions
        path.total_length > 4 
    }

fn is_valid_path(path: &Path, start_mint_idx: usize, max_output: u128) -> bool {
    // Example logic; replace with your actual conditions
    (path.calculate_avg_yield() as f64 > max_output as f64 * 0.8 as f64 && path.calculate_avg_yield() as f64 <= max_output as f64 * 3.33) || path.nodes.len() < 3
}

async fn compute_yield_improvement(&self, edge: &Edge, path: &Path) -> (u128, Option<PoolQuote>, Edge) {
    let amount = path.yields[path.yields.len()-1]; // Or however you determine the amount to compute yield for

    // Compute yield for the given edge
    let (new_yield, quote, _updated_edge) = self.compute_yield(*edge, amount, path).await;

    (new_yield, quote, *edge)
}

async fn compute_yield(&self, edge: Edge, amount: u128, path: &Path) -> (u128, Option<PoolQuote>, Edge) {
    let from = edge.from;
    let to = edge.to;
    let pool = self.graph.0.get(&PoolIndex(from)).and_then(|p| p.0.get(&PoolIndex(to)));
    if pool.is_none(){
        return (0, None, edge);
    }
    let pool = pool.unwrap();
    let mut new_balance = 0;
    for pool in pool {
        let mut pool = pool.clone();
        let connection: solana_client::rpc_client::RpcClient = solana_client::rpc_client::RpcClient::new(self.cluster.url().to_string());
        new_balance = pool.get_quote_with_amounts_scaled(amount, &self.token_mints[from], &self.token_mints[to], &Arc::new(connection));
        if new_balance > 0 {
            return (new_balance, Some(pool.clone()), edge);
        }   
    }
    if new_balance == 0 {
        let mut blacklist = self.blacklist.try_lock().unwrap();
        blacklist.push((from, to));
        drop(blacklist);
    }
    (0, None, edge)
    
}
async fn get_yield(&self, mut edge: Edge, start_mint_idx: usize, amount: u128, current_hops: usize, _to: usize) -> (u128, Option<PoolQuote>, Edge) {
    edge.from = start_mint_idx;
    let mut queue = BinaryHeap::new();
    let initial_path = Path { edges: vec![edge.clone()], total_length: 1,
        total_yield: 0, most_recent_yield: 0, current_node: edge.to, nodes: vec![edge.from], pool_idxs: vec![], yields: vec![], last_edge: edge.clone() };
    queue.push((Reverse(1), edge, initial_path)); // Reverse for min-heap

    while let Some((Reverse(hops), current_edge, path)) = queue.pop() {
       
        if hops > 5 {
            continue;
        }

        if current_edge.to == start_mint_idx {
            let yield_result = self.compute_yield(current_edge, amount, &path).await;
            return (yield_result.0, yield_result.1, current_edge);
        }

        if let Some(edges) = self.graph_edges.get(current_edge.to) {
            for &next_edge in edges {
                let mut new_path = path.clone();
                new_path.edges.push(next_edge);
                new_path.total_length += 1;

                queue.push((Reverse(new_path.total_length), next_edge, new_path));
            }
        }
    }

    (0, None, edge)
}



pub fn current_timestamp() -> usize {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as usize
}


}

pub  async  fn get_arbitrage_instructions<'a>(
    token_mints: Arc<Vec<Pubkey>>,
    src_mint: Pubkey,
        swap_start_amount: u128,
        mint_idxs: Vec<usize>,
        amounts: Vec<u128>,
        pools: Arc<Vec<PoolQuote>>,   
    owner: Arc<Keypair>
     ) -> (Vec<Vec<Instruction>>, bool) {
            
        // gather swap ixs
        let mut ixs = vec![];
        let swap_state_pda =
            Pubkey::from_str("8cjtn4GEw6eVhZ9r1YatfiU65aDEBf1Fof5sTuuH6yVM").unwrap();
        let _src_ata = derive_token_address(&owner.clone().pubkey(), &src_mint);

    let provider = anchor_client::Client::new_with_options(
        Cluster::Mainnet,
        owner.clone(),
        solana_sdk::commitment_config::CommitmentConfig::confirmed(),
    );
        let program: Program<Arc<Keypair>> = provider.program(*crate::constants::ARB_PROGRAM_ID).unwrap();

        // initialize swap ix
        let ix = program
            .request()
            .accounts(tmp_accounts::TokenAndSwapState {
                swap_state: swap_state_pda,
            })
            .args(tmp_ix::StartSwap {
                swap_input: swap_start_amount as u64,
            })
            .instructions()
            .unwrap();
            
        ixs.push(ix);
        let flag = false;
        println!("{:?}", amounts);


        for i in 0..mint_idxs.len() - 1{
           let swap_start_amount = amounts[i];

            let program: Program<Arc<Keypair>> = provider.program(*crate::constants::ARB_PROGRAM_ID).unwrap();

            let [mint_idx0, mint_idx1] = [mint_idxs[i], mint_idxs[i + 1]];
            println!("mint idxs are {} {}", mint_idx0, mint_idx1);
            let pool = &pools[i];
            println!("pool is {:?}", pool.get_own_addr());
            let pool3 = pool.clone();
            let token_mints = token_mints.clone();
            println!("getting quote with amounts scaled {} {} {} {} ", i, token_mints[mint_idx0].clone(), token_mints[mint_idx1].clone(), swap_start_amount);
            let swap_ix = pool3.swap_ix(token_mints[mint_idx0],
            token_mints[mint_idx1], swap_start_amount,
            owner.clone().pubkey(), &program);
            println!("swap ix is {:?}", swap_ix.1.len());
            if swap_start_amount == 0 {
                return (vec![], false);
            }
            ixs.push(swap_ix.1);
            println!("ixs is {}", ixs.len());
        }
       
        ixs.concat();
        (ixs,  flag) 

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
    
pub     fn create_and_or_extend_luts(
        remaining_public_keys: &Vec<Pubkey>,
        connection: &solana_client::rpc_client::RpcClient,
        luts: &mut Vec<AddressLookupTableAccount>,
        payer: &Keypair,
    ) -> Result<Vec<AddressLookupTableAccount>, Box<dyn std::error::Error>> {
        let mut used_luts = Vec::new();
    
        for pubkeys in remaining_public_keys.chunks(25) {
            let (lut, _index) = find_or_create_lut(connection, payer, luts, remaining_public_keys.len()).unwrap()   ;
                let extend_ix = extend_lookup_table(
                    lut.key,
                    payer.pubkey(),
                    Some(payer.pubkey()),
                    pubkeys.to_vec(),
                );
                let latest_blockhash = connection.get_latest_blockhash(); 
                //println!("extending lut: {:?}", lut.key);
               let hm: Result<solana_sdk::signature::Signature, solana_client::client_error::ClientError> = connection
                    .send_and_confirm_transaction(&VersionedTransaction::try_new(
                            VersionedMessage::V0(v0::Message::try_compile(
                                &payer.pubkey(),
                                &[extend_ix],
                                &[],
                                latest_blockhash.unwrap(),
                            ).unwrap()),
                            &[payer],
                        ).unwrap()
                    );
                    if hm.is_ok() {
                        let _signature = hm.unwrap();
                        
                    }
                    
    
                        
                used_luts.push(lut);
            }
    
        Ok(used_luts)
    }
     fn find_or_create_lut(
        connection:  &solana_client::rpc_client::RpcClient,
        payer: &Keypair,
        luts: &mut Vec<AddressLookupTableAccount>,
        howmany: usize
    ) -> Result<(AddressLookupTableAccount, usize), Box<dyn std::error::Error>> {
        luts.shuffle(&mut rand::thread_rng());
        for (index, lut) in luts.iter().enumerate() {
            let acc = connection.get_account(&lut.key);
            if acc.is_err() {
                continue;
            }
            let acc = acc.unwrap();
            let address_lookup_table = AddressLookupTable::deserialize(&acc.data).unwrap();
            //println!("{}, {}", lut.addresses.len(), address_lookup_table.meta.authority.unwrap() == payer.pubkey());
            if lut.addresses.len() < (255_usize -howmany) && address_lookup_table.meta.authority.unwrap() == payer.pubkey() {
                return Ok((lut.clone(), index));
            }
        }
        Ok((create_new_lut(connection, payer).unwrap(), luts.len()))
    }
    
     fn create_new_lut(
        connection: &solana_client::rpc_client::RpcClient,
        payer: &Keypair,
    ) -> Result<AddressLookupTableAccount, Box<dyn std::error::Error>> {
        // Create a new AddressLookupTable
        let recent_slot = connection
        .get_slot_with_commitment(CommitmentConfig::confirmed())
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
      let hm = connection
        .send_transaction(&VersionedTransaction::try_new(
                VersionedMessage::V0(v0::Message::try_compile(
                    &payer.pubkey(),
                    &[create_ix],
                    &[],
                    latest_blockhash,
                ).unwrap()),
                &[payer],
            ).unwrap()
        );
        if hm.is_ok() {
            let _signature = hm.unwrap();
            
        }
    
        
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
        connection: &solana_client::rpc_client::RpcClient,
    ) -> u64 {
        //println!("calculating recent fee");
        //println!("write locked accounts: {:?}", write_locked_accounts.len());
        // do in chunks of 100 
        let write_locked_accounts = write_locked_accounts.to_vec();
        let chunks = write_locked_accounts.chunks(100);
        for chunk in chunks {
                let account_infos = connection.get_multiple_accounts_with_commitment(
                    chunk,
                    CommitmentConfig::confirmed()
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
    
pub    async fn get_address_lookup_table_accounts(client: &solana_client::rpc_client::RpcClient, keys: Vec<String>, payer: Pubkey) -> Vec<AddressLookupTableAccount> {
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
// from https://github.com/solana-labs/solana/blob/10d677a0927b2ca450b784f750477f05ff6afffe/sdk/program/src/message/versions/v0/mod.rs#L209
async fn create_tx_with_address_table_lookup(
    client: &RpcClient,
    instructions: &[Instruction],
    address_lookup_table_key: Pubkey,
    payer: &Keypair,
) -> VersionedTransaction {
    let raw_account = client.get_account(&address_lookup_table_key).await.unwrap();
    let address_lookup_table = AddressLookupTable::deserialize(&raw_account.data).unwrap();
    let address_lookup_table_account = AddressLookupTableAccount {
        key: address_lookup_table_key,
        addresses: address_lookup_table.addresses.to_vec(),
    };

    let blockhash = client.get_latest_blockhash().await.unwrap();
    let tx = VersionedTransaction::try_new(
        VersionedMessage::V0(v0::Message::try_compile(
            &payer.pubkey(),
            instructions,
            &[address_lookup_table_account],
            blockhash,
        ).unwrap()),
        &[payer],
    ).unwrap();

    assert!(!tx.message.address_table_lookups().unwrap().is_empty());
    tx
}