use solana_client::nonblocking::rpc_client::RpcClient;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::signature::read_keypair_file;
use anchor_client::solana_sdk::signature::Keypair;
use anchor_client::Cluster;
use crate::monitor::pools::{pool_factory, PoolOperations, PoolType};
use crate::utils::read_json_dir;

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
    let _cluster = Cluster::Mainnet;
    let connection = RpcClient::new_with_commitment(
        "https://rpc.shyft.to?api_key=jdXnGbRsn0Jvt5t9".to_string(),
        CommitmentConfig::processed(),
    );

    // let owner_kp_path = "/Users/vbetsun/.config/solana/uwuU3qc2RwN6CpzfBAhg6wAxiEx138jy5wB3Xvx18Rw.json";
    let owner_kp_path = "../mainnet-fork/localnet_owner.key";
    // let owner_kp_path = "../program/target/deploy/tmp-keypair.json";
    // setup anchor things
    // println!("owner: {}", owner.pubkey());
    // let program = provider.program(*ARB_PROGRAM_ID).unwrap();
    let owner = read_keypair_file(owner_kp_path).unwrap();

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
    _pool: &mut Box<dyn PoolOperations>,
    _pool_path: &str,
    _connection: &RpcClient,
    _owner: &Keypair,
) -> u64 {
    1
}
