



use std::collections::HashSet;


use rand::seq::SliceRandom;
use solana_address_lookup_table_program::instruction::{extend_lookup_table, create_lookup_table};
use solana_client::nonblocking::rpc_client::RpcClient;



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
    pub yield_value: u128,  // This should be the actual yield value
}
impl Eq for Edge {}
impl Edge {
    fn set_yield_value(&mut self, yield_value: u128) {
        self.yield_value = yield_value;
    }
    fn get_yield_value(&self) -> u128 {
        self.yield_value
    }
}
impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to
    }
}
#[derive(Clone)]
pub struct Path {
   pub nodes: Vec<usize>,
   pub pool_idxs: Vec<PoolQuote>,
   pub yields: Vec<u128>,
   current_node: usize,

    total_yield: u128,
    total_length: usize,
    most_recent_yield: u128,
    most_recent_yield_usd : f64,
}

impl Path {
    fn new(start_mint_idx: usize, amount: u128) -> Self {
        Self {
            nodes: vec![start_mint_idx],
            pool_idxs: vec![],
            yields: vec![amount],
            current_node: start_mint_idx,
            total_yield: 0,
            total_length: 0,
            most_recent_yield: 0,
            most_recent_yield_usd: 0.0,
        }
    }

    fn last_node(&self) -> usize {
        self.nodes.last().copied().unwrap_or_default()
    }

    fn contains_node(&self, node: usize) -> bool {
        self.nodes.contains(&node)
    }

    fn extend(&mut self, edge: usize, additional_yield: u128, pool_idx: PoolQuote, poolidx_1: u128) {
       
        self.nodes.push(edge);
        self.pool_idxs.push(pool_idx.clone());
        self.yields.push(additional_yield);
        self.current_node = edge;
        self.total_yield += (additional_yield as f64 / poolidx_1 as f64) as u128;
        self.total_length += 1;
        self.most_recent_yield = additional_yield;
        self.most_recent_yield_usd = additional_yield as f64 / poolidx_1 as f64;

    }
}

impl Ord for Path {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.most_recent_yield_usd as u64).cmp(&(other.most_recent_yield_usd as u64))
    }
}

impl PartialOrd for Path {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Eq for Path {}
impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        self.total_yield == other.total_yield
    }
}


impl Arbitrager {  
    pub async fn find_yield(&mut self, start_mint_idx: usize, max_hops: usize, max_output: u128) -> Option<Path> {
        let mut max_heap = BinaryHeap::new();
        let mut best_paths = HashMap::new();
        let mut visited = HashSet::new();
        
        max_heap.push(Path {
            current_node: start_mint_idx,
            total_yield: 0,
            nodes: vec![start_mint_idx],
            pool_idxs: vec![],
            yields: vec![max_output],
            total_length: 0,
            most_recent_yield: max_output,
            most_recent_yield_usd: max_output as f64,
        });
    
        while let Some(mut path) = max_heap.pop() {
            if visited.contains(&path.current_node) {
                continue;
            }
            visited.insert(path.current_node);
    
            if path.nodes.len() > max_hops {
                continue;
            }
            let current_yield = path.yields.last().copied().unwrap_or_default();
            let mut edges_to_update = vec![];
            let mut futures = FuturesUnordered::new();
            if let Some(edges) = self.graph_edges.get(path.current_node) {
                for edge in edges.iter() {
                    let mut edge = *edge;
                    let to = edge.to;
                    if edge.get_yield_value() == 0 {
                        let old_edge = edge;
                        edge.set_yield_value(self.get_yield(edge, start_mint_idx, 1_000, 0, to).await.0);
                       
                        edges_to_update.push((old_edge, edge));
                    }
                    
                    futures.push(self.compute_yield(edge, current_yield));
                }
            }

            


            while let Some((yield_value, quote, mut edge)) = futures.next().await {
                self.process_yield(&mut edge,  &mut max_heap, &mut best_paths, &mut path, yield_value, quote, start_mint_idx, max_hops, max_output).await;
                // so edge is right here
                // update graph
                


            }
            drop(futures);
            for (old_edge, edge) in edges_to_update {
                 let edges = self.graph_edges.get_mut(old_edge.to).unwrap();
                 edges.remove(&old_edge);
                 edges.insert(edge);
             }
            if best_paths.len() > 5 {
                break;
            }
        }
        // Extract the best path overall or for a specific target node
        // This returns the path with the highest yield overall
         best_paths.into_iter()
                .max_by_key(|&(_, yield_value)| yield_value)
                .map(|(node, _)| {
                    max_heap.into_iter().find(|path| path.current_node == node).unwrap()
                })
        
    }
    async fn process_yield(
        &self,
        edge: &mut Edge,
        max_heap: &mut BinaryHeap<Path>,
        best_paths: &mut HashMap<usize, u128>,
        path: &mut Path,
        yield_value: u128,
        quote: Option<PoolQuote>, // Replace with actual type
        start_mint_idx: usize,
        max_hops: usize,
        max_output: u128,
    ) -> Edge {
        if yield_value > 0 {
            let mut new_path = path.clone();
            if quote.is_none() {
                return *edge;
            }
            let quote = quote.unwrap();
            let poolidx_1 = edge.get_yield_value();
            
            
            new_path.extend(edge.to, yield_value, quote.clone(), poolidx_1);
            let new_total_yield = new_path.total_yield;
            // Update best path for this node
            if new_total_yield > *best_paths.get(&edge.to).unwrap_or(&0) && new_total_yield < 1_000_000 {

                println!("new best path for node {} with yield {}", edge.to, new_total_yield);
                if Arbitrager::is_invalid_path(&new_path.clone(), start_mint_idx, max_hops, max_output) {
                    return *edge;
                }
                if Arbitrager::is_valid_path(&new_path, start_mint_idx, max_hops, max_output) {
                    println!("new GUD best path for node {} with yield {}", edge.to, new_total_yield);
                    best_paths.insert(edge.to, new_total_yield);
                }
                
            } 
            max_heap.push(new_path.clone());
        }
        *edge
    }
    
    fn is_valid_path(path: &Path, start_mint_idx: usize, _max_hops: usize, _max_output: u128) -> bool {
        path.last_node() == start_mint_idx && path.total_yield > 0 && path.yields[path.yields.len()-1] as f64 <= path.yields[0] as f64 * 1.2 && path.yields[path.yields.len()-1] as f64
         > path.yields[0] as f64 * 1.002 && path.total_length > 2
    }

    fn is_invalid_path(path: &Path, _start_mint_idx: usize, _max_hops: usize, _max_output: u128) -> bool {
        path.total_length > _max_hops
    }
    #[async_recursion::async_recursion]

    async fn get_yield(&self, edge: Edge, start_mint_idx: usize, amount: u128, current_hops: usize, _to: usize) -> (u128, Option<PoolQuote>, Edge) {
        let mut from = edge.from;
        let to = edge.to;
        if current_hops == 0 {
            from = start_mint_idx;
        }
        if current_hops > 5 {
            return (0, None, edge);
        }
        let mut futures = FuturesUnordered::new();

        if let Some(edges) = self.graph_edges.get(from) {
            if edges.iter()
                .any(|edge| edge.to == to) {
                return self.compute_yield(edge, amount,).await;
            }
                for edge in edges.clone() {
                    let edge = edge;
                    
                        futures.push(self.find_yield_recursive(edge, start_mint_idx, amount, current_hops+1, to));
                     
                }
        }
        while let Some((yield_value, quote, edge)) = futures.next().await {
            if yield_value > 0 {
                return (yield_value, quote, edge);
            }
        }
        (0, None, edge)
    }    

    async fn compute_yield(&self, edge: Edge, amount: u128) -> (u128, Option<PoolQuote>, Edge) {
        let from = edge.from;
        let to = edge.to;
        let pool = self.graph.0.get(&PoolIndex(from)).and_then(|p| p.0.get(&PoolIndex(to)));
      
        if pool.is_none(){
            return (0, None, edge);
        }
        let pool = pool.unwrap();
        
        let timestamp = Arbitrager::current_timestamp() % pool.len();
        
        let mut pool = pool.iter().nth(timestamp).unwrap().clone();
        let connection: solana_client::rpc_client::RpcClient = solana_client::rpc_client::RpcClient::new(self.cluster.url().to_string());
        let new_balance = pool.get_quote_with_amounts_scaled(amount, &self.token_mints[from], &self.token_mints[to], &Arc::new(connection));

        if new_balance == 0 {
            let mut blacklist = self.blacklist.try_lock().unwrap();
            blacklist.push((from, to));
            drop(blacklist);
            return (0, None, edge);
        }
        (new_balance, Some(pool.clone()), edge)
    }
    async fn find_yield_recursive(&self, edge: Edge, start_mint_idx: usize, amount: u128, current_hops: usize, to: usize) -> (u128, Option<PoolQuote>, Edge) {
        if current_hops > 5 {
            return (0, None, edge);
        }
        
        let result = self.get_yield(edge, start_mint_idx, amount, current_hops, to).await;
    
        if let (yield_value, quote, edge) = result {
            if yield_value > 0 {
                return (yield_value, quote, edge);
            }
        }
    
        (0, None, edge)
    }


    fn current_timestamp() -> usize {
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
        let mut swap_start_amount = amounts[0];


        for i in 0..mint_idxs.len() - 1 {

            let program: Program<Arc<Keypair>> = provider.program(*crate::constants::ARB_PROGRAM_ID).unwrap();

            let [mint_idx0, mint_idx1] = [mint_idxs[i], mint_idxs[i + 1]];
            println!("mint idxs are {} {}", mint_idx0, mint_idx1);
            let pool = &pools[i];
            let pool3 = pool.clone();
            let token_mints = token_mints.clone();
            println!("getting quote with amounts scaled {} {} {} ", i, token_mints[mint_idx0].clone(), token_mints[mint_idx1].clone());
            let swap_ix = pool3.swap_ix(token_mints[mint_idx0],
            token_mints[mint_idx1], swap_start_amount,
            owner.clone().pubkey(), &program);
            println!("swap ix is {:?}", swap_ix.1.len());
             swap_start_amount = amounts[i+1];
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
               let hm = connection
                    .send_transaction(&VersionedTransaction::try_new(
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