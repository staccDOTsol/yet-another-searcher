use crate::constants::*;
use crate::monitor::pools::PoolOperations;
use anchor_client::solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::fs;
use std::ops::{DerefMut, Deref};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;

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
}
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
