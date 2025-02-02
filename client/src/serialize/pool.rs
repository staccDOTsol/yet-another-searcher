use serde;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JSONFeeStructure2 {
    pub trade_fee_numerator: u64,
    pub trade_fee_denominator: u64,
    pub owner_trade_fee_numerator: u64,
    pub owner_trade_fee_denominator: u64,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JSONFeeStructure {
    pub trader_fee: Fraction,
    pub owner_fee: Fraction,
}

#[derive(Deserialize, Serialize, Debug, Clone)]

pub struct Fraction {
    pub numerator: u64,
    pub denominator: u64,
}
