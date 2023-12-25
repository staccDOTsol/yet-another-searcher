use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::signature::read_keypair_file;
use anchor_client::{Client, Cluster};
use async_trait::async_trait;
use solana_sdk::program_pack::Pack;
use solana_sdk::signer::Signer;
use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;

use std::sync::{Arc, Mutex};
type ShardedDb = Arc<Mutex<HashMap<String, Account>>>;


use serde;
use serde::{Deserialize, Serialize};

use anchor_client::solana_sdk::pubkey::Pubkey;

use solana_sdk::account::Account;
use solana_sdk::instruction::Instruction;


use tmp::accounts as tmp_accounts;
use tmp::instruction as tmp_ix;

use crate::constants::*;
use crate::monitor::pools::{PoolOperations, PoolType};
use crate::monitor::pool_utils::base::CurveType;
use crate::monitor::pool_utils::{fees::Fees, orca::get_pool_quote_with_amounts};
use crate::serialize::pool::JSONFeeStructure2;
use crate::serialize::token::{ Token, WrappedPubkey};
use crate::utils::{derive_token_address, str2pubkey};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AldrinPool {
    pub lp_token_freeze_vault: WrappedPubkey,
    pub pool_mint: WrappedPubkey,
    pub pool_signer: WrappedPubkey,
    pub pool_signer_nonce: u64,
    pub authority: WrappedPubkey,
    pub initializer_account: WrappedPubkey,
    pub fee_base_account: WrappedPubkey,
    pub fee_quote_account: WrappedPubkey,
    pub fee_pool_token_account: WrappedPubkey,
    // !
    pub token_ids: Vec<String>,
    pub tokens: HashMap<String, Token>,
    pub fees: JSONFeeStructure2,
    pub curve_type: u8,
    //
    pub curve: WrappedPubkey,
    pub pool_public_key: WrappedPubkey,
    pub pool_version: u8,
    // to set later
    #[serde(skip)]
    pub pool_amounts: HashMap<String, u128>,
}
#[async_trait]

impl PoolOperations for AldrinPool {
    fn clone_box(&self) -> Box<dyn PoolOperations> {
        Box::new(self.clone())
    }
    fn get_pool_type(&self) -> PoolType {
        PoolType::AldrinPoolType
    }
    async fn swap_ix(
        &self,
        //impl<C: Deref<Target = impl Signer> + Clone> Program<C>
        _mint_in: &Pubkey,
        mint_out: &Pubkey,
        _start_bal: u128,
    ) -> (bool, Vec<Instruction>) {
        let state_pda = Pubkey::from_str("8cjtn4GEw6eVhZ9r1YatfiU65aDEBf1Fof5sTuuH6yVM").unwrap();

        let _owner_kp_path = "/root/.config/solana/id.json";
        // setup anchor things
        let owner3 = Arc::new(read_keypair_file("/root/.config/solana/id.json").unwrap());
        
        let owner = owner3.try_pubkey().unwrap()    ;
        let provider = Client::new_with_options(
            Cluster::Mainnet,
            owner3.clone(),
            CommitmentConfig::processed(),
        );
        let program = provider.program(*ARB_PROGRAM_ID).unwrap();
        let base_token_mint = &self.token_ids[0];
        let quote_token_mint = &self.token_ids[1];

        let base_token_vault = self.tokens.get(base_token_mint).unwrap().addr.0;
        let quote_token_vault = self.tokens.get(quote_token_mint).unwrap().addr.0;

        let is_inverted = &mint_out.to_string() == quote_token_mint;
        let user_base_ata =
            derive_token_address(&owner, &Pubkey::from_str(base_token_mint).unwrap());
        let user_quote_ata =
            derive_token_address(&owner, &Pubkey::from_str(quote_token_mint).unwrap());

        let swap_ix;
        if self.pool_version == 1 {
let pool_public_key = self.pool_public_key.0;
let pool_signer = self.pool_signer.0;
let pool_mint = self.pool_mint.0;
let fee_pool_token_account = self.fee_pool_token_account.0;
            swap_ix = program
                .request()
                .accounts(tmp_accounts::AldrinSwapV1 {
                    pool_public_key,
                    pool_signer,
                    pool_mint,
                    base_token_vault,
                    quote_token_vault,
                    fee_pool_token_account,
                    user_transfer_authority: owner,
                    user_base_ata,
                    user_quote_ata,
                    // ...
                    aldrin_v1_program: *ALDRIN_V1_PROGRAM_ID,
                    token_program: *TOKEN_PROGRAM_ID,
                    swap_state: state_pda,
                })
                .args(tmp_ix::AldrinSwapV1 { is_inverted })
                .instructions()
                .unwrap();
            
        } else {
            let pool_public_key = self.pool_public_key.0;
            let pool_signer = self.pool_signer.0;
            let pool_mint = self.pool_mint.0;
            let fee_pool_token_account = self.fee_pool_token_account.0;
            let curve = self.curve.0;
            swap_ix =  program
                .request()
                .accounts(tmp_accounts::AldrinSwapV2 {
                    pool_public_key,
                    pool_signer,
                    pool_mint,
                    base_token_vault,
                    quote_token_vault,
                    fee_pool_token_account,
                    user_transfer_authority: owner,
                    user_base_ata,
                    user_quote_ata,
                    // ...
                    aldrin_v2_program: *ALDRIN_V2_PROGRAM_ID,
                    curve,
                    token_program: *TOKEN_PROGRAM_ID,
                    swap_state: state_pda,
                })
                .args(tmp_ix::AldrinSwapV2 { is_inverted })
                .instructions()
                .unwrap();

        }
        (false, swap_ix)
    }

    fn get_quote_with_amounts_scaled(
        & self,
        scaled_amount_in: u128,
        mint_in: &Pubkey,
        mint_out: &Pubkey,
    ) -> u128 {
        if !self.pool_amounts.contains_key(&mint_in.to_string())
            || !self.pool_amounts.contains_key(&mint_out.to_string())
        {
            println!("aldrin pool amounts not found");
            return 0;
        }
        let pool_src_amount = *self.pool_amounts.get(&mint_in.to_string()).unwrap();
        let pool_dst_amount = *self.pool_amounts.get(&mint_out.to_string()).unwrap();

        // compute fees
        let fees = Fees {
            trade_fee_numerator: self.fees.trade_fee_numerator,
            trade_fee_denominator: self.fees.trade_fee_denominator,
            owner_trade_fee_numerator: self.fees.owner_trade_fee_numerator,
            owner_trade_fee_denominator: self.fees.owner_trade_fee_denominator,
            owner_withdraw_fee_numerator: 0,
            owner_withdraw_fee_denominator: 0,
            host_fee_numerator: 0,
            host_fee_denominator: 0,
        };

        let ctype = if self.curve_type == 1 {
            CurveType::Stable
        } else {
            CurveType::ConstantProduct
        };

        // get quote -- works for either constant product or stable swap

        get_pool_quote_with_amounts(
            scaled_amount_in,
            ctype,
            170, // from sdk
            &fees,
            pool_src_amount,
            pool_dst_amount,
            None,
        )
        .unwrap()
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
        self.pool_public_key.0
    }
    fn get_name(&self) -> String {
        if self.pool_version == 1 {
            "AldrinV1".to_string()
        } else {
            "AldrinV2".to_string()
        }
    }

    fn get_update_accounts(&self) -> Vec<Pubkey> {
        // pool vault amount
        // TODO: replace with token_ids + ['addr'] key
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

        let amount0 = spl_token::state::Account::unpack(acc_data0).unwrap().amount as u128;
        let amount1 = spl_token::state::Account::unpack(acc_data1).unwrap().amount as u128;

        self.pool_amounts.insert(id0.clone(), amount0);
        self.pool_amounts.insert(id1.clone(), amount1);
    }
    
    fn set_update_accounts2(&mut self, _pubkey: Pubkey, data: &[u8], _cluster: Cluster) {
        let acc_data0 = data;

        let amount0 = spl_token::state::Account::unpack(acc_data0).unwrap();
        let _mint = amount0.mint;
        let id0 = &self.token_ids[0];
        let id1 = &self.token_ids[1];
        if _mint.to_string() == *id0 {
            self.pool_amounts
                .insert(id0.clone(), amount0.amount as u128);

            } else if _mint.to_string() == *id1 {
                
                            self.pool_amounts
                .insert(id1.clone(), amount0.amount as u128);
        }
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
