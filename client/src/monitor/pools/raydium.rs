use crate::monitor::pool_utils::constant_price::ConstantPriceCurve;
use crate::monitor::pool_utils::{fees::Fees, orca::get_pool_quote_with_amounts};
use crate::monitor::pool_utils::serum::FeeTier;
use crate::monitor::pools::{PoolOperations, PoolType};
use crate::serialize::pool::JSONFeeStructure;
use crate::serialize::token::{ Token, WrappedPubkey};
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::signature::read_keypair_file;
use anchor_client::{Client, Cluster};
use async_trait::async_trait;
use openbook_dex::critbit::SlabView;
use openbook_dex::matching::OrderBookState;
use openbook_dex::state::{Market, AccountFlag};
use serde;
use solana_program::account_info::AccountInfo;
use solana_program::stake_history::Epoch;
use solana_sdk::program_pack::Pack;
use solana_sdk::signer::Signer;
use std::sync::{Arc, Mutex};
use std::ops::DerefMut;

use raydium_contract_instructions::{
    amm_instruction::{ID as ammProgramID, swap_base_in as amm_swap},
    stable_instruction::{ID as stableProgramID, swap_base_in as stable_swap},
};


type ShardedDb = Arc<Mutex<HashMap<String, Account>>>;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use std::str::FromStr;

use solana_sdk::signature::Keypair;

use anchor_client::solana_sdk::pubkey::Pubkey;
use serde::{Deserialize, Serialize};
use solana_sdk::account::Account;

use solana_sdk::instruction::Instruction;

use tmp::accounts as tmp_accounts;
use tmp::instruction as tmp_ix;

use crate::constants::*;
use crate::monitor::pool_utils::base::CurveType;
use crate::utils::{derive_token_address, str2pubkey};

struct Iteration {
    amount_in: u64,
    amount_out: u64,
}

fn account_info<'a>(pk: &'a Pubkey, account: &'a mut Account) -> AccountInfo<'a> {
    AccountInfo::new(
        pk,
        false,
        true,
        &mut account.lamports,
        &mut account.data,
        &account.owner,
        false,
        Epoch::default(),
    )
}
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RaydiumPool {
    
    pub id: WrappedPubkey,
    pub base_mint: WrappedPubkey,
    pub quote_mint: WrappedPubkey,
    pub lp_mint: WrappedPubkey,
    pub base_decimals: u8,
    pub quote_decimals: u8,
    pub lp_decimals: u8,
    pub version: u8,
    pub program_id: WrappedPubkey,
    pub authority: WrappedPubkey,
    pub open_orders: WrappedPubkey,
    pub target_orders: WrappedPubkey,
    pub base_vault: WrappedPubkey,
    pub quote_vault: WrappedPubkey,
    pub withdraw_queue: WrappedPubkey,
    pub lp_vault: WrappedPubkey,
    pub market_version: u8,
    pub market_program_id: WrappedPubkey,
    pub market_id: WrappedPubkey,
    pub market_authority: WrappedPubkey,
    pub market_base_vault: WrappedPubkey,
    pub market_quote_vault: WrappedPubkey,
    pub market_bids: WrappedPubkey,
    pub market_asks: WrappedPubkey,
    pub market_event_queue: WrappedPubkey,
    pub model_data_account: Option<WrappedPubkey>,
    #[serde(skip)]
    pub accounts: Vec<Option<Account>>,
    #[serde(skip)]
    pub pool_amounts: HashMap<String, u128>,
}


// bid: quote -> base
fn bid_iteration(iteration: &mut Iteration, fee_tier: &FeeTier, ob: &mut OrderBookState) -> bool {
    let mut quote_lot_size = ob.market_state.pc_lot_size;
    let base_lot_size = ob.market_state.coin_lot_size;
    if quote_lot_size == 0 {
        quote_lot_size = 1;
    }
    let start_amount_in = iteration.amount_in;
    let max_pc_qty = fee_tier.remove_taker_fee(iteration.amount_in) / quote_lot_size;
    let mut pc_qty_remaining = max_pc_qty;

    let done = loop {
        let flag = match ob.asks.find_min() {
            // min = best ask
            Some(_) => false,
            None => true,
        };
        if flag {
            break true;
        }
        let best_ask = ob.asks.find_min().unwrap();
        let best_offer_ref = ob.asks.get_mut(best_ask).unwrap().as_leaf_mut().unwrap();

        let trade_price = best_offer_ref.price();
        let offer_size = best_offer_ref.quantity();
        let trade_qty = offer_size.min(pc_qty_remaining / best_offer_ref.price().get());

        if trade_qty == 0 || offer_size == 0 {
            // fin
            break true;
        }

        pc_qty_remaining -= trade_qty * trade_price.get();
        iteration.amount_out += trade_qty * base_lot_size;

        best_offer_ref.set_quantity(best_offer_ref.quantity() - trade_qty);

        if best_offer_ref.quantity() == 0 {
            let best_offer_id = best_offer_ref.order_id();
            ob.asks.remove_by_key(best_offer_id).unwrap();
        }
        break false;
    };

    let native_accum_fill_price = (max_pc_qty - pc_qty_remaining) * quote_lot_size;
    let native_taker_fee = fee_tier.taker_fee(native_accum_fill_price);
    let native_pc_qty_remaining = start_amount_in - native_accum_fill_price - native_taker_fee;
    iteration.amount_in = native_pc_qty_remaining;

    done
}

// ask: base -> quote
fn ask_iteration(iteration: &mut Iteration, fee_tier: &FeeTier, ob: &mut OrderBookState) -> bool {
    let pc_lot_size = ob.market_state.pc_lot_size;
    let mut coin_lot_size = ob.market_state.coin_lot_size;
    if coin_lot_size == 0 {
        coin_lot_size = 1;
    }
    let max_qty = iteration.amount_in;
    let mut unfilled_qty = max_qty / coin_lot_size;
    let mut accum_fill_price = 0;

    let done = loop {
        let best_bid = match ob.bids.find_max() {
            // min = best ask
            Some(best_bid) => best_bid,
            None => {
                break true; // no more bids
            }
        };
        let best_bid_ref = ob.bids.get_mut(best_bid).unwrap().as_leaf_mut().unwrap();

        let trade_price = best_bid_ref.price();
        let bid_size = best_bid_ref.quantity();
        let trade_qty = bid_size.min(unfilled_qty);

        if trade_qty == 0 || bid_size == 0 {
            // fin
            break true;
        }

        best_bid_ref.set_quantity(best_bid_ref.quantity() - trade_qty);
        unfilled_qty -= trade_qty;
        accum_fill_price += trade_qty * trade_price.get();

        if best_bid_ref.quantity() == 0 {
            let best_offer_id = best_bid_ref.order_id();
            ob.bids.remove_by_key(best_offer_id).unwrap();
        }
        break false;
    };
    // fees applied after
    let native_taker_pc_qty = accum_fill_price * pc_lot_size;
    let native_taker_fee = fee_tier.taker_fee(native_taker_pc_qty);
    let net_taker_pc_qty = native_taker_pc_qty - native_taker_fee;

    iteration.amount_out += net_taker_pc_qty;
    iteration.amount_in = unfilled_qty * coin_lot_size;

    done
}

#[async_trait]


impl PoolOperations for RaydiumPool {
    fn clone_box(&self) -> Box<dyn PoolOperations> {
        Box::new(self.clone())
    }
    fn get_pool_type(&self) -> PoolType {
        PoolType::RaydiumPoolType
    }
    
async    fn swap_ix(
        &self,
        mint_in: &Pubkey,
        mint_out: &Pubkey,
        _start_bal: u128,
    ) -> (bool, Vec<Instruction>) {
        let swap_state = Pubkey::from_str("8cjtn4GEw6eVhZ9r1YatfiU65aDEBf1Fof5sTuuH6yVM").unwrap();
        let owner3 = Arc::new(read_keypair_file("/Users/stevengavacs/.config/solana/id.json".clone()).unwrap());

        let owner = owner3.try_pubkey().unwrap();
        let user_src = derive_token_address(&owner, mint_in);
        let user_dst = derive_token_address(&owner, mint_out);

        let mut swap_ix = amm_swap(
            &ammProgramID,
            &self.id,
            &self.authority,
            &self.open_orders,
            &self.target_orders,
            &self.base_vault,
            &self.quote_vault,
            &self.market_program_id,
            &self.market_id,
            &self.market_bids,
            &self.market_asks,
            &self.market_event_queue,
            &self.base_vault,
            &self.quote_vault,
            &self.market_authority,
            &user_src,
            &user_dst,
            &owner, 
            _start_bal as u64,
            0 as u64
        );
    if swap_ix.is_err() {
        swap_ix = stable_swap(
            &stableProgramID,
            &self.id,
            &self.authority,
            &self.open_orders,
            &self.base_vault,
            &self.quote_vault,
            &self.model_data_account.as_ref().unwrap(),
            &self.market_program_id,
            &self.market_id,
            &self.market_bids,
            &self.market_asks,
            &self.market_event_queue,
            &self.base_vault,
            &self.quote_vault,
            &self.market_authority,
            &user_src,
            &user_dst,
            &owner, 
            _start_bal as u64,
            0 as u64
        );
    }        

        (false, vec![swap_ix.unwrap()])
    }

    fn get_quote_with_amounts_scaled(
        & self,
        scaled_amount_in: u128,
        mint_in: &Pubkey,
        mint_out: &Pubkey,
    ) -> u128 {
        
        let pool_src_amount = self.pool_amounts.get(&mint_in.to_string());
        let pool_dst_amount = self.pool_amounts.get(&mint_out.to_string());
        if pool_src_amount.is_none() || pool_dst_amount.is_none() {
            return 0;
        }
        let pool_src_amount = pool_src_amount.unwrap();
        let pool_dst_amount = pool_dst_amount.unwrap();
        
        // compute fees
        let fees = Fees {
            trade_fee_numerator: 0,
            trade_fee_denominator: 1,
            owner_trade_fee_numerator: 0,
            owner_trade_fee_denominator: 1,
            owner_withdraw_fee_numerator: 0,
            owner_withdraw_fee_denominator: 0,
            host_fee_numerator: 0,
            host_fee_denominator: 0,
        };

        let ctype = if self.version != 1 {
            CurveType::Stable
        } else {
            CurveType::ConstantProduct
        };

        // get quote -- works for either constant product or stable swap

        let amt = get_pool_quote_with_amounts(
            scaled_amount_in,
            ctype,
            170, // from sdk
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



    fn can_trade(&self, mint_in: &Pubkey, _mint_out: &Pubkey) -> bool {
        for amount in self.pool_amounts.values() {
            if *amount == 0 {
                return false;
            }
        }
        true
        
    }fn get_name(&self) -> String {
        "Raydium".to_string()
    }

    fn get_own_addr(&self) -> Pubkey {
        self.id.0
    }
    fn get_update_accounts(&self) -> Vec<Pubkey> {
        vec![self.base_vault.0, self.quote_vault.0]
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

        let amount0 = spl_token::state::Account::unpack(acc_data0).unwrap().amount as u128;
        let amount1 = spl_token::state::Account::unpack(acc_data1).unwrap().amount as u128;

        self.pool_amounts.insert(id0.clone(), amount0);
        self.pool_amounts.insert(id1.clone(), amount1);
    }

    fn set_update_accounts2(&mut self, _pubkey: Pubkey, data: &[u8], _cluster: Cluster) {
        let acc_data0 = data;
        let amount0 = spl_token::state::Account::unpack(acc_data0).unwrap();
        let _mint = amount0.mint;
        let _mint = amount0.mint;
        let id0 = self.base_mint.0.to_string();
        let id1 = self.quote_mint.0.to_string();
        if _mint.to_string() == id0.to_string() {
            self.pool_amounts
                .entry(id0.clone())
                .and_modify(|e| *e = amount0.amount as u128)
                .or_insert(amount0.amount as u128);
        } else if _mint.to_string() == id1.to_string() {
            self.pool_amounts
                .entry(id1.clone())
                .and_modify(|e| *e = amount0.amount as u128)
                .or_insert(amount0.amount as u128);
        }
    }


    fn mint_2_addr(&self, mint: &Pubkey) -> Pubkey {
        if *mint == self.base_mint.0 {
            self.quote_mint.0
        } else if *mint == self.quote_mint.0 {
            self.base_mint.0
        } else {
            panic!("invalid mint bro")
        }
    }

    fn get_mints(&self) -> Vec<Pubkey> {
        let mut mints = vec![self.base_mint.0, self.quote_mint.0];
        mints.sort();
        mints
    }

    fn mint_2_scale(&self, mint: &Pubkey) -> u64 {
        if *mint == self.base_mint.0 {
            self.base_decimals as u64
        } else if *mint == self.quote_mint.0 {
            self.quote_decimals as u64
        } else {
            panic!("invalid mint bro")
        }
    }

}
