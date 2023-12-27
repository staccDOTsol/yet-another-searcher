
use crate::monitor::pool_utils::{fees::Fees, orca::get_pool_quote_with_amounts};
use crate::monitor::pool_utils::serum::FeeTier;
use crate::monitor::pools::{PoolOperations, PoolType};
use crate::serialize::token::{ WrappedPubkey, unpack_token_account};
use crate::monitor::pools::math;
use anchor_client::solana_sdk::signature::read_keypair_file;
use anchor_client::{Cluster, Program};
use async_trait::async_trait;
use num_traits::ToPrimitive;

use serum_dex::critbit::SlabView;
use serum_dex::matching::OrderBookState;
use serum_dex::state::OpenOrders;
use serum_dex::{
    matching::Side,
    state::{EventView, MarketState, ToAlignedBytes},
};
use raydium_amm::check_assert_eq;
use raydium_amm::math::{SwapDirection, U128, Calculator, CheckedCeilDiv};
use raydium_amm::processor::Processor;
use raydium_amm::state::{AmmInfo, Loadable, AmmStatus};
use serde;
use solana_client::rpc_client::RpcClient;
use solana_program::account_info::AccountInfo;
use solana_program::stake_history::Epoch;
use solana_sdk::program_pack::Pack;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use std::sync::{Arc, Mutex};


use raydium_contract_instructions::{
    amm_instruction::{ID as ammProgramID, swap_base_in as amm_swap},
    stable_instruction::{ID as stableProgramID, swap_base_in as stable_swap},
};


type ShardedDb = Arc<Mutex<HashMap<String, Account>>>;
use std::collections::HashMap;
use std::fmt::Debug;

use std::str::FromStr;



use anchor_client::solana_sdk::pubkey::Pubkey;
use serde::{Deserialize, Serialize};
use solana_sdk::account::Account;

use solana_sdk::instruction::Instruction;





use crate::monitor::pool_utils::base::CurveType;
use crate::utils::{derive_token_address, store_amount_in_redis, get_amount_from_redis};

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
#[derive(Deserialize, Serialize, Clone)]
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
    #[serde(skip)]
    pub cache: HashMap<(Pubkey, Pubkey, Pubkey, Pubkey), (Account, Account, Account, Account, AmmInfo, MarketState, Box<serum_dex::state::OpenOrders>, Account)>,

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
        let flag = ob.asks.find_min().is_none();
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
    fn swap_ix(
        &self,
        mint_in: Pubkey,
        mint_out: Pubkey,
        _start_bal: u128,
        pubkey: Pubkey,
        program: &Program<Arc<Keypair>>
    ) -> (bool, Vec<Instruction>) {
        let _swap_state = Pubkey::from_str("8cjtn4GEw6eVhZ9r1YatfiU65aDEBf1Fof5sTuuH6yVM").unwrap();
        let user_src = derive_token_address(&pubkey, &mint_in);
        let user_dst = derive_token_address(&pubkey, &mint_out);

        let user_src_acc = program.rpc().get_account(&user_src);
        let user_dst_acc = program.rpc().get_account(&user_dst);
        let ctype = if self.version == 1 {
            CurveType::Stable
        } else {
            CurveType::ConstantProduct
        };
        let program_id = self.program_id.clone();
        let id = self.id.clone();
        let authority = Pubkey::from_str("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1").unwrap();
        let open_orders = self.open_orders.clone();
        let target_orders = self.target_orders.clone();
        let market_program_id = self.market_program_id.clone();
        let market_id = self.market_id.clone();
        let market_id_acc = program.rpc().get_account(&market_id.0).unwrap();
        let market_bids = self.market_bids.clone();
        let market_asks = self.market_asks.clone();
        let market_event_queue = self.market_event_queue.clone();
        let base_vault = self.base_vault.clone();
        let quote_vault = self.quote_vault.clone();
        let market_authority = self.market_authority.clone();
        let market_base_vault = self.market_base_vault.clone();
        let market_quote_vault = self.market_quote_vault.clone();
        if ctype == CurveType::ConstantProduct {
            let swap_ix =
                amm_swap(
                    &ammProgramID,
                    &id,
                    &authority,
                    &open_orders,
                    &target_orders,
                    &base_vault,
                    &quote_vault,
                    &market_program_id,
                    &market_id,
                    &market_bids,
                    &market_asks,
                    &market_event_queue,
                    &market_base_vault,
                    &market_quote_vault,
                    &market_authority,
                    &user_src,
                    &user_dst,
                    &pubkey, 
                    _start_bal as u64,
                    0_u64
                )
                ;

    
                if swap_ix.is_err() {
                    return (false, vec![]);
                }
                let mut ixs = vec![];
                let   swap_ix = swap_ix.unwrap();
ixs.push(swap_ix);
if user_dst_acc.is_err() {
    // create ata
    let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account(
        &pubkey,
        &pubkey,
        &mint_out,
        &spl_token::ID

    );
    ixs.insert(0, create_ata_ix);
}

                return (false, ixs);
            } else {
                let model_data_account = self.model_data_account.clone().unwrap();
                  let  swap_ix: Result<Instruction, anchor_lang::prelude::ProgramError> = stable_swap(
                        &stableProgramID,
                        &id,
                        &authority,
                        &open_orders,
                        &base_vault,
                        &quote_vault,
                        &model_data_account,
                        &market_program_id,
                        &market_id,
                        &market_bids,
                        &market_asks,
                        &market_event_queue,
                        &market_base_vault,
                        &market_quote_vault,
                        &market_authority,
                        &user_src,
                        &user_dst,
                        &pubkey, 
                        _start_bal as u64,
                        0_u64
                    );

    
            if swap_ix.is_err() {
                return (false, vec![]);
            }
            let   swap_ix = swap_ix.unwrap();
            let mut ixs = vec![];
            ixs.push(swap_ix);
            if user_src_acc.is_err() {
                let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account(
                    &pubkey,
                    &pubkey,
                    &mint_in,
                    &spl_token::ID
                );
                ixs.insert(0, create_ata_ix);
            
            }
            if user_dst_acc.is_err() {
                // create ata
                let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account(
                    &pubkey,
                    &pubkey,
                    &mint_out,
                    &spl_token::ID
            
                );
                ixs.insert(0, create_ata_ix);
            }
            
                            return (false, ixs);
            }        

    }
    async fn get_quote_with_amounts_scaled_new(
        & self,
        scaled_amount_in: u128,
        mint_in: &Pubkey,
        mint_out: &Pubkey,
        amt1: u128, 
        amt2: u128
    ) -> u128 {
        return 0;
    }
    
    fn get_quote_with_amounts_scaled(
        &mut self,
        scaled_amount_in: u128,
        mint_in: &Pubkey,
        mint_out: &Pubkey,
        program: &Arc<RpcClient >
    ) -> u128 {
        let mut pool_src_amount = self.pool_amounts.get(&self.quote_mint.0.to_string());
        let mut pool_dst_amount = self.pool_amounts.get(&self.base_mint.0.to_string());
        
        if pool_src_amount.is_none() || pool_dst_amount.is_none() {
            return 0;
        }
        let mut pool_src_amount = *pool_src_amount.unwrap();
        let mut pool_dst_amount = *pool_dst_amount.unwrap();
        
        let idx0 = self.base_mint.0.to_string();
        let swap_direction: SwapDirection = if idx0 == mint_out.to_string() {
            SwapDirection::Coin2PC
            
        } else {
            pool_dst_amount = *self.pool_amounts.get(&self.quote_mint.0.to_string()).unwrap();
            pool_src_amount = *self.pool_amounts.get(&self.base_mint.0.to_string()).unwrap();
            SwapDirection::PC2Coin
        };        ;
        let key = (self.open_orders.0, self.authority.0, self.market_id.0, self.id.0);
        if let Some(value) = self.cache.get(&key) {
            // If the result is in the cache, use it.

           let mut amm_open_orders_info = value.0.clone();
          let  mut  amm_authority_info = value.1.clone();
          let mut   market_info = value.2.clone();
          let  mut  amm_info = value.3.clone();
          let mut   amm = value.4.clone();
          let  mut  market_state = value.5.clone();
          let  mut  open_orders = value.6.clone();
          let  mut  market_event_queue_info = value.7.clone();

        let amm_info = AccountInfo::new(
            &self.id.0,
            false,
            false,
            &mut amm_info.lamports,
            &mut amm_info.data,
            &amm_info.owner,
            false,
            Epoch::default(),
        );
        let amm_authority_info = AccountInfo::new(
            &self.authority.0,
            false,
            false,
            &mut amm_authority_info.lamports,
            &mut amm_authority_info.data,
            &amm_authority_info.owner,
            false,
            Epoch::default(),
        );
        let amm_open_orders_info = AccountInfo::new(
            &self.open_orders.0,
            false,
            false,
            &mut amm_open_orders_info.lamports,
            &mut amm_open_orders_info.data,
            &amm_open_orders_info.owner,
            false,
            Epoch::default(),
        );
        let market_info = AccountInfo::new(
            &self.market_id.0,
            false,
            false,
            &mut market_info.lamports,
            &mut market_info.data,
            &market_info.owner,
            false,
            Epoch::default(),
        );

        let market_event_queue_info = AccountInfo::new(
            &self.market_event_queue.0,
            false,
            false,
            &mut market_event_queue_info.lamports,
            &mut market_event_queue_info.data,
            &market_event_queue_info.owner,
            false,
            Epoch::default(),
        );
        let total_pc_without_take_pnl;
        let total_coin_without_take_pnl;
           let atuplemaybe =
                Calculator::calc_total_without_take_pnl(
                    pool_src_amount as u64,
                    pool_dst_amount as u64,
                    &open_orders,
                    &amm,
                    &Box::new(market_state),
                    &market_event_queue_info,
                    &amm_open_orders_info,
                );
                if atuplemaybe.clone().is_err() {
                    println!("raydium err {}", atuplemaybe.clone().err().unwrap());
                    return 0;
                }
                 (total_pc_without_take_pnl, total_coin_without_take_pnl) = atuplemaybe.unwrap();
        let swap_fee = U128::from(scaled_amount_in)
            .checked_mul(amm.fees.swap_fee_numerator.into())
            .unwrap()
            .checked_ceil_div(amm.fees.swap_fee_denominator.into())
            .unwrap()
            .0;
        let swap_in_after_deduct_fee = U128::from(scaled_amount_in).checked_sub(swap_fee).unwrap();
        let swap_amount_out = Calculator::swap_token_amount_base_in(
            swap_in_after_deduct_fee,
            total_pc_without_take_pnl.into(),
            total_coin_without_take_pnl.into(),
            swap_direction,
        );
        return swap_amount_out.as_u128();
        } else {
            let connection = program.clone();
            let mut amm_open_orders_info = connection.get_account(&self.open_orders.0).unwrap();
            let mut amm_authority_info = connection.get_account(&self.authority.0).unwrap();
            let mut market_info = connection.get_account(&self.market_id.0).unwrap();
            let mut amm_info: Account = connection.get_account(&self.id.0).unwrap();

            let amm_info2 = AccountInfo::new(
                &self.id.0,
                false,
                false,
                &mut amm_info.lamports,
                &mut amm_info.data,
                &amm_info.owner,
                false,
                Epoch::default(),
            );
            let amm = AmmInfo::load_mut_checked(&amm_info2, &Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8").unwrap()).unwrap();
          
        let amm_authority_info2 = AccountInfo::new(
            &self.authority.0,
            false,
            false,
            &mut amm_authority_info.lamports,
            &mut amm_authority_info.data,
            &amm_authority_info.owner,
            false,
            Epoch::default(),
        );
        let amm_open_orders_info2 = AccountInfo::new(
            &self.open_orders.0,
            false,
            false,
            &mut amm_open_orders_info.lamports,
            &mut amm_open_orders_info.data,
            &amm_open_orders_info.owner,
            false,
            Epoch::default(),
        );
        let market_info2 = AccountInfo::new(
            &self.market_id.0,
            false,
            false,
            &mut market_info.lamports,
            &mut market_info.data,
            &market_info.owner,
            false,
            Epoch::default(),
        );
        let  mut market_event_queue_info = connection.get_account(&self.market_event_queue.0).unwrap();
    
    
                let (market_state, open_orders) = Processor::load_serum_market_order(
                    &market_info2,
                    &amm_open_orders_info2,
                    &amm_authority_info2,
                    &amm,
                    false,
                ).unwrap();
                let mut market_event_queue_info = connection.get_account(&self.market_event_queue.0).unwrap();
                let mut cache = self.cache.clone();
                let mut amm_info: Account = connection.get_account(&self.id.0).unwrap();

                cache.insert((self.open_orders.0, self.authority.0, self.market_id.0, self.id.0), (amm_open_orders_info.clone(), amm_authority_info.clone(), market_info.clone(), amm_info.clone(), *amm, *market_state, Box::new(*open_orders), market_event_queue_info.clone()));
                self.cache = cache;
                let amm_open_orders_info = AccountInfo::new(
                    &self.open_orders.0,
                    false,
                    false,
                    &mut amm_open_orders_info.lamports,
                    &mut amm_open_orders_info.data,
                    &amm_open_orders_info.owner,
                    false,
                    Epoch::default(),
                );
                let market_info = AccountInfo::new(
                    &self.market_id.0,
                    false,
                    false,
                    &mut market_info.lamports,
                    &mut market_info.data,
                    &market_info.owner,
                    false,
                    Epoch::default(),
                );
        
                let market_event_queue_info = AccountInfo::new(
                    &self.market_event_queue.0,
                    false,
                    false,
                    &mut market_event_queue_info.lamports,
                    &mut market_event_queue_info.data,
                    &market_event_queue_info.owner,
                    false,
                    Epoch::default(),
                );
                let total_pc_without_take_pnl;
                let total_coin_without_take_pnl;
                   let atuplemaybe =
                        Calculator::calc_total_without_take_pnl(
                            pool_src_amount as u64,
                            pool_dst_amount as u64,
                            &open_orders,
                            &amm,
                            &(market_state),
                            &market_event_queue_info,
                            &amm_open_orders_info,
                        );
                        if atuplemaybe.clone().is_err() {
                            println!("raydium err {}", atuplemaybe.clone().err().unwrap());
                            return 0;
                        }
                         (total_pc_without_take_pnl, total_coin_without_take_pnl) = atuplemaybe.unwrap();
                let swap_fee = U128::from(scaled_amount_in)
                    .checked_mul(amm.fees.swap_fee_numerator.into())
                    .unwrap()
                    .checked_ceil_div(amm.fees.swap_fee_denominator.into())
                    .unwrap()
                    .0;
                let swap_in_after_deduct_fee = U128::from(scaled_amount_in).checked_sub(swap_fee).unwrap();
                let swap_amount_out = Calculator::swap_token_amount_base_in(
                    swap_in_after_deduct_fee,
                    total_pc_without_take_pnl.into(),
                    total_coin_without_take_pnl.into(),
                    swap_direction,
                );
                return swap_amount_out.as_u128();
        }

    }



    fn can_trade(&mut self, _mint_in: &Pubkey, _mint_out: &Pubkey) -> bool {
        if self.pool_amounts.get(&_mint_in.to_string()).is_none() ||

            self.pool_amounts.get(&_mint_out.to_string()).is_none() {
            return false;
        }
        let mints = self.get_mints();
        let connection = RpcClient::new("https://jarrett-solana-7ba9.mainnet.rpcpool.com/8d890735-edf2-4a75-af84-92f7c9e31718".to_string());
        let test = self.get_quote_with_amounts_scaled(
1000000,
            &mints[0],
            &mints[1],
           &Arc::new(connection))
            ;
        if test == 0 {
            return false;
        }
        self.pool_amounts.get(&_mint_in.to_string()).unwrap() > &0 
            && self.pool_amounts.get(&_mint_out.to_string()).unwrap() > &0

        
    }fn get_name(&self) -> String {
        "Raydium".to_string()
    }

    fn get_own_addr(&self) -> Pubkey {
        self.id.0
    }
    fn get_update_accounts(&self) -> Vec<Pubkey> {
        vec![self.base_vault.0, self.quote_vault.0]
    }

    fn set_update_accounts(&mut self, accounts: Vec<Option<&Account>>, _cluster: Cluster) {
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
        let connection = RpcClient::new(_cluster.url().to_string());
        self.pool_amounts.insert(id0.clone(), amount0);
        self.pool_amounts.insert(id1.clone(), amount1);
       
    }

    fn set_update_accounts2(&mut self, _pubkey: Pubkey, data: &[u8], _cluster: Cluster) {
        let acc_data0 = data;
        let mut amount0 = unpack_token_account(acc_data0);
        
        let _mint = amount0.mint;
        let _mint = amount0.mint;
        let id0 = self.base_mint.0.to_string();
        let id1 = self.quote_mint.0.to_string();
        println!("raydium {} {} {} {}", _mint.to_string(), id0, id1, amount0.amount);

 
                if _mint.to_string() == id0 {
            self.pool_amounts
                .entry(id0.clone())
                .and_modify(|e| *e = amount0.amount as u128)
                .or_insert(amount0.amount as u128);
        } else if _mint.to_string() == id1 {
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
