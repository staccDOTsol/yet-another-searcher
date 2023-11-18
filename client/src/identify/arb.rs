use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_client::rpc_config::RpcSendTransactionConfig;
use solana_program::{
    address_lookup_table::AddressLookupTableAccount,
    message::{v0, Message, VersionedMessage},
};
use solana_transaction_status::UiTransactionEncoding;
use std::path::PathBuf;

use std::sync::{Arc, Mutex};

type ShardedDb = Arc<Mutex<HashMap<String, Account>>>;
use bincode::serialize;
use solana_sdk::{
    account::Account,
    address_lookup_table::{
        instruction::ProgramInstruction,
        program::{check_id, id},
        state::{
            AddressLookupTable, LookupTableMeta, LookupTableStatus, ProgramState,
            LOOKUP_TABLE_MAX_ADDRESSES, LOOKUP_TABLE_META_SIZE,
        },
    },
    clock::Slot,
    feature_set,
    instruction::InstructionError,
    program_utils::limited_deserialize,
    pubkey::{Pubkey, PUBKEY_BYTES},
    system_instruction,
};
use std::str::FromStr;
use std::thread;
use std::time;

use anyhow::Result;
use serde_json::json;
use solana_client::rpc_request::RpcRequest;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::{
    self,
    commitment_config::CommitmentConfig,
    instruction::Instruction,
    signature::{read_keypair_file, Keypair, Signature},
    signer::Signer,
    transaction::{Transaction, VersionedTransaction},
};

use derive_more::FromStr;
use num_traits::pow;
use solana_client::client_error::ClientError;
use solana_sdk::signers::Signers;
use structopt::StructOpt;

use flash_loan_sdk::instruction::{flash_borrow, flash_repay};
use flash_loan_sdk::{available_liquidity, flash_loan_fee, get_reserve, FLASH_LOAN_ID};

use anchor_client::{Cluster, Program};
use std::collections::{HashMap, HashSet};

use std::borrow::{Borrow, BorrowMut};
use std::rc::Rc;
use std::vec;

use log::info;

use tmp::accounts as tmp_accounts;
use tmp::instruction as tmp_ix;

use crate::monitor::pools::{PoolOperations, PoolType};

use crate::utils::{derive_token_address, PoolGraph, PoolIndex, PoolQuote};

// from https://github.com/solana-labs/solana/blob/10d677a0927b2ca450b784f750477f05ff6afffe/sdk/program/src/message/versions/v0/mod.rs#L209
fn create_tx_with_address_table_lookup(
    client: &RpcClient,
    instructions: &[Instruction],
    address_lookup_table_key: Pubkey,
    payer: &Keypair,
) -> VersionedTransaction {
    let raw_account = client.get_account(&address_lookup_table_key).unwrap();
    let address_lookup_table = AddressLookupTable::deserialize(&raw_account.data).unwrap();
    let address_lookup_table_account = AddressLookupTableAccount {
        key: address_lookup_table_key,
        addresses: address_lookup_table.addresses.to_vec(),
    };

    let blockhash = client.get_latest_blockhash().unwrap();
    let tx = VersionedTransaction::try_new(
        VersionedMessage::V0(
            v0::Message::try_compile(
                &payer.pubkey(),
                instructions,
                &[address_lookup_table_account],
                blockhash,
            )
            .unwrap(),
        ),
        &[payer],
    )
    .unwrap();

    assert!(tx.message.address_table_lookups().unwrap().len() > 0);
    tx
}
pub struct Arbitrager {
    pub token_mints: Vec<Pubkey>,
    pub graph_edges: Vec<HashSet<usize>>, // used for quick searching over the graph
    pub graph: PoolGraph,
    pub cluster: Cluster,
    // vv -- need to clone these explicitly -- vv
    pub owner: Rc<Keypair>,
    pub connection: RpcClient,
}
unsafe impl Send for Arbitrager {}
unsafe impl Sync for Arbitrager {}

impl Arbitrager {
    pub fn brute_force_search(
        &self,
        start_mint_idx: usize,
        init_balance: u128,
        curr_balance: u128,
        path: Vec<usize>,
        pool_path: Vec<PoolQuote>,
        sent_arbs: &mut HashSet<String>,
        transfer: Instruction,
        page_config: &ShardedDb,
    ) {
        let src_curr = path[path.len() - 1]; // last mint
        let src_mint = self.token_mints[src_curr];

        let out_edges = &self.graph_edges[src_curr];

        // path = 4 = A -> B -> C -> D
        // path >= 5 == not valid bc max tx size is swaps
        if path.len() == 4 {
            return;
        };

        for dst_mint_idx in out_edges {
            if path.contains(dst_mint_idx) && *dst_mint_idx != start_mint_idx {
                continue;
            }
            if self.graph.0.get(&PoolIndex(src_curr)).is_none()
                || self
                    .graph
                    .0
                    .get(&PoolIndex(src_curr))
                    .unwrap()
                    .0
                    .get(&PoolIndex(*dst_mint_idx))
                    .is_none()
            {
                continue;
            }
            let mut pools = &mut self
                .graph
                .0
                .get(&PoolIndex(src_curr))
                .unwrap()
                .0
                .get(&PoolIndex(*dst_mint_idx))
                .unwrap();

            let dst_mint_idx = *dst_mint_idx;
            let dst_mint = self.token_mints[dst_mint_idx];

            for pool in pools.iter_mut() {
                let mut new_balance;
                new_balance = pool.0.borrow_mut().get_quote_with_amounts_scaled(
                    curr_balance,
                    &src_mint,
                    &dst_mint,
                    &page_config,
                );

                let mut new_path = path.clone();
                new_path.push(dst_mint_idx);

                let mut new_pool_path = pool_path.clone();
                new_pool_path.push(pool.clone()); // clone the pointer
                let mut new_init_balance = init_balance;

                if dst_mint_idx == start_mint_idx {
                    // println!("{:?} -> {:?} (-{:?})", init_balance, new_balance, init_balance - new_balance);
                    let mut mult = 1.0002;

                    println!("new balance: {:?}", new_balance);
                    // if new_balance > init_balance - 1086310399 {
                    if new_balance as f64 > new_init_balance as f64 * mult {
                        // ... profitable arb!
                        println!("found arbitrage: {:?} -> {:?}", init_balance, new_balance);

                        // check if arb was sent with a larger size
                        // key = {mint_path}{pool_names}
                        let mint_keys: Vec<String> =
                            new_path.clone().iter_mut().map(|i| i.to_string()).collect();
                        let pool_keys: Vec<String> =
                            new_pool_path.iter().map(|p| p.0.get_name()).collect();
                        let arb_key = format!("{}{}", mint_keys.join(""), pool_keys.join(""));
                        if sent_arbs.contains(&arb_key) {
                            println!("arb already sent...");
                            continue; // dont re-send an already sent arb -- bad for network
                        } else {
                            sent_arbs.insert(arb_key);
                        }
                        let ookp = Keypair::new();
                        let mut ixs;

                        ixs = vec![self
                            .get_arbitrage_instructions(
                                new_init_balance,
                                &new_path,
                                &new_pool_path,
                                vec![&mut Keypair::new()],
                                &ookp,
                            )
                            .0
                            .concat()];
                        let owner: &Keypair = self.owner.borrow();
                        let url = "https://rpc.shyft.to?api_key=jdXnGbRsn0Jvt5t9";
                        let reserve =
                            Pubkey::from_str("8qow5YNnT9NfvxVsxYMiKV4ddggT5gEe3uLUvjQ6uYaZ")
                                .unwrap();
                        let program_id =
                            Pubkey::from_str("F1aShdFVv12jar3oM2fi6SDqbefSnnCVRzaxbPH3you7")
                                .unwrap();
                        println!("=====================Setup=====================");
                        println!("Solana cluster       : {}", url);
                        println!("Flash loan program id: {}", program_id);
                        println!("Flash loan reserve   : {}", reserve);
                        println!("===============================================");
                        let wallet =
                            Pubkey::from_str("Et1ZTDXDQ9V7TyyZCFX6nAJmxffpsD1iFerYpArQVRAf")
                                .unwrap();

                        let rpc_client = RpcClient::new_with_commitment(
                            url.clone(),
                            CommitmentConfig::confirmed(),
                        );

                        // From Solana RPC rate limit perspective it is more efficient to load Reserve once from the chain and then
                        // use it in subsequent calls.
                        let reserve = get_reserve(&reserve, &rpc_client).expect("Getting reserve");

                        let authority_kp =
                            read_keypair_file("/Users/stevengavacs/.config/solana/id.json")
                                .expect("Reading authority key pair file");
                        let src_ata = derive_token_address(&self.owner.pubkey(), &src_mint);

                        // flatten to Vec<Instructions>
                        ixs.push(vec![transfer.clone()]);

                        let versioned_tx = create_tx_with_address_table_lookup(
                            &rpc_client,
                            &ixs.concat(),
                            Pubkey::from_str("xcacBTrGeNbZqxZJBhyjPPpNV6ZdnKJf3F27YjqSSzy")
                                .unwrap(),
                            &owner,
                        );
                        let serialized_versioned_tx = serialize(&versioned_tx).unwrap();
                        println!(
                            "The serialized versioned tx is {} bytes",
                            serialized_versioned_tx.len()
                        );
                        let serialized_encoded = base64::encode(serialized_versioned_tx);
                        let config = RpcSendTransactionConfig {
                            skip_preflight: false,
                            preflight_commitment: Some(CommitmentLevel::Processed),
                            encoding: Some(UiTransactionEncoding::Base64),
                            ..RpcSendTransactionConfig::default()
                        };

                        let signature = rpc_client.send::<String>(
                            RpcRequest::SendTransaction,
                            json!([serialized_encoded, config]),
                        );
                        if signature.is_err() {
                            println!("Error: {:?}", signature);
                        } else {
                            let result = rpc_client.confirm_transaction_with_commitment(
                                &Signature::from_str(signature.unwrap().as_str()).unwrap(),
                                CommitmentConfig::finalized(),
                            );
                            if result.is_err() {
                                println!("Error: {:?}", result);
                            } else {
                                println!("Result: {:?}", result);
                            }
                        }
                    }
                } else if !path.contains(&dst_mint_idx) {
                    // ... search deeper
                    self.brute_force_search(
                        start_mint_idx,
                        init_balance,
                        new_balance,   // !
                        new_path,      // !
                        new_pool_path, // !
                        sent_arbs,
                        transfer.clone(),
                        page_config,
                    );
                }
            }
        }
    }

    fn get_arbitrage_instructions<'a>(
        &self,
        swap_start_amount: u128,
        mint_idxs: &Vec<usize>,
        pools: &Vec<PoolQuote>,
        mut signers: Vec<&mut Keypair>,

        ookp: &Keypair,
    ) -> (Vec<Vec<Instruction>>, bool) {
        // gather swap ixs
        let mut ixs = vec![];
        let swap_state_pda =
            (Pubkey::from_str("8cjtn4GEw6eVhZ9r1YatfiU65aDEBf1Fof5sTuuH6yVM").unwrap());
        let src_mint = self.token_mints[mint_idxs[0]];
        let src_ata = derive_token_address(&self.owner.pubkey(), &src_mint);

        // setup anchor things
        let owner = solana_sdk::signer::keypair::read_keypair_file(
            "/Users/stevengavacs/.config/solana/id.json",
        )
        .unwrap();
        let rc_owner = Rc::new(owner);
        let provider = anchor_client::Client::new_with_options(
            Cluster::Mainnet,
            rc_owner.clone(),
            solana_sdk::commitment_config::CommitmentConfig::recent(),
        );
        let program = provider.program(*crate::constants::ARB_PROGRAM_ID).unwrap();

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
        let mut flag = false;
        for i in 0..mint_idxs.len() - 1 {
            let [mint_idx0, mint_idx1] = [mint_idxs[i], mint_idxs[i + 1]];
            let [mint0, mint1] = [self.token_mints[mint_idx0], self.token_mints[mint_idx1]];
            let pool = &pools[i];
            let mut swap_ix = pool.0.swap_ix(
                &self.owner.pubkey(),
                &mint0,
                &mint1,
                ookp,
                swap_start_amount,
            );
            ixs.push(swap_ix.1.clone());
            for ix in swap_ix.1 {
                for key in ix.accounts.iter() {
                    println!("key: {:?}", key.pubkey);
                }
            }
            let pool_type = pool.0.get_pool_type();
            match pool_type {
                PoolType::OrcaPoolType => {}
                PoolType::MercurialPoolType => {}
                PoolType::SaberPoolType => {}
                PoolType::AldrinPoolType => {}
                PoolType::SerumPoolType => {
                    flag = true;
                }
            }
        }

        ixs.concat();
        (ixs, flag)
    }
}
