use anchor_client::Cluster;
use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::signature::read_keypair_file;
use client::config::runtime::RuntimeConfig;
use solana_sdk::account::Account;
use core::panic;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use clap::Parser;

use client::monitor::setup::*;
use client::identify::search::*;
use client::config::*;
// use log::{debug, warn};
type ShardedDb = Arc<Mutex<HashMap<String, Account>>>;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(short, long)]
    pub config: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // initialize startup config
    let config: &str = args.config.as_str();
    let conf = initialize_config(config);
    // initialize runtime config
    let mut runtime_conf: RuntimeConfig;
    runtime_conf.setup(conf);

    // start searcher thread, pass reference to runtime config
    search(graph);

    // start execute thread
    pooldir
    // watch for shutdown signal
}
