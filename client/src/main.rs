use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::read_keypair_file;
use anchor_client::solana_sdk::signature::{Keypair, Signer};
use boilerplate::Boilerplate;

use serde;
use serde::{Deserialize, Serialize};

use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
struct Config {
    msg: String,
}
#[derive(Debug)]
struct AppState {
    cfg: Mutex<Config>,
}
use std::net::ToSocketAddrs;
use axum::Error;
use axum::routing::post;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    set_header::SetResponseHeaderLayer,
  };
use std::path::PathBuf;
use axum::{
    body,
    extract::{Extension, Json, Path, Query},
    http::{header, HeaderMap, HeaderValue, StatusCode, Uri},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Router
  };
use  axum_server::Handle;

use derive_more::FromStr;
use num_traits::pow;
use solana_client::client_error::ClientError;
use solana_sdk::signature::{Signature};
use solana_sdk::signers::Signers;
use solana_sdk::transaction::Transaction;
use structopt::StructOpt;

use flash_loan_sdk::instruction::{flash_borrow, flash_repay};
use flash_loan_sdk::{available_liquidity, flash_loan_fee, get_reserve, FLASH_LOAN_ID};


use anchor_client::{Client, Cluster};

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::io::Write;
use std::rc::Rc;
use std::str::FromStr;

use std::borrow::Borrow;
use std::vec;

use clap::Parser;

use log::{debug, info, warn};
use solana_sdk::account::Account;

use client::arb::*;
use client::constants::*;
use client::pool::{pool_factory, PoolDir, PoolOperations, PoolType};
use client::serialize::token::unpack_token_account;
use client::utils::{
    derive_token_address, read_json_dir, PoolEdge, PoolGraph, PoolIndex, PoolQuote,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(short, long)]
    pub cluster: String,
    #[clap(short, long)]
    pub flashing: String,
}

fn add_pool_to_graph<'a>(
    graph: &mut PoolGraph,
    idx0: PoolIndex,
    idx1: PoolIndex,
    quote: &PoolQuote,
) {
    // idx0 = A, idx1 = B
    let edges = graph
        .0
        .entry(idx0)
        .or_insert_with(|| PoolEdge(HashMap::new()));
    let quotes = edges.0.entry(idx1).or_insert_with(|| vec![]);
    quotes.push(quote.clone());
}

#[derive(Debug)]
pub enum ServerError {
  BadRequest(String),
  Internal(Error),
  NotAcceptable(String),
  NotFound(String),
}

pub type ServerResult<T> = Result<T, ServerError>;

impl IntoResponse for ServerError {
  fn into_response(self) -> Response {
    match self {
      Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message).into_response(),
      Self::Internal(error) => {
        eprintln!("error serving request: {error}");
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          StatusCode::INTERNAL_SERVER_ERROR
            .canonical_reason()
            .unwrap_or_default(),
        )
          .into_response()
      }
      Self::NotAcceptable(content_type) => (
        StatusCode::NOT_ACCEPTABLE,
        format!("inscription content type `{content_type}` is not acceptable"),
      )
        .into_response(),
      Self::NotFound(message) => (
        StatusCode::NOT_FOUND,
        [(header::CACHE_CONTROL, HeaderValue::from_static("no-store"))],
        message,
      )
        .into_response(),
    }
  }
}

pub trait OptionExt<T> {
  fn ok_or_not_found<F: FnOnce() -> S, S: Into<String>>(self, f: F) -> ServerResult<T>;
}

impl<T> OptionExt<T> for Option<T> {
  fn ok_or_not_found<F: FnOnce() -> S, S: Into<String>>(self, f: F) -> ServerResult<T> {
    match self {
      Some(value) => Ok(value),
      None => Err(ServerError::NotFound(f().into() + " not found")),
    }
  }
}

impl From<Error> for ServerError {
  fn from(error: Error) -> Self {
    Self::Internal(error)
  }
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct Data {
    pub version: u8,
    pub bump: Vec<u8>,
    pub owner: String,
    pub asset_id: String,
    pub amount: String,
    pub currency: Option<String>,
    pub expiry: String,
    pub private_taker: Option<bool>,
    pub maker_broker: Option<String>,
    pub raw: String  
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct ShyftInfo {
    pub slot: u64,
    pub pubkey: String,
    pub lamports: u64,
    pub owner: String,
    pub data: Data
}
#[derive( Deserialize, Serialize, Debug, Clone)]
pub(crate) struct SetJson {
    pub account: String,
    pub account_info: ShyftInfo,
}
#[derive(Boilerplate, Deserialize, Serialize, Debug, Clone)]
pub(crate) struct HomeHtml {
    pub account: String,
    pub account_info: ShyftInfo,
}
#[axum_macros::debug_handler]

// your existing home function
async fn home(
    Extension(state): Extension<Arc<AppState>>,
    body: Json<HomeHtml>,
) -> ServerResult<Response> {
    
    Ok(
        Json(
            SetJson{
                account: body.account.clone(),
                account_info: body.account_info.clone(),
            }
        ).into_response()
        
        

    )

}

  #[tokio::main]

async fn main() {
println!("Listening on http://localhost:3000") ;
let cfg = Config {
    msg: "cruel".to_string(),
};

let cfg_data = Arc::new(AppState{
    cfg: Mutex::new(cfg.clone()),
});
let router = Router::new()
.route("/", post(home))                           .layer(Extension(cfg_data))

.layer(
  CorsLayer::new()
    .allow_methods([http::Method::POST])
    .allow_origin(Any),
);
let handle = Handle::new();
let addr = ("localhost", 3000)
.to_socket_addrs().unwrap()
.next()
.ok_or_else(|| anyhow::anyhow!("failed to get socket addrs")).unwrap();
println!("addr = {}", addr);
axum_server::Server::bind(addr)
  .handle(handle)
  .serve(router.into_make_service())
    .await.unwrap();
println!("Listening on http://localhost:3000") ;

}
