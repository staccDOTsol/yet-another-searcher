use std::env;
use std::fs;
use toml;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct StartupConfig {
    pub rpc_url: String,
    pub cluster: String,
    pub keypair: String,
    pub flashloan: bool,
}

pub fn initialize_startup(filename: &str) -> StartupConfig {
    if filename == "" {
        // set to default
        let config = StartupConfig{ ..Default::default() };
        return config
    }

    let conf: String = fs::read_to_string(filename).unwrap();
    let conf_slice: &str = &conf[..];  // take a full slice of the string
    let config: StartupConfig = toml::from_str(conf_slice).unwrap();
    // TODO: handle error of not receiving a field
    return config
}

impl Default for StartupConfig {
    fn default() -> StartupConfig {
        let homedir = env::var("HOME").unwrap();
        return StartupConfig {
            rpc_url: "api.mainnet.solana.com".to_string(),
            cluster: "mainnet".to_string(),
            keypair: homedir + "/.config/solana/id.json",
            flashloan: false,
        }
   }
}