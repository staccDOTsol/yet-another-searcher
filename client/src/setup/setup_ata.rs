use std::vec;
use log::warn;

use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::signature::read_keypair_file;
use anchor_client::solana_sdk::signature::Signer;

use solana_sdk::instruction::Instruction;
use solana_sdk::transaction::Transaction;
use solana_sdk::instruction::AccountMeta;
use solana_sdk::system_program;


use client::constants::*;
use client::monitor::pools::pool::{pool_factory, PoolType};
use client::monitor::pools::pool::PoolDir;
use client::serialize::token::unpack_token_account;
use client::utils::{derive_token_address, read_json_dir};

fn main() {

    env_logger::init();
    let owner_kp_path = "/Users/stevengavacs/.config/solana/id.json";
    let owner = read_keypair_file(owner_kp_path).unwrap();

    // ** setup RPC connection
    let connection = RpcClient::new_with_commitment(
        "https://rpc.shyft.to?api_key=jdXnGbRsn0Jvt5t9",
        CommitmentConfig::processed(),
    );
    let send_tx_connection =
        RpcClient::new_with_commitment("https://rpc.shyft.to?api_key=jdXnGbRsn0Jvt5t9", CommitmentConfig::processed());
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
    


    let mut token_mints = vec![];
    for pool_dir in pool_dirs {
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
            for mint in pool_mints {
                if !token_mints.contains(&mint) {
                    token_mints.push(mint);
                }
            }
        }
    }

    // make sure all tokens have ATA
    // print initial balances
    let mut user_token_addrs = vec![];
    for mint in &token_mints {
        let user_token_addr = derive_token_address(&owner.pubkey(), mint);
        user_token_addrs.push(user_token_addr);
    }
    // get pool amounts in single RPC
    let mut token_amounts = vec![];
    // max 100 accounts per get_multiple_accounts
    for token_addr_chunk in user_token_addrs.chunks(99) {
        let token_accounts = connection.get_multiple_accounts(token_addr_chunk).unwrap();
        for account in token_accounts {
            let amount = match account {
                Some(account) => {
                    let data = account.data;

                    unpack_token_account(&data).amount as i64
                }
                None => -1_i64, // no ATA!
            };
            token_amounts.push(amount);
        }
    }

    // print balances + create ATA acccounts
    let mut create_ata_ixs = vec![];
    let n = token_amounts.len();
    for i in 0..n {
        let mint = &token_mints[i];
        let amount = token_amounts[i];
        if amount >= 0 {
            println!("balance {}: {}", mint, amount);
            continue;
        }
        let addr = user_token_addrs[i];

        // create ATA!
        println!("creating ATA for Token {:?}...", mint);
        let accounts = vec![
            AccountMeta::new(owner.pubkey(), true),
            AccountMeta::new(addr, false),
            AccountMeta::new_readonly(owner.pubkey(), false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(*TOKEN_PROGRAM_ID, false),
            AccountMeta::new_readonly(solana_sdk::sysvar::rent::id(), false),
        ];
        let ix = Instruction {
            program_id: *ASSOCIATED_TOKEN_PROGRAM_ID,
            accounts,
            data: vec![],
        };

        create_ata_ixs.push(ix);
    }

    println!(
        "creating {} / {} total token accounts...",
        create_ata_ixs.len(),
        n
    );

    for chunck_ixs in create_ata_ixs.chunks(9) {
        let tx = {
            let recent_hash = send_tx_connection.get_latest_blockhash().unwrap();
            Transaction::new_signed_with_payer(
                chunck_ixs,
                Some(&owner.pubkey()),
                &[&owner],
                recent_hash,
            )
        };
        println!("creating {} token accounts in tx...", chunck_ixs.len());
        let signature = send_tx_connection.send_transaction(&tx).unwrap();
        println!("signature: {}", signature);
    }

    println!("done setup :)");
}
