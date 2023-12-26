use solana_client::nonblocking::rpc_client::RpcClient;

use anchor_client::solana_client::rpc_config::RpcSendTransactionConfig;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::read_keypair_file;
use anchor_client::solana_sdk::signature::{Keypair, Signer};
use std::str::FromStr;
use anchor_client::{Client, Cluster};

use client::monitor::pools::SerumPool;

use solana_sdk::transaction::Transaction;

use std::rc::Rc;

use std::collections::HashMap;

use std::vec;




use client::constants::*;
use client::monitor::pools::pool::{PoolDir, PoolType};
use client::utils::read_json_dir;

use indicatif::ProgressBar;

fn main() {
    let cluster = Cluster::Mainnet;

    env_logger::init();
    // let owner_kp_path = "../../../mainnet.key";
    let owner_kp_path = "/root/.config/solana/id.json";
    let owner = read_keypair_file(owner_kp_path).unwrap();
    let oo_path = match cluster {
        Cluster::Localnet => "./serum_open_orders.json",
        Cluster::Mainnet => {
            "./serum_open_orders.json"
        }
        _ => panic!("clsuter {} not supported", cluster),
    };
    let oo_str = std::fs::read_to_string(oo_path).unwrap();
    let oo: HashMap<String, String> = serde_json::from_str(&oo_str).unwrap();
    let mut open_orders = vec![];

    let mut market_to_open_orders = HashMap::new();

    for (market, oo) in oo {
       open_orders.push(market.clone());
       market_to_open_orders.insert(
           market,
           oo
       );

    }
    println!("open orders: {:?}", open_orders.len());   
    // ** setup RPC connection
    let connection = RpcClient::new_with_commitment("https://rpc.shyft.to?api_key=jdXnGbRsn0Jvt5t9".to_string(), CommitmentConfig::confirmed());

    let provider = Client::new_with_options(cluster, Rc::new(owner), CommitmentConfig::confirmed());
    let _program = provider.program(*ARB_PROGRAM_ID).unwrap();
    let owner = read_keypair_file(owner_kp_path).unwrap();

    let serum_dir = PoolDir {
        pool_type: PoolType::SerumPoolType,
        dir_path: "../pools/serum/".to_string(),
    };

    let mut pool_paths = read_json_dir(&serum_dir.dir_path);

    let max_space = 3228;
    let max_rent_exemption_amount = connection
        .get_minimum_balance_for_rent_exemption(max_space)
        .unwrap();
    let total_fee = max_rent_exemption_amount * pool_paths.len() as u64;
    let lamports_per_sol = 1000000000;
    let sol_fee = total_fee as f64 / lamports_per_sol as f64;
    println!(
        "# open orders: {:?} USDC cost: {:?}",
        pool_paths.len(),
        sol_fee * 90_f64
    );

    // return;

    let pb = ProgressBar::new(pool_paths.len() as u64);
    pool_paths.reverse();
    for pool_path in pool_paths {

        let json_str = std::fs::read_to_string(&pool_path).unwrap();
        let pool: SerumPool = serde_json::from_str(&json_str).unwrap();
        if open_orders.contains(&pool.own_address.0.to_string()) {
            println!("already have open orders for {}", pool.own_address.0);
            continue;
        }

        // do a swap and check the amount
        let mut PROGRAM_LAYOUT_VERSIONS = HashMap::new();
        PROGRAM_LAYOUT_VERSIONS.insert("4ckmDgGdxQoPDLUkDT3vHgSAkzA3QRdNq5ywwY4sUSJn", 1);
        PROGRAM_LAYOUT_VERSIONS.insert("BJ3jrUzddfuSrZHXSCxMUUQsjKEyLmuuyZebkcaFp2fg", 1);
        PROGRAM_LAYOUT_VERSIONS.insert("EUqojwWA2rd19FZrzeBncJsm38Jm1hEhE3zsmX3bRc2o", 2);
        PROGRAM_LAYOUT_VERSIONS.insert("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin", 3);

        let _LAYOUT_V1_SPAN = 3220;
        let LAYOUT_V2_SPAN = 3228;

        let space = LAYOUT_V2_SPAN;

        let open_orders = Keypair::new();

        let rent_exemption_amount = connection
            .get_minimum_balance_for_rent_exemption(space)
            .unwrap();

            let create_account_ix = solana_sdk::system_instruction::create_account(
                &owner.pubkey(),
                &open_orders.pubkey(),
                rent_exemption_amount,
                space as u64,
                &SERUM_PROGRAM_ID,
            );
    
            let init_ix = openbook_dex::instruction::init_open_orders(
                &Pubkey::from_str("srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX").unwrap(),
                &open_orders.pubkey(),
&owner.pubkey(),
            &    pool.own_address.0,
                None
            ).unwrap();
            let tx = Transaction::new_signed_with_payer(
                &[create_account_ix.clone(), init_ix.clone()],
                Some(&owner.pubkey()),
                &[(&owner), (&open_orders)],
                connection.get_latest_blockhash().unwrap(),
            );
            let signature = connection
                                .send_transaction_with_config(
                                    &tx,
                                    RpcSendTransactionConfig {
                                        skip_preflight: true,
                                        ..RpcSendTransactionConfig::default()
                                    },
                                )
                                ;
                                if signature.is_err() {
                                    println!("error: {:#?}", signature.err().unwrap()); 
                                }
                                else {
                            println!("signature: {:?}", signature.unwrap());
                                }

        market_to_open_orders.insert(
            pool.own_address.0.to_string(),
            open_orders.pubkey().to_string(),
        );

        // save open orders accounts as .JSON
        let json_market_oo = serde_json::to_string(&market_to_open_orders).unwrap();
        std::fs::write("./serum_open_orders.json", json_market_oo).unwrap();

        pb.inc(1);
    }

    // save open orders accounts as .JSON
    let json_market_oo = serde_json::to_string(&market_to_open_orders).unwrap();
    std::fs::write("./serum_open_orders.json", json_market_oo).unwrap();
}
