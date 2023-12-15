use boilerplate::Boilerplate;
use serde;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct ShyftInfo {
    pub pubkey: String,
    pub lamports: u64,
    pub owner: String,
    pub slot : u64,
    pub executable: bool,
    pub rent_epoch: u64,
    pub data: ShyftData,
    pub write_version: u64,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct SetJson {
    pub status_code: u8,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct ShyftData {
    pub version: u8,
    pub bump: Vec<u8>,
    pub owner: String,
    pub asset_id: String,
    pub expirty: u64,
    pub raw: String,
    }
#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct HomeHtml2 {
    pub account: HomeHtml,
    pub account_info: ShyftInfo,
}
