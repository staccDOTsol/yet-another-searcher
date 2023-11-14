use crate::pool::PoolOperations;
use crate::serialize::pool::JSONFeeStructure;
use crate::serialize::token::{unpack_token_account, Token, WrappedPubkey};
use serde;
use anchor_client::Client;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::read_keypair_file;
use solana_sdk::transaction::Transaction;
use std::rc::Rc;
use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::signature::Signer;

use serde::{Deserialize, Serialize};
use solana_sdk::account::Account;
use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::Cluster;
use anchor_client::Program;

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
    fn swap_ix(
        &self,
        program: &Program,
        owner: &Pubkey,
        mint_in: &Pubkey,
        mint_out: &Pubkey,
    ) -> Vec<Instruction> {
        let swap_state = Pubkey::from_str("8cjtn4GEw6eVhZ9r1YatfiU65aDEBf1Fof5sTuuH6yVM").unwrap();
        let user_src = derive_token_address(owner, mint_in);
        let user_dst = derive_token_address(owner, mint_out);

        let cluster = Cluster::Mainnet;


        let owner_kp_path = match cluster {
            Cluster::Localnet => "../../mainnet_fork/localnet_owner.key",
            Cluster::Mainnet => {
                "/Users/stevengavacs/.config/solana/id.json"
            }
            _ => panic!("shouldnt get here"),
        };

        // ** setup RPC connection
        let connection_url = match cluster {
            Cluster::Mainnet => {
                "https://rpc.shyft.to?api_key=jdXnGbRsn0Jvt5t9"
            }
            _ => cluster.url(),
        };

        let send_tx_connection =
            RpcClient::new_with_commitment(cluster.url(), CommitmentConfig::recent());
    
        // setup anchor things
        let owner2 = read_keypair_file(owner_kp_path.clone()).unwrap();
        let rc_owner = Rc::new(read_keypair_file(owner_kp_path.clone()).unwrap());
        let provider = Client::new_with_options(
            cluster.clone(),
            rc_owner.clone(),
            CommitmentConfig::recent(),
        );
        let program = provider.program(*ARB_PROGRAM_ID);
        let connection = RpcClient::new_with_commitment(connection_url, CommitmentConfig::recent());
        let user_src_account_info = connection.get_account(&user_dst);
        if user_src_account_info.is_err() {
            
        let instructions = program
            .request()
            .instruction(
                spl_associated_token_account::create_associated_token_account(
                    &owner,
                    &owner,
                    mint_out,
                ),
            )
            .instructions().unwrap();
let recent_blockhash = connection.get_latest_blockhash().unwrap();
        let mut tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&owner),
            &[&owner2],
            recent_blockhash
        );
        send_tx_connection.send_and_confirm_transaction_with_spinner_and_config(
            &tx,
            CommitmentConfig::recent(),
            RpcSendTransactionConfig {
                skip_preflight: true,
                ..RpcSendTransactionConfig::default()
            },
        ).unwrap();
    }
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

        swap_ix
    }

    fn get_quote_with_amounts_scaled(
        &self,
        scaled_amount_in: u128,
        mint_in: &Pubkey,
        mint_out: &Pubkey,
    ) -> u128 {
        if !self.pool_amounts.contains_key(&mint_in.to_string())
            || !self.pool_amounts.contains_key(&mint_out.to_string())
        {
            return 0;
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

        get_pool_quote_with_amounts(
            scaled_amount_in,
            ctype,
            self.amp,
            &fees,
            *pool_src_amount,
            *pool_dst_amount,
            None,
        )
        .unwrap()
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
