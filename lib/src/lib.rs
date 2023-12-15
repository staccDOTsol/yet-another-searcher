use solana_client::rpc_client::RpcClient;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;

use crate::error::{FlashProgramError, FlashSdkError};
use crate::math::{Decimal, Rate, TryMul};
use crate::types::Reserve;

pub mod error;
pub mod instruction;
pub mod math;
pub mod types;

pub const FLASH_LOAN_ID: &str = "F1aShdFVv12jar3oM2fi6SDqbefSnnCVRzaxbPH3you7";

/// Calculates total fees for flash borrow of specified `amount`
/// Type of token to be borrowed is determined by `reserve`
/// Use this function when you have reserve's Pubkey and already inited RpcClient.
pub fn flash_loan_fee_via_rpc(
    reserve_key: &Pubkey,
    borrow_amount: u64,
    rpc_client: &RpcClient,
) -> Result<u64, FlashSdkError> {
    let reserve = get_reserve(reserve_key, rpc_client)?;

    flash_loan_fee(&reserve, borrow_amount)
}

/// Calculates total fees for flash borrow of specified `amount`
/// Type of token to be borrowed is determined by `reserve`
pub fn flash_loan_fee(reserve: &Reserve, borrow_amount: u64) -> Result<u64, FlashSdkError> {
    let borrow_fee_rate = Rate::from_scaled_val(reserve.config.fees.flash_loan_fee_wad);
    let texture_fee_rate = Rate::from_percent(reserve.config.fees.texture_fee_percentage);
    let borrow_amount = Decimal::from(borrow_amount);

    if borrow_fee_rate > Rate::zero() && borrow_amount > Decimal::zero() {
        let minimum_fee = if texture_fee_rate > Rate::zero() {
            2u64
        } else {
            1u64
        };

        let borrow_fee_amount = borrow_amount
            .try_mul(borrow_fee_rate)
            .map_err(|_| FlashSdkError::FlashError(FlashProgramError::MathOverflow))?;

        let borrow_fee_decimal = borrow_fee_amount.max(minimum_fee.into());
        if borrow_fee_decimal >= borrow_amount {
            return Err(FlashSdkError::FlashError(FlashProgramError::BorrowTooSmall));
        }

        let borrow_fee = borrow_fee_decimal
            .try_round_u64()
            .map_err(|_| FlashSdkError::FlashError(FlashProgramError::MathOverflow))?;

        Ok(borrow_fee)
    } else {
        Ok(0)
    }
}

/// Returns maximum amount of tokens which could be flash borrowed from given `reserve`.
/// Use this function when you have reserve's Pubkey and already inited RpcClient.
pub fn available_liquidity_via_rpc(
    reserve_key: &Pubkey,
    rpc_client: &RpcClient,
) -> Result<u64, FlashSdkError> {
    let reserve = get_reserve(reserve_key, rpc_client)?;

    Ok(available_liquidity(&reserve))
}

/// Returns deserialized Reserve structure getting it from account specified by `reserve_key`
/// via provided RpcClient
pub fn get_reserve(reserve_key: &Pubkey, rpc_client: &RpcClient) -> Result<Reserve, FlashSdkError> {
    let raw_data = rpc_client
        .get_account_data(reserve_key)
        .map_err(|_| FlashSdkError::RpcError)?;

    let reserve = Reserve::unpack(&*raw_data).map_err(|_| FlashSdkError::DeserializationError)?;

    Ok(reserve)
}

/// Returns maximum amount of tokens which could be flash borrowed from given `reserve`.
/// Use this function when you have deserialized Reserve structure.
pub fn available_liquidity(reserve: &Reserve) -> u64 {
    reserve.liquidity.available_amount
}
