use boilerplate::Boilerplate;
use serde;
use serde::{Deserialize, Serialize};

/*
parsed: {
    pubkey: 'EaXdHx7x3mdGA38j5RSmKYSXMzAFzzUXCLNBEDXDn1d5',
    lamports: 457104960,
    owner: 'srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX',
    executable: false,
    rent_epoch: 0,
    data: ['AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA'... 77400 more characters,
      'base64'
    ],
    write_version: 939620503313
*/
  #[derive(Deserialize, Serialize, Debug, Clone)]
  pub(crate) struct ShyftInfo {
      pub pubkey: String,
      pub lamports: u64,
      pub owner: String,
      pub executable: bool,
      pub rent_epoch: u64,
      pub data: Vec<String>,
      pub write_version: u64,
  }
  #[derive(Deserialize, Serialize, Debug, Clone)]
  pub(crate) struct SetJson {
      pub status_code: u8,
  }
  
  #[derive(Deserialize, Serialize, Debug, Clone)]
  pub(crate) struct VecHomeHtml {
      pub account: Vec<HomeHtml2>,
  }
  #[derive(Deserialize, Serialize, Debug, Clone)]
  pub(crate) struct HomeHtml2 {
      pub account: HomeHtml,
  }
  #[derive(Boilerplate, Deserialize, Serialize, Debug, Clone)]
  pub(crate) struct HomeHtml {
      pub parsed: ShyftInfo,
  }