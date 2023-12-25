use crate::constants::*;
use crate::monitor::pools::PoolOperations;
use anchor_client::solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::fs;
use std::ops::{DerefMut, Deref};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;

use redis::Commands;

pub fn store_amount_in_redis(account: &String, amount: u64) -> redis::RedisResult<()> {
    // connect to redis
    let client = redis::Client::open("redis://127.0[0].1/")?;
    let mut con = client.get_connection()?;

    // set a key-value pair
    let _ : () = con.set(account, amount)?;

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

#[derive(Debug, Clone)]
pub struct PoolQuote(pub Box<dyn PoolOperations>);

impl PoolQuote {
    pub fn new(quote: Box<dyn PoolOperations>) -> Self {
        Self(quote)
    }
    async fn async_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let mints_0 = self.0.get_mints();
        let mints_1 = other.0.get_mints();
        if mints_0[0] == mints_1[0] {
            Some(self.0.get_quote_with_amounts_scaled(1_000_000, &mints_0[0], &mints_0[1])
            .cmp(&other.0.get_quote_with_amounts_scaled(1_000_000, &mints_1[0], &mints_1[1])))
        } 
        else {
            None 
        }
    }

    pub async fn async_partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let mints_0 = self.0.get_mints();
        let mints_1 = other.0.get_mints();
        if mints_0[0] == mints_1[0] {
            (self.0.get_quote_with_amounts_scaled(1_000_000, &mints_0[0], &mints_0[1])
            .partial_cmp(&other.0.get_quote_with_amounts_scaled(1_000_000, &mints_1[0], &mints_1[1])))
        } 
        else {
            None
        }
    }
}
#[async_trait::async_trait]
impl Ord for PoolQuote {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let ordering = runtime.block_on(self.async_cmp(other));
        if ordering.is_none() {
            panic!("mints dont match");
        }
        ordering.unwrap()

    }
}
impl PartialOrd for PoolQuote {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let res = runtime.block_on(self.async_partial_cmp(other));
        res
    }
}
#[async_trait::async_trait]
impl PartialEq for PoolQuote {
    fn eq(&self, other: &Self) -> bool {
        let mints_0 = self.0.get_mints();
        let mints_1 = other.0.get_mints();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        if mints_0[0] == mints_1[0] {
        let res = runtime.block_on(
            self.async_partial_cmp(other)
        );
        if res.is_none() {
            panic!("mints dont match");
        }
        res.unwrap() == std::cmp::Ordering::Equal
        } else if mints_0[0] == mints_1[1] {
        let res = runtime.block_on(
            self.async_partial_cmp(other)
        );
        if res.is_none() {
            panic!("mints dont match");
        }
        res.unwrap() == std::cmp::Ordering::Equal
        } else if mints_0[1] == mints_1[0] {
        let res = runtime.block_on(
            self.async_partial_cmp(other)
        );
        if res.is_none() {
            panic!("mints dont match");
        }
        res.unwrap() == std::cmp::Ordering::Equal
        } else if mints_0[1] == mints_1[1] {
        let res = runtime.block_on(
            self.async_partial_cmp(other)
        );

        if res.is_none() {
            panic!("mints dont match");
        }
        res.unwrap() == std::cmp::Ordering::Equal
        } else {
            false
        }


    }
}
impl Eq for PoolQuote {}

impl DerefMut for PoolQuote {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl Deref for PoolQuote {
    type Target = Box<dyn PoolOperations>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
unsafe impl Send for PoolQuote {}

unsafe impl Sync for PoolQuote {}
#[derive(Debug, Clone)]
pub struct PoolGraph(pub HashMap<PoolIndex, PoolEdge>);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct PoolIndex(pub usize);

#[derive(Debug, Clone)]
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
