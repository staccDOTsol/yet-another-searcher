use async_trait::async_trait;
use bytemuck::bytes_of;
use chrono::Utc;
use solana_sdk::signer::Signer;
use core::panic;
use std::num::NonZeroU64;
use std::sync::{Arc, Mutex};

type ShardedDb = Arc<Mutex<HashMap<String, Account>>>;
use crate::monitor::pools::PoolOperations;
use crate::serialize::token::WrappedPubkey;
use anchor_client::Cluster;
use openbook_dex::matching::OrderType;
use openbook_dex::matching::Side;
use openbook_dex::state::AccountFlag;
use serde;
use serde::{Deserialize, Serialize};
use solana_sdk::signature::{Keypair, read_keypair_file};
use std::collections::HashMap;
use std::fmt::Debug;

use std::vec;

use crate::constants::*;
use crate::monitor::pools::PoolType;

use crate::utils::derive_token_address;

use solana_sdk::pubkey::Pubkey;

use openbook_dex::{critbit::SlabView, matching::OrderBookState, state::Market};
use std::ops::DerefMut;

use solana_sdk::instruction::Instruction;

use crate::monitor::pool_utils::serum::*;
use solana_sdk::account::Account;
use solana_sdk::account_info::AccountInfo;
use solana_sdk::clock::Epoch;

use std::str::FromStr;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SerumPool {
    pub own_address: WrappedPubkey,
    pub base_mint: WrappedPubkey,
    pub quote_mint: WrappedPubkey,
    pub base_scale: u64,
    pub quote_scale: u64,
    pub base_vault: WrappedPubkey,
    pub quote_vault: WrappedPubkey,
    pub request_queue: WrappedPubkey,
    pub event_queue: WrappedPubkey,
    pub bids: WrappedPubkey,
    pub asks: WrappedPubkey,
    pub vault_signer_nonce: String,
    pub fee_rate_bps: String,
    // !!
    #[serde(skip)]
    pub accounts: Option<Vec<Option<Account>>>,
    #[serde(skip)]
    pub open_orders: Option<HashMap<String, String>>,
}

fn gen_vault_signer_seeds<'a>(nonce: &'a u64, market: &'a Pubkey) -> [&'a [u8]; 2] {
    [market.as_ref(), bytes_of(nonce)]
}

// Returns the amount of lots for the base currency of a trade with `size`.
fn coin_lots(market: &Market, size: u64) -> NonZeroU64 {
    NonZeroU64::new(size.checked_div(market.coin_lot_size).unwrap()).unwrap()
}

#[inline]
pub fn gen_vault_signer_key(nonce: u64, market: &Pubkey, program_id: &Pubkey) -> Pubkey {
    let seeds = gen_vault_signer_seeds(&nonce, market);
    Pubkey::create_program_address(&seeds, program_id).unwrap()
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

struct Iteration {
    amount_in: u64,
    amount_out: u64,
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

impl PoolOperations for SerumPool {
    fn get_name(&self) -> String {
        "Serum".to_string()
    }

    fn clone_box(&self) -> Box<dyn PoolOperations> {
        Box::new(self.clone())
    }
    fn get_own_addr(&self) -> Pubkey {
        self.own_address.0
    }
    fn get_pool_type(&self) -> PoolType {
        PoolType::SerumPoolType
    }
    fn get_update_accounts(&self) -> Vec<Pubkey> {
        vec![self.own_address.0, self.bids.0, self.asks.0]
    }

    fn set_update_accounts(&mut self, accounts: Vec<Option<Account>>, cluster: Cluster) {
        self.accounts = Some(accounts);

        let oo_path = match cluster {
            Cluster::Localnet => "./serum_open_orders.json",
            Cluster::Mainnet => "./serum_open_orders.json",
            _ => panic!("clsuter {} not supported", cluster),
        };
        let oo_str = std::fs::read_to_string(oo_path).unwrap();
        let oo_book: HashMap<String, String> = serde_json::from_str(&oo_str).unwrap();
        self.open_orders = Some(oo_book);
    }

    fn set_update_accounts2(&mut self, _pubkey: Pubkey, data: &[u8], _cluster: Cluster) {
        let testing = self.accounts.clone();
        let mut taccs = vec![];
        if testing.is_some() {
            taccs = testing.unwrap();
        } else {
            return;
        }
        println!("length acocunts {}", taccs.len());
        if taccs.len() < 3 {
            return;
        }
        println!("made it here 3");

        let flags = Market::account_flags(data);
        if flags.is_err() {
            return;
        }
        let flags = flags.unwrap();
        if flags.intersects(AccountFlag::Bids) {
            println!("made it here 2.5");
            if taccs.len() < 2 {
                taccs.push(Some(Account {
                    lamports: 0,
                    data: data.to_vec(),
                    owner: Pubkey::default(),
                    executable: false,
                    rent_epoch: 0,
                }));
            }
            let mut bids = taccs.get(1).unwrap().clone().unwrap();
            bids.data = data.to_vec();
            let tval = taccs.get(2);
            if tval.is_none() {
                self.accounts = Some(vec![taccs.get(0).unwrap().clone(), Some(bids)]);
            } else {
                self.accounts = Some(vec![
                    taccs.get(0).unwrap().clone(),
                    Some(bids),
                    taccs.get(2).unwrap().clone(),
                ]);
            }
        }
        if flags.intersects(AccountFlag::Asks) {
            if taccs.len() < 3 {
                println!("made it here 32");
                taccs.push(Some(Account {
                    lamports: 0,
                    data: data.to_vec(),
                    owner: Pubkey::default(),
                    executable: false,
                    rent_epoch: 0,
                }));
                self.accounts = Some(vec![
                    taccs.get(0).unwrap().clone(),
                    taccs.get(1).unwrap().clone(),
                    (taccs.get(2).unwrap().clone()),
                ]);
            } else {
                let mut asks = taccs.get(2).unwrap().clone().unwrap();
                asks.data = data.to_vec();
                println!("made it here 3.52");
                self.accounts = Some(vec![
                    taccs.get(0).unwrap().clone(),
                    taccs.get(1).unwrap().clone(),
                    Some(asks),
                ]);
            }
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
            self.base_scale
        } else if *mint == self.quote_mint.0 {
            self.quote_scale
        } else {
            panic!("invalid mint bro")
        }
    }

    fn get_quote_with_amounts_scaled(
        & self,
        amount_in: u128,
        mint_in: &Pubkey,
        _mint_out: &Pubkey,
    ) -> u128 {
        let market_pk = self.own_address.0;
        let fee_tier = FeeTier::from_srm_and_msrm_balances(&market_pk, 0, 0);
        let mut iteration = Iteration {
            amount_in: amount_in as u64,
            amount_out: 0,
        };
        if self.accounts.is_none() {
            println!("made it here 1");
            return 0;
        }
        if self.accounts.as_ref().unwrap().len() < 2 {
            println!("made it here 3");
            return 0;
        }
        let market_acc = &self.accounts.as_ref().unwrap()[0];
        let bids_acc = &self.accounts.as_ref().unwrap()[1];
        let tval = self.accounts.as_ref().unwrap();
        if tval.len() < 3 {
            println!("made it here 4");
            return 0;
        }

        let asks_acc = &self.accounts.as_ref().unwrap()[2];

        // clone accounts for simulation (improve later?)
        let market_acc = &mut market_acc.clone().unwrap();
        let bid_acc = &mut bids_acc.clone().unwrap();
        let ask_acc = &mut asks_acc.clone().unwrap();

        let market_acc_info = &account_info(&self.own_address.0, market_acc);

        let bids_acc = &account_info(&self.bids.0, bid_acc);
        let asks_acc = &account_info(&self.asks.0, ask_acc);

        let m = Market::load(market_acc_info, &SERUM_PROGRAM_ID, false);
        if !m.is_ok() {
            println!("{}", "m is none");
            return 0;
        }
        let mut market = m.unwrap();
        let mut bids = market.load_bids_mut(bids_acc).unwrap();
        let mut asks = market.load_asks_mut(asks_acc).unwrap();
        // are these ordered correctly?
        let mut ob = OrderBookState {
            bids: bids.deref_mut(),
            asks: asks.deref_mut(),
            market_state: market.deref_mut(),
        };

        if *mint_in == self.quote_mint.0 {
            // bid: quote -> base
            let mut count = 0;
            loop {
                count += 1;
                let done = bid_iteration(&mut iteration, &fee_tier, &mut ob);
                if done || iteration.amount_out == 0 || count == 5 {
                    break;
                }
            }
            iteration.amount_out as u128
        } else if *mint_in == self.base_mint.0 {
            // ask: base -> quote
            let mut count = 0;
            loop {
                count += 1;
                let done = ask_iteration(&mut iteration, &fee_tier, &mut ob);
                if done || iteration.amount_in == 0 || count == 5 {
                    break;
                }
            }
            iteration.amount_out as u128
        } else {
            println!("{}", 0);
            0
        }
    }

async    fn swap_ix(
        &self,
        mint_in: &Pubkey,
        _mint_out: &Pubkey,
        start_bal: u128
    ) -> (bool, Vec<Instruction>) {
        let oos = self.open_orders.as_ref().unwrap();
        let open_orders =
            Pubkey::from_str(oos.get(&self.own_address.0.to_string()).unwrap()).unwrap();

        let _swap_state = Pubkey::from_str("8cjtn4GEw6eVhZ9r1YatfiU65aDEBf1Fof5sTuuH6yVM").unwrap();
        let _space = 3228;
        let owner3 = Arc::new(read_keypair_file("/Users/stevengavacs/.config/solana/id.json".clone()).unwrap());

        let owner = owner3.try_pubkey().unwrap();

        let base_ata = derive_token_address(&owner, &self.base_mint);
        let quote_ata = derive_token_address(&owner, &self.quote_mint);

        let side = if *mint_in == self.quote_mint.0 {
            Side::Bid
        } else {
            Side::Ask
        };
        let payer_acc = if side == Side::Ask {
            base_ata
        } else {
            quote_ata
        };
        let _side = if side == openbook_dex::matching::Side::Ask {
            openbook_dex::matching::Side::Ask
        } else {
            openbook_dex::matching::Side::Bid
        };
        let vault_signer_nonce = self.vault_signer_nonce.parse::<u64>().unwrap();
        let _vault_signer =
            gen_vault_signer_key(vault_signer_nonce, &self.own_address.0, &SERUM_PROGRAM_ID);
        let limit_price;
        let max_coin_qty;

        let max_native_pc_qty_including_fees;
        if _side == openbook_dex::matching::Side::Ask {
            limit_price = NonZeroU64::new(1).unwrap();
            max_coin_qty = NonZeroU64::MIN;
            max_native_pc_qty_including_fees = NonZeroU64::new(start_bal as u64 - 1).unwrap();
        } else {
            limit_price = NonZeroU64::MAX;
            max_coin_qty = NonZeroU64::new(start_bal as u64 - 1).unwrap();
            max_native_pc_qty_including_fees = NonZeroU64::MIN;
        }
        let limit: u16 = 3;

        let now = Utc::now();
        let ts = now.timestamp();
        println!("{}", ts);
        (
            (true),
            vec![openbook_dex::instruction::new_order(
                &self.own_address.0,
                &open_orders,
                &self.request_queue.0,
                &self.event_queue.0,
                &self.bids.0,
                &self.asks.0,
                &payer_acc,
                &owner,
                &self.base_vault.0,
                &self.quote_vault.0,
                &TOKEN_PROGRAM_ID,
                &solana_sdk::sysvar::rent::id(),
                None,
                &Pubkey::from_str("srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX").unwrap(),
                _side,
                limit_price,
                max_coin_qty,
                OrderType::Limit,
                0,
                openbook_dex::instruction::SelfTradeBehavior::DecrementTake,
                limit,
                max_native_pc_qty_including_fees,
                ts + 40000,
            )
            .unwrap()],
        )
    }

    fn can_trade(&self, mint_in: &Pubkey, _mint_out: &Pubkey) -> bool {
        let market_acc = &self.accounts.as_ref().unwrap()[0];
        let bids_acc = &self.accounts.as_ref().unwrap()[1];
        let asks_acc = &self.accounts.as_ref().unwrap()[2];

        // clone accounts for simulation (improve later?)
        let market_acc = &mut market_acc.clone().unwrap();
        let bid_acc = &mut bids_acc.clone().unwrap();
        let ask_acc = &mut asks_acc.clone().unwrap();

        let market_acc_info = &account_info(&self.own_address.0, market_acc);
        let bids_acc = &account_info(&self.bids.0, bid_acc);
        let asks_acc = &account_info(&self.asks.0, ask_acc);

        let m = Market::load(market_acc_info, &SERUM_PROGRAM_ID, false);
        if m.is_err() {
            return false;
        }
        let market = m.unwrap();
        let bids = market.load_bids_mut(bids_acc).unwrap();
        let asks = market.load_asks_mut(asks_acc).unwrap();

        // is there a bid or ask we can trade with???
        if *mint_in == self.quote_mint.0 {
            // bid: quote -> base
            match asks.find_min() {
                // min = best ask
                Some(_) => true,
                None => false,
            }
        } else if *mint_in == self.base_mint.0 {
            // ask: base -> quote
            match bids.find_max() {
                Some(_) => true,
                None => false,
            }
        } else {
            panic!("invalid mints");
        }
    }
}
