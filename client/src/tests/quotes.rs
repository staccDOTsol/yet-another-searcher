use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::read_keypair_file;
use anchor_client::solana_sdk::signature::{Keypair, Signer};
use anchor_client::{Client, Cluster, Program};
use std::str::FromStr;
use solana_sdk::transaction::Transaction;
use spl_token::instruction::mint_to;

use std::rc::Rc;
use std::vec;

use tmp::accounts as tmp_accounts;
use tmp::instruction as tmp_ix;

use crate::constants::*;
use crate::pool::{pool_factory, PoolOperations, PoolType};
use crate::utils::{derive_token_address, read_json_dir};

#[test]
fn serum() {
    let pool_dir = "../pools/serum/".to_string();
    let pool_type = PoolType::SerumPoolType;
    test_all_pool_quotes(pool_dir, pool_type);
}

#[test]
fn aldrin() {
    let pool_dir = "../pools/aldrin/".to_string();
    let pool_type = PoolType::AldrinPoolType;
    test_all_pool_quotes(pool_dir, pool_type);
}

#[test]
fn saber() {
    let pool_dir = "../pools/saber/".to_string();
    let pool_type = PoolType::SaberPoolType;
    test_all_pool_quotes(pool_dir, pool_type);
}

#[test]
fn mercurial() {
    let pool_dir = "../pools/mercurial/".to_string();
    let pool_type = PoolType::MercurialPoolType;
    test_all_pool_quotes(pool_dir, pool_type);
}

#[test]
fn orca() {
    let pool_dir = "../pools/orca/".to_string();
    let pool_type = PoolType::OrcaPoolType;
    test_all_pool_quotes(pool_dir, pool_type);
}

fn test_all_pool_quotes(pool_dir: String, pool_type: PoolType) {
    // setup stuff
    let cluster = Cluster::Mainnet;
    let connection = RpcClient::new_with_commitment(cluster.url(), CommitmentConfig::recent());

    // let owner_kp_path = "/Users/vbetsun/.config/solana/uwuU3qc2RwN6CpzfBAhg6wAxiEx138jy5wB3Xvx18Rw.json";
    let owner_kp_path = "../mainnet-fork/localnet_owner.key";
    // let owner_kp_path = "../program/target/deploy/tmp-keypair.json";
    // setup anchor things
    let owner = read_keypair_file(owner_kp_path.clone()).unwrap();
    println!("owner: {}", owner.pubkey());

    let provider = Client::new_with_options(cluster, Rc::new(owner), CommitmentConfig::recent());
    let program = provider.program(*ARB_PROGRAM_ID).unwrap();
    let owner = read_keypair_file(owner_kp_path.clone()).unwrap();

    let pool_paths = read_json_dir(&pool_dir);
    let mut err_count = 0;
    let n_pools = pool_paths.len();
    println!("found {} pools...", n_pools);

    for pool_path in pool_paths {
        let contents = std::fs::read_to_string(&pool_path).unwrap();
        let mut pool = pool_factory(&pool_type, &contents);

        // println!("{}", pool_path);
        let err_flag = test_pool_quote(&mut pool, &pool_path, &connection, &owner);
        err_count += err_flag;
    }
    println!("POOL ERRORS: {} / {}", err_count, n_pools);
}

fn test_pool_quote(
    pool: &mut Box<dyn PoolOperations>,
    pool_path: &str,
    connection: &RpcClient,
    owner: &Keypair,
) -> u64 {
    1
}
