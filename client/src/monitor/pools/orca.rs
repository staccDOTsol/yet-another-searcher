use crate::monitor::pools::{PoolOperations, PoolType};
use crate::serialize::pool::JSONFeeStructure;
use crate::serialize::token::{ Token, WrappedPubkey, unpack_token_account};


use anchor_client::{Cluster, Program};

use async_trait::async_trait;
use serde;
use solana_client::rpc_client::RpcClient;


use solana_sdk::signature::Keypair;


use std::sync::{Arc, Mutex};

type ShardedDb = Arc<Mutex<HashMap<String, Account>>>;
use std::collections::HashMap;
use std::fmt::Debug;

use std::str::FromStr;



use anchor_client::solana_sdk::pubkey::Pubkey;
use serde::{Deserialize, Serialize};
use solana_sdk::account::Account;

use solana_sdk::instruction::Instruction;

use tmp::accounts as tmp_accounts;
use tmp::instruction as tmp_ix;

use crate::constants::*;
use crate::monitor::pool_utils::base::CurveType;
use crate::monitor::pool_utils::{fees::Fees, orca::get_pool_quote_with_amounts};
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
#[async_trait]


impl PoolOperations for OrcaPool {
    fn clone_box(&self) -> Box<dyn PoolOperations> {
        Box::new(self.clone())
    }
    fn get_pool_type(&self) -> PoolType {
        PoolType::OrcaPoolType
    }
    
fn swap_ix(
        &self,
        mint_in: Pubkey,
        mint_out: Pubkey,
        _start_bal: u128,
        owner: Pubkey,
        program: &Program<Arc<Keypair>>
    ) -> (bool, Vec<Instruction>) {
        let swap_state = Pubkey::from_str("8cjtn4GEw6eVhZ9r1YatfiU65aDEBf1Fof5sTuuH6yVM").unwrap();
        let user_src = derive_token_address(&owner, &mint_in);
        let user_dst = derive_token_address(&owner, &mint_out);
        let user_src_acc = program.rpc().get_account(&user_src);
        let user_dst_acc = program.rpc().get_account(&user_dst);
       
        let (authority_pda, _) =
            Pubkey::find_program_address(&[&self.address.to_bytes()], &ORCA_PROGRAM_ID);

        let pool_src = self.mint_2_addr(&mint_in);
        let pool_dst = self.mint_2_addr(&mint_out);
        if pool_src == Pubkey::new_from_array([0u8; 32])
            || pool_dst == Pubkey::new_from_array([0u8; 32])
        {
            return (false, vec![]);
        }

let _token_swap = self.address.0;
let _pool_mint= self.pool_token_mint.0;
let _fee_account = self.fee_account.0;
let mut swap_ix = program
.request()
.accounts(tmp_accounts::OrcaSwap {
    token_swap: self.address.0,
    authority: authority_pda,
    user_transfer_authority: owner,
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
if user_dst_acc.is_err() {
    // create ata
    let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account(
        &owner,
        &owner,
        &mint_out,
        &spl_token::ID

    );
    swap_ix.insert(0, create_ata_ix);
}



        (false, swap_ix)
    }

    async fn get_quote_with_amounts_scaled_new(
        & self,
        scaled_amount_in: u128,
        mint_in: &Pubkey,
        _mint_out: &Pubkey,
        amt1: u128, 
        amt2: u128
    ) -> u128 {
        let mut pool_src_amount = amt1;
        let mut pool_dst_amount = amt2;
        let idx0 = self.token_ids[0].clone();
        let idx1 = self.token_ids[1].clone();
        if mint_in.to_string() == idx0 {
            pool_src_amount = amt1;
            pool_dst_amount = amt2;
        } else if mint_in.to_string() == idx1 {
            pool_src_amount = amt2;
            pool_dst_amount = amt1;
        }

            let _pool_amounts = [pool_src_amount, pool_dst_amount];
            let _percision_multipliers = [1, 1];


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
        let amt = get_pool_quote_with_amounts(
            scaled_amount_in,
            ctype,
            100,
            &fees,
            pool_src_amount,
            pool_dst_amount,
            None,
        )
        ;
        if amt.is_err() {
            return 0;
        }
        let amt = amt.unwrap();
        if amt > 0 {
            amt - 1
        } else {
            amt
        }
    }
    fn get_quote_with_amounts_scaled(
        &mut self,
        scaled_amount_in: u128,
        mint_in: &Pubkey,
        mint_out: &Pubkey,
        _program: &Arc<RpcClient >
    ) -> u128 {
        let pool_src_amount = self.pool_amounts.get(&mint_in.to_string());
        let pool_dst_amount = self.pool_amounts.get(&mint_out.to_string());

        if pool_src_amount.is_none() || pool_dst_amount.is_none() {
            return 0;
        }
        let pool_src_amount = *pool_src_amount.unwrap();
        let pool_dst_amount = *pool_dst_amount.unwrap();



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
            return 0;
        };

        // get quote -- works for either constant product or stable swap
        let amt = get_pool_quote_with_amounts(
            scaled_amount_in,
            ctype,
            100,
            &fees,
            pool_src_amount,
            pool_dst_amount,
            None,
        )
        ;
        if amt.is_err() {
            return 0;
        }
        let amt = amt.unwrap();
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

    fn can_trade(&mut self, _mint_in: &Pubkey, _mint_out: &Pubkey) -> bool {
        for amount in self.pool_amounts.values() {
            if *amount == 0 {
                return false;
            }
        }
        true
    }

    fn set_update_accounts(&mut self, accounts: Vec<Option<Account>>, _cluster: Cluster) -> bool {
        let ids: Vec<String> = self
            .get_mints()
            .iter()
            .map(|mint| mint.to_string())
            .collect();
        let id0 = &ids[0];
        let id1 = &ids[1];

        if accounts.len() < 2 {
            return false
        }

        let acc_data0 = &accounts[0].as_ref();
        let acc_data1 = &accounts[1].as_ref();
        if acc_data0.is_none() || acc_data1.is_none() {
            return false  
        }
        let acc_data0 = &acc_data0.unwrap().data;
        let acc_data1 = &acc_data1.unwrap().data;

        let amount0 = unpack_token_account(acc_data0).amount as u128;
        let amount1 = unpack_token_account(acc_data1).amount as u128;

        self.pool_amounts.insert(id0.clone(), amount0);
        self.pool_amounts.insert(id1.clone(), amount1);
        true
    }

    fn set_update_accounts2(&mut self, _pubkey: Pubkey, data: &[u8], _cluster: Cluster)  {
        let acc_data0 = data;
        let amount0 = unpack_token_account(acc_data0);
        
        
        let _mint = amount0.mint;
        let _mint = amount0.mint;
        let id0 = &self.token_ids[0];
        let id1 = &self.token_ids[1];
        if _mint.to_string() == *id0 {
            self.pool_amounts
                .entry(id0.clone())
                .and_modify(|e| *e = amount0.amount as u128)
                .or_insert(amount0.amount as u128);
        } else if _mint.to_string() == *id1 {
            self.pool_amounts
                .entry(id1.clone())
                .and_modify(|e| *e = amount0.amount as u128)
                .or_insert(amount0.amount as u128);
        }
    }

    fn get_own_addr(&self) -> Pubkey {
        self.address.0
    }
    fn get_name(&self) -> String {
        "Orca".to_string()
    }

    fn mint_2_addr(&self, mint: &Pubkey) -> Pubkey {
        let token = self.tokens.get(&mint.to_string());
        if token.is_none() {
            return Pubkey::new_from_array([0u8; 32]);
        }
        let token = token.unwrap();

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
