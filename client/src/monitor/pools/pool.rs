use anchor_client::solana_sdk::pubkey::Pubkey;
use async_trait::async_trait;
use solana_sdk::account::Account;
use solana_sdk::instruction::Instruction;
use solana_sdk::signature::Keypair;

use crate::monitor::pools::*;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use anchor_client::Cluster;

#[derive(Debug, Clone)]
pub struct PoolDir {
    pub pool_type: PoolType,
    pub dir_path: String,
}

#[derive(Debug, Clone)]
pub enum PoolType {
    OrcaPoolType,
    MercurialPoolType,
    SaberPoolType,
    AldrinPoolType,
    SerumPoolType,
    RaydiumPoolType,
}

pub fn pool_factory(pool_type: &PoolType, json_str: &String) -> Box<dyn PoolOperations> {
    match pool_type {
        PoolType::RaydiumPoolType => {
            let pool: RaydiumPool = serde_json::from_str(json_str).unwrap();
            Box::new(pool)
        }
        PoolType::OrcaPoolType => {
            let pool: OrcaPool = serde_json::from_str(json_str).unwrap();
            Box::new(pool)
        }
        PoolType::MercurialPoolType => {
            let pool: MercurialPool = serde_json::from_str(json_str).unwrap();
            Box::new(pool)
        }
        PoolType::SaberPoolType => {
            let pool: SaberPool = serde_json::from_str(json_str).unwrap();
            Box::new(pool)
        }
        PoolType::AldrinPoolType => {
            let pool: AldrinPool = serde_json::from_str(json_str).unwrap();
            Box::new(pool)
        }
        PoolType::SerumPoolType => {
            let pool: SerumPool = serde_json::from_str(json_str).unwrap();
            Box::new(pool)
        }
    }
}

type ShardedDb = Arc<Mutex<HashMap<String, Account>>>;
#[async_trait]

pub trait PoolOperations: Debug + Send {
    fn clone_box(&self) -> Box<dyn PoolOperations>;

    fn get_pool_type(&self) -> PoolType;
    fn get_name(&self) -> String;
    fn get_own_addr(&self) -> Pubkey;
    fn get_update_accounts(&self) -> Vec<Pubkey>;
    fn set_update_accounts(&mut self, accounts: Vec<Option<Account>>, cluster: Cluster);
    fn set_update_accounts2(&mut self, pubkey: Pubkey, data: &[u8], cluster: Cluster);

    fn mint_2_addr(&self, mint: &Pubkey) -> Pubkey;
    fn get_mints(&self) -> Vec<Pubkey>;
    fn mint_2_scale(&self, mint: &Pubkey) -> u64;

    fn get_quote_with_amounts_scaled(
        &self,
        amount_in: u128,
        mint_in: &Pubkey,
        mint_out: &Pubkey,
    ) -> u128;
   async fn swap_ix(
        &self,
        mint_in: &Pubkey,
        mint_out: &Pubkey,
        start_bal: u128
    ) -> (bool, Vec<Instruction>);

    fn can_trade(&self, mint_in: &Pubkey, mint_out: &Pubkey) -> bool; // used for tests
}
impl Clone for Box<dyn PoolOperations> {
    fn clone(&self) -> Box<dyn PoolOperations> {
        self.clone_box()
    }
}