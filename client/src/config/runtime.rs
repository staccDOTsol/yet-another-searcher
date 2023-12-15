// runtime config is a config that is used across all threads
// it encapsulates the start up config and is shared with all 
// threads by reference
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::signature::read_keypair_file;
use anchor_client::solana_sdk::signature::Keypair;
use anchor_client::Cluster;
use solana_sdk::account::Account;
use crate::utils::PoolGraph;
use crate::config::startup::StartupConfig;
use crate::monitor::setup::*;
use crate::monitor::pools::*;

type ShardedDb = Arc<Mutex<HashMap<String, Account>>>;
pub struct RuntimeConfig {
    startup_config: StartupConfig,
    // solana specific stuff
    cluster: Cluster,
    connection: RpcClient,
    owner: Keypair,
    rc_owner: Rc<Keypair>,

    // TODO: create other application specific shared configurations
    page_config: Arc<Arc<Mutex<HashMap<String, Account>>>>,
    graph: PoolGraph,
    pool_dirs: Vec<PoolDir>,
}
impl RuntimeConfig {
    pub fn setup(mut self, sc: StartupConfig) {
        self.set_startup_config(sc);
        self.parse_cluster();
        self.create_connection();
        self.set_owner();
        self.init_page_config();
        self.init_graph();
        self.get_pool_dirs();
    }
    
    pub fn set_startup_config(mut self, sc: StartupConfig){
        self.startup_config = sc;
        return 
    }

    pub fn parse_cluster(mut self) {
        let cluster = match self.startup_config.cluster.as_str() {
            "localnet" => Cluster::Localnet,
            "mainnet" => Cluster::Mainnet,
            _ => panic!("invalid cluster type"),
        };

        self.cluster = cluster;
        return
    }

    pub fn create_connection(mut self) {
        println!("using connection: {}", self.startup_config.rpc_url);
        let connection = RpcClient::new_with_commitment(self.startup_config.rpc_url, CommitmentConfig::processed());
        return
    }

    pub fn set_owner(mut self) {
        let owner = read_keypair_file(self.startup_config.keypair).unwrap();
        self.owner = owner;
        let rc_owner = Rc::new(owner);
        self.rc_owner = rc_owner;
        return
    }

    pub fn init_page_config(mut self) {
        let page_config = Arc::new(ShardedDb::new(Mutex::new(HashMap::new())));
        self.page_config = page_config;
        return
    }

    pub fn get_pool_dirs(mut self) {
        let pool_dirs = get_pool_directories();
        self.pool_dirs = pool_dirs;
        return
    }

    pub fn init_graph(mut self) {
        let graph = build_graph(self.pool_dirs,
                self.connection,
                self.cluster,
                &self.page_config,
                self.rc_owner);
        self.graph = graph;
        return
    }
}

impl Default for RuntimeConfig {
    // Default state of runtime config assumes no config file has been passed
    // and sets up the runtime config with default values sourcing from
    // StartConfig::Default()
    fn default() -> RuntimeConfig {
        let rc: RuntimeConfig;
        let default_startup = StartupConfig::default();
        rc.setup(default_startup);
        return rc
    }
}
