use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::signature::read_keypair_file;
use solana_sdk::program_pack::Pack;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

type ShardedDb = Arc<Mutex<HashMap<String, Account>>>;
use anchor_client::{Client, Cluster};

use std::str::FromStr;

use crate::monitor::pools::{PoolOperations, PoolType};
use crate::serialize::token::{unpack_token_account, Token, WrappedPubkey};
use serde;
use serde::{Deserialize, Serialize};
use solana_sdk::signature::Keypair;

use anchor_client::solana_sdk::pubkey::Pubkey;

use solana_sdk::account::Account;
use solana_sdk::instruction::Instruction;

use tmp::accounts as tmp_accounts;
use tmp::instruction as tmp_ix;

use crate::constants::*;
use crate::monitor::pool_utils::stable::Stable;
use crate::utils::{derive_token_address, str2pubkey};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SaberPool {
    pub pool_account: WrappedPubkey,
    pub authority: WrappedPubkey,
    pub pool_token_mint: WrappedPubkey,
    pub token_ids: Vec<String>,
    pub tokens: HashMap<String, Token>,
    pub target_amp: u64,
    pub fee_numerator: u64,
    pub fee_denominator: u64,
    // unique
    pub fee_accounts: HashMap<String, WrappedPubkey>,
    // to set later
    #[serde(skip)]
    pub pool_amounts: HashMap<String, u128>,
}

impl PoolOperations for SaberPool {
    fn clone_box(&self) -> Box<dyn PoolOperations> {
        Box::new(self.clone())
    }
    fn get_pool_type(&self) -> PoolType {
        PoolType::SaberPoolType
    }
    fn swap_ix(
        &self,
        owner: &Pubkey,
        mint_in: &Pubkey,
        mint_out: &Pubkey,
        _ookp: &Keypair,
        _start_bal: u128,
    ) -> (bool, Vec<Instruction>) {
        let swap_state = Pubkey::from_str("8cjtn4GEw6eVhZ9r1YatfiU65aDEBf1Fof5sTuuH6yVM").unwrap();

        let user_src = derive_token_address(owner, mint_in);
        let user_dst = derive_token_address(owner, mint_out);

        let owner_kp_path = "/Users/stevengavacs/.config/solana/id.json";
        // setup anchor things

        let owner2 = read_keypair_file(owner_kp_path.clone()).unwrap();
        let rc_owner = Rc::new(owner2);
        let provider = Client::new_with_options(
            Cluster::Mainnet,
            rc_owner.clone(),
            CommitmentConfig::processed(),
        );
        let program = provider.program(*ARB_PROGRAM_ID).unwrap();
        let pool_src = self.tokens.get(&mint_in.to_string()).unwrap().addr.0;
        let pool_dst = self.tokens.get(&mint_out.to_string()).unwrap().addr.0;
        let fee_acc;
        if self.fee_accounts.contains_key(&mint_in.to_string()) {
            fee_acc = self.fee_accounts.get(&mint_in.to_string()).unwrap().clone();
        } else if self.fee_accounts.contains_key(&mint_out.to_string())  {
            fee_acc = self
                .fee_accounts
                .get(&mint_out.to_string())
                .unwrap()
                .clone();
        } else {
            return (false, vec![]);
        }
        let swap_ix = program
            .request()
            .accounts(tmp_accounts::SaberSwap {
                pool_account: self.pool_account.0,
                authority: self.authority.0,
                user_transfer_authority: *owner,
                user_src,
                user_dst,
                pool_src,
                pool_dst,
                fee_dst: fee_acc.0,
                saber_swap_program: *SABER_PROGRAM_ID,
                swap_state,
                token_program: *TOKEN_PROGRAM_ID,
            })
            .args(tmp_ix::SaberSwap {})
            .instructions()
            .unwrap();
        (false, swap_ix)
    }

    fn get_quote_with_amounts_scaled(
        & self,
        scaled_amount_in: u128,
        mint_in: &Pubkey,
        mint_out: &Pubkey,
    ) -> u128 {
        let calculator = Stable {
            amp: self.target_amp,
            fee_numerator: self.fee_numerator as u128,
            fee_denominator: self.fee_denominator as u128,
        };
        if self.pool_amounts.contains_key(&mint_in.to_string())
            && self.pool_amounts.contains_key(&mint_out.to_string())
        {
            let pool_src_amount = self.pool_amounts.get(&mint_in.to_string()).unwrap();
            let pool_dst_amount = self.pool_amounts.get(&mint_out.to_string()).unwrap();
            let pool_amounts = [*pool_src_amount, *pool_dst_amount];
            let percision_multipliers = [1, 1];

            calculator.get_quote(pool_amounts, percision_multipliers, scaled_amount_in)
        } else {
            0
        }
    }

    fn get_update_accounts(&self) -> Vec<Pubkey> {
        // pool vault amount
        let accounts = self
            .get_mints()
            .iter()
            .map(|mint| self.mint_2_addr(mint))
            .collect();
        accounts
    }

    fn set_update_accounts(&mut self, accounts: Vec<Option<Account>>, _cluster: Cluster) {
        let ids: Vec<String> = self
            .get_mints()
            .iter()
            .map(|mint| mint.to_string())
            .collect();
        let id0 = &ids[0];
        let id1 = &ids[1];

        let acc_data0 = &accounts[0].as_ref().unwrap().data;
        let acc_data1 = &accounts[1].as_ref().unwrap().data;

        let amount0 = unpack_token_account(acc_data0).amount as u128;
        let amount1 = unpack_token_account(acc_data1).amount as u128;

        self.pool_amounts.insert(id0.clone(), amount0);
        self.pool_amounts.insert(id1.clone(), amount1);
    }

    fn set_update_accounts2(&mut self, _pubkey: Pubkey, data: &[u8], _cluster: Cluster) {
        let acc_data0 = data;

        let amount0 = spl_token::state::Account::unpack(acc_data0);
        if amount0.is_err() {
            println!("saber pool amount err: {:?}", amount0);
            return;
        }
        let amount0 = amount0.unwrap();
        let _mint = amount0.mint;
        let id0 = &self.token_ids[0];
        let id1 = &self.token_ids[1];
        if _mint.to_string() == id0.to_string() {
            self.pool_amounts
                .insert(id0.clone(), amount0.amount as u128);
        } else {
            self.pool_amounts
                .insert(id1.clone(), amount0.amount as u128);
        }
    }

    fn can_trade(&self, _mint_in: &Pubkey, _mint_out: &Pubkey) -> bool {
        for amount in self.pool_amounts.values() {
            if *amount == 0 {
                return false;
            }
        }
        true
    }

    fn get_own_addr(&self) -> Pubkey {
        self.pool_account.0
    }
    fn get_name(&self) -> String {
        "Saber".to_string()
    }

    fn mint_2_addr(&self, mint: &Pubkey) -> Pubkey {
        if self.tokens.contains_key(&mint.to_string()) {
            let token = self.tokens.get(&mint.to_string()).unwrap();

            token.addr.0
        } else {
            Pubkey::new_from_array([0; 32])
        }
    }

    fn mint_2_scale(&self, mint: &Pubkey) -> u64 {
        let token = self.tokens.get(&mint.to_string()).unwrap();

        token.scale
    }

    fn get_mints(&self) -> Vec<Pubkey> {
        let mut mints: Vec<Pubkey> = self.token_ids.iter().map(|k| str2pubkey(k)).collect();
        // sort so that its consistent across different pools
        mints.sort();
        mints
    }
}
