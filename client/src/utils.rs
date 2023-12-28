use crate::constants::*;
use crate::monitor::pools::{PoolOperations};
use anchor_client::solana_sdk::pubkey::Pubkey;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;


use std::str::FromStr;
use std::sync::Arc;

use redis::Commands;

pub fn store_amount_in_redis(account: &String, amount: u64) -> redis::RedisResult<()> {
    // connect to redis
    let client = redis::Client::open("redis://127.0[0].1/")?;
    let mut con = client.get_connection()?;

    // set a key-value pair
    con.set(account, amount)?;

    Ok(())
}    
pub fn get_amount_from_redis(account: &String) -> redis::RedisResult<u128> {
    // connect to redis
    let client = redis::Client::open("redis://127.0[0].1/")?;
    let mut con = client.get_connection()?;

    // get the value of a key
    let amount = con.get(account);

    if amount.is_err() {
        return Ok(0);
    }
    let amount: i32 = amount.unwrap();
    

    Ok(amount as u128)
}

pub fn read_json_dir(dir: &String) -> Vec<String> {
    let _paths = fs::read_dir(dir).unwrap();
    let mut paths = Vec::new();
    for path in _paths {
        let p = path.unwrap().path();
        let path_str = p;
        match path_str.extension() {
            Some(ex) => {
                if ex == "json" {
                    let path = path_str.to_str().unwrap().to_string();
                    paths.push(path);
                }
            }
            None => {}
        }
    }
    paths
}

pub fn str2pubkey(s: &str) -> Pubkey {
    Pubkey::from_str(s).unwrap()
}

pub fn derive_token_address(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    let (pda, _) = Pubkey::find_program_address(
        &[
            &owner.to_bytes(),
            &TOKEN_PROGRAM_ID.to_bytes(),
            &mint.to_bytes(),
        ],
        &ASSOCIATED_TOKEN_PROGRAM_ID,
    );
    pda
}

#[derive(Clone)]
pub struct PoolQuote(pub Arc<RefCell<Box<dyn PoolOperations>>>);

unsafe impl Send for PoolQuote {}

unsafe impl Sync for PoolQuote {}

impl PoolQuote {
    pub fn new(quote: Arc<RefCell<Box<dyn PoolOperations>>>) -> Self {
        Self(quote)
    }
}

#[derive(Clone)]
pub struct PoolGraph(pub HashMap<PoolIndex, PoolEdge>);
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct PoolIndex(pub usize);

#[derive(Clone)]
pub struct PoolEdge(pub HashMap<PoolIndex, Vec<PoolQuote>>);

impl Default for PoolGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl PoolGraph {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}