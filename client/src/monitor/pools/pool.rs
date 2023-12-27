use anchor_client::solana_sdk::pubkey::Pubkey;
use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::instruction::Instruction;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;


use crate::monitor::pools::*;
use crate::utils::PoolQuote;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use anchor_client::{Cluster, Program};

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
pub trait PoolOperations: Send {
    fn clone_box(&self) -> Box<dyn PoolOperations>;
    
    fn get_pool_type(&self) -> PoolType;
    fn get_name(&self) -> String;
    fn get_own_addr(&self) -> Pubkey;
    fn get_update_accounts(&self) -> Vec<Pubkey>;
    fn set_update_accounts(&mut self, accounts: Vec<Option<&Account>>, cluster: Cluster);
    fn set_update_accounts2(&mut self, pubkey: Pubkey, data: &[u8], cluster: Cluster);

    fn mint_2_addr(&self, mint: &Pubkey) -> Pubkey;
    fn get_mints(&self) -> Vec<Pubkey>;
    fn mint_2_scale(&self, mint: &Pubkey) -> u64;

    fn get_quote_with_amounts_scaled(
        &mut self,
        amount_in: u128,
        mint_in: &Pubkey,
        mint_out: &Pubkey,
        program: &Arc<RpcClient >
    ) -> u128;
    async fn get_quote_with_amounts_scaled_new(
        &self,
        amount_in: u128,
        mint_in: &Pubkey,
        mint_out: &Pubkey,
        amount: u128,
        amount_out: u128,
    ) -> u128;
    fn swap_ix(
        &self,
        mint_in: Pubkey,
        mint_out: Pubkey,
        start_bal: u128,
        owner: Pubkey,
        program: &Program<Arc<Keypair>>
    ) -> (bool, Vec<Instruction>);

    fn can_trade(&mut self, mint_in: &Pubkey, mint_out: &Pubkey) -> bool; // used for tests
}
impl Clone for Box<dyn PoolOperations> {
    fn clone(&self) -> Box<dyn PoolOperations> {
        self.clone_box()
    }
}
#[async_trait::async_trait]
impl PoolOperations for PoolQuote {
    fn clone_box(&self) -> Box<dyn PoolOperations> {
        self.0.borrow().clone_box()
    }
    fn get_pool_type(&self) -> PoolType {
        self.0.borrow().get_pool_type()
    }
    fn get_name(&self) -> String {
        self.0.borrow().get_name()
    }
    fn get_own_addr(&self) -> Pubkey {
        self.0.borrow().get_own_addr()
    }
    fn get_update_accounts(&self) -> Vec<Pubkey> {
        self.0.borrow().get_update_accounts()
    }
    fn set_update_accounts(&mut self, accounts: Vec<Option<&Account>>, cluster: Cluster) {
        self.0.borrow_mut().set_update_accounts(accounts, cluster)
    }
    fn set_update_accounts2(&mut self, pubkey: Pubkey, data: &[u8], cluster: Cluster) {
        self.0.borrow_mut().set_update_accounts2(pubkey, data, cluster)
    }
    fn mint_2_addr(&self, mint: &Pubkey) -> Pubkey {
        self.0.borrow().mint_2_addr(mint)
    }
    fn get_mints(&self) -> Vec<Pubkey> {
        self.0.borrow().get_mints()
    }
    fn mint_2_scale(&self, mint: &Pubkey) -> u64 {
        self.0.borrow().mint_2_scale(mint)
    }
    fn get_quote_with_amounts_scaled(
        &mut self,
        amount_in: u128,
        mint_in: &Pubkey,
        mint_out: &Pubkey,
        program: &Arc<RpcClient >
    ) -> u128 {
        self.0
            .borrow_mut()
            .get_quote_with_amounts_scaled(amount_in, mint_in, mint_out, program)
    }
    async fn get_quote_with_amounts_scaled_new(
        &self,
        amount_in: u128,
        mint_in: &Pubkey,
        mint_out: &Pubkey,
        amount: u128,
        amount_out: u128,
    ) -> u128 {
        0
            
    }
    fn swap_ix(
        &self,
        mint_in: Pubkey,
        mint_out: Pubkey,
        start_bal: u128,
        owner: Pubkey,
        program: &Program<Arc<Keypair>>
    ) -> (bool, Vec<Instruction>) {
        self.0
            .borrow()
            .swap_ix(mint_in, mint_out, start_bal, owner, program)
    }

    fn can_trade(&mut self, mint_in: &Pubkey, mint_out: &Pubkey) -> bool {
        self.0.borrow_mut().can_trade(mint_in, mint_out)
    }
}