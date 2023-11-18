use crate::pool::{PoolOperations, PoolType};
use crate::serialize::pool::JSONFeeStructure;
use crate::serialize::token::{unpack_token_account, Token, WrappedPubkey};
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::signature::read_keypair_file;
use anchor_client::{Client, Cluster};
use serde;
use solana_sdk::program_pack::Pack;
use std::sync::{Arc, Mutex};

type ShardedDb = Arc<Mutex<HashMap<String, Account>>>;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::rc::Rc;
use std::str::FromStr;

use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::signature::Keypair;
use solana_sdk::signature::Signer;
use solana_sdk::transaction::Transaction;

use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::Program;
use serde::{Deserialize, Serialize};
use solana_sdk::account::Account;

use solana_sdk::instruction::Instruction;

use tmp::accounts as tmp_accounts;
use tmp::instruction as tmp_ix;

use crate::constants::*;
use crate::pool_utils::base::CurveType;
use crate::pool_utils::{fees::Fees, orca::get_pool_quote_with_amounts};
use crate::utils::{derive_token_address, str2pubkey};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrcaPool {
    pub address: WrappedPubkey,
    pub nonce: u64,
    pub authority: WrappedPubkey,
    pub pool_token_mint: WrappedPubkey,
    pub pool_token_decimals: u64,
    pub fee_account: WrappedPubkey,
    pub token_ids: Vec<String>,
    pub tokens: HashMap<String, Token>,
    pub fee_structure: JSONFeeStructure,
    pub curve_type: u8,
    #[serde(default)]
    pub amp: u64,
    // to set later
    #[serde(skip)]
    pub pool_amounts: HashMap<String, u128>,
}

impl PoolOperations for OrcaPool {
    fn clone_box(&self) -> Box<dyn PoolOperations> {
        Box::new(self.clone())
    }
    fn get_pool_type(&self) -> PoolType {
        PoolType::OrcaPoolType
    }
    fn swap_ix(
        &self,
        owner: &Pubkey,
        mint_in: &Pubkey,
        mint_out: &Pubkey,
        ookp: &Keypair,
        start_bal: u128,
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
            CommitmentConfig::recent(),
        );
        let program = provider.program(*ARB_PROGRAM_ID).unwrap();
        let (authority_pda, _) =
            Pubkey::find_program_address(&[&self.address.to_bytes()], &ORCA_PROGRAM_ID);

        let pool_src = self.mint_2_addr(mint_in);
        let pool_dst = self.mint_2_addr(mint_out);

        let swap_ix = program
            .request()
            .accounts(tmp_accounts::OrcaSwap {
                token_swap: self.address.0,
                authority: authority_pda,
                user_transfer_authority: *owner,
                user_src,
                pool_src,
                user_dst,
                pool_dst,
                pool_mint: self.pool_token_mint.0,
                fee_account: self.fee_account.0,
                token_program: *TOKEN_PROGRAM_ID,
                token_swap_program: *ORCA_PROGRAM_ID,
                swap_state,
            })
            .args(tmp_ix::OrcaSwap {})
            .instructions()
            .unwrap();

        (false, swap_ix)
    }

    fn get_quote_with_amounts_scaled(
        &mut self,
        scaled_amount_in: u128,
        mint_in: &Pubkey,
        mint_out: &Pubkey,
        page_config: &ShardedDb,
    ) -> u128 {
        if !self.pool_amounts.contains_key(&mint_in.to_string())
            || !self.pool_amounts.contains_key(&mint_out.to_string())
        {
            println!("orca pool amounts not found");
            return 0;
        }
        let pc = page_config.lock().unwrap();
        if pc.contains_key(&self.get_own_addr().to_string()) {
            let acc = pc.get(&self.get_own_addr().to_string()).unwrap();
            let acc_data = &acc.data;
            let amount0 = unpack_token_account(acc_data).amount as u128;
            let id0 = &self.token_ids[0];
            let id1 = &self.token_ids[1];
            if id0.to_string() == mint_in.to_string() {
                self.pool_amounts.insert(id0.clone(), amount0);
            } else {
                self.pool_amounts.insert(id1.clone(), amount0);
            }
        }
        let pool_src_amount = self.pool_amounts.get(&mint_in.to_string()).unwrap();
        let pool_dst_amount = self.pool_amounts.get(&mint_out.to_string()).unwrap();

        // compute fees
        let trader_fee = &self.fee_structure.trader_fee;
        let owner_fee = &self.fee_structure.owner_fee;
        let fees = Fees {
            trade_fee_numerator: trader_fee.numerator,
            trade_fee_denominator: trader_fee.denominator,
            owner_trade_fee_numerator: owner_fee.numerator,
            owner_trade_fee_denominator: owner_fee.denominator,
            owner_withdraw_fee_numerator: 0,
            owner_withdraw_fee_denominator: 0,
            host_fee_numerator: 0,
            host_fee_denominator: 0,
        };
        let ctype = if self.curve_type == 0 {
            CurveType::ConstantProduct
        } else if self.curve_type == 2 {
            CurveType::Stable
        } else {
            panic!("invalid self curve type: {:?}", self.curve_type);
        };

        // get quote -- works for either constant product or stable swap
        println!("{} {} ", pool_src_amount, pool_dst_amount);
        let mut amt = get_pool_quote_with_amounts(
            scaled_amount_in,
            ctype,
            self.amp,
            &fees,
            *pool_src_amount,
            *pool_dst_amount,
            None,
        )
        .unwrap();
        if amt > 0 {
            amt - 1
        } else {
            amt
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

    fn can_trade(&self, _mint_in: &Pubkey, _mint_out: &Pubkey) -> bool {
        for amount in self.pool_amounts.values() {
            if *amount == 0 {
                return false;
            }
        }
        true
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

    fn set_update_accounts2(&mut self, pubkey: Pubkey, data: &[u8], _cluster: Cluster) {
        let mut acc_data0 = data;
        let amount0 = spl_token::state::Account::unpack(acc_data0).unwrap();
        let _mint = amount0.mint;
        let _mint = amount0.mint;
        let id0 = &self.token_ids[0];
        let id1 = &self.token_ids[1];
        if _mint.to_string() == id0.to_string() {
            self.pool_amounts
                .insert(id0.clone(), amount0.amount as u128);
        } else if _mint.to_string() == id1.to_string() {
            self.pool_amounts
                .insert(id1.clone(), amount0.amount as u128);
        }
    }

    fn get_own_addr(&self) -> Pubkey {
        self.address.0
    }
    fn get_name(&self) -> String {
        "Orca".to_string()
    }

    fn mint_2_addr(&self, mint: &Pubkey) -> Pubkey {
        let token = self.tokens.get(&mint.to_string()).unwrap();

        token.addr.0
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