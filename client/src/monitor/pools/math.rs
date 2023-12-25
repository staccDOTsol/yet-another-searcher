//! Common math routines.
use anyhow::anyhow;
use {
    arrayref::array_ref,
    solana_program::{hash::Hasher, msg, pubkey::Pubkey},
    std::fmt::Display,
};

pub fn checked_add<T>(arg1: T, arg2: T) -> Result<T, anyhow::Error>
where
    T: num_traits::PrimInt + Display,
{
    if let Some(res) = arg1.checked_add(&arg2) {
        Ok(res)
    } else {
        msg!("Error: Overflow in {} + {}", arg1, arg2);
        Err(anyhow!("Error: Overflow in {} + {}", arg1, arg2).into())
    }
}

pub fn checked_sub<T>(arg1: T, arg2: T) -> Result<T, anyhow::Error>
where
    T: num_traits::PrimInt + Display,
{
    if let Some(res) = arg1.checked_sub(&arg2) {
        Ok(res)
    } else {
        msg!("Error: Overflow in {} - {}", arg1, arg2);
        Err(anyhow!("Error: Overflow in {} - {}", arg1, arg2).into())
    }
}

pub fn checked_div<T>(arg1: T, arg2: T) -> Result<T, anyhow::Error>
where
    T: num_traits::PrimInt + Display,
{
    if let Some(res) = arg1.checked_div(&arg2) {
        Ok(res)
    } else {
        msg!("Error: Overflow in {} / {}", arg1, arg2);
        Err(anyhow!("Error: Overflow in {} / {}", arg1, arg2).into())
    }
}

pub fn checked_mul<T>(arg1: T, arg2: T) -> Result<T, anyhow::Error>
where
    T: num_traits::PrimInt + Display,
{
    if let Some(res) = arg1.checked_mul(&arg2) {
        Ok(res)
    } else {
        msg!("Error: Overflow in {} * {}", arg1, arg2);
        Err(anyhow!("Error: Overflow in {} * {}", arg1, arg2).into())
    }
}

pub fn checked_pow<T>(arg: T, exp: usize) -> Result<T, anyhow::Error>
where
    T: num_traits::PrimInt + Display,
{
    if let Some(res) = num_traits::checked_pow(arg, exp) {
        Ok(res)
    } else {
        msg!("Error: Overflow in {} ^ {}", arg, exp);
        Err(anyhow!("Error: Overflow in {} ^ {}", arg, exp).into())
    }
}

pub fn checked_powf(arg: f64, exp: f64) -> Result<f64, anyhow::Error> {
    let res = f64::powf(arg, exp);
    if res.is_finite() {
        Ok(res)
    } else {
        msg!("Error: Overflow in {} ^ {}", arg, exp);
        Err(anyhow!("Error: Overflow in {} ^ {}", arg, exp).into())
    }
}

pub fn checked_powi(arg: f64, exp: i32) -> Result<f64, anyhow::Error> {
    let res = if exp > 0 {
        f64::powi(arg, exp)
    } else {
        // wrokaround due to f64::powi() not working properly on-chain with negative exponent
        1.0 / f64::powi(arg, -exp)
    };
    if res.is_finite() {
        Ok(res)
    } else {
        msg!("Error: Overflow in {} ^ {}", arg, exp);
        Err(anyhow!("Error: Overflow in {} ^ {}", arg, exp).into())
    }
}

pub fn checked_as_u64<T>(arg: T) -> Result<u64, anyhow::Error>
where
    T: Display + num_traits::ToPrimitive + Clone,
{
    let option: Option<u64> = num_traits::NumCast::from(arg.clone());
    if let Some(res) = option {
        Ok(res)
    } else {
        msg!("Error: Overflow in {} as u64", arg);
        Err(anyhow!("Error: Overflow in {} as u64", arg).into())
    }
}

pub fn checked_as_u128<T>(arg: T) -> Result<u128, anyhow::Error>
where
    T: Display + num_traits::ToPrimitive + Clone,
{
    let option: Option<u128> = num_traits::NumCast::from(arg.clone());
    if let Some(res) = option {
        Ok(res)
    } else {
        msg!("Error: Overflow in {} as u128", arg);
        Err(anyhow!("Error: Overflow in {} as u128", arg).into())
    }
}

/// Returns numerator and denominator for the given fee
pub fn get_fee_parts(fee: f64) -> (u64, u64) {
    if fee <= 0.0 || fee > 1.0 {
        return (0, 1);
    }
    let mut numerator = fee;
    let mut denominator = 1u64;
    let mut i = 0;
    while numerator != (numerator as u64) as f64 && i < 6 {
        numerator *= 10.0;
        denominator *= 10;
        i += 1;
    }
    if numerator as u64 == denominator {
        (1, 1)
    } else {
        (numerator as u64, denominator)
    }
}

pub fn get_no_fee_amount(
    amount: u64,
    fee_numerator: u64,
    fee_denominator: u64,
) -> Result<u64, anyhow::Error> {
    if amount == 0 {
        return Ok(0);
    }
    checked_sub(
        amount,
        std::cmp::max(
            checked_as_u64(checked_div(
                checked_mul(amount as u128, fee_numerator as u128)?,
                fee_denominator as u128,
            )?)?,
            1,
        ),
    )
}

pub fn hash_address(current_hash: u64, address: &Pubkey) -> u64 {
    let mut input: Vec<u8>;
    let mut hasher = Hasher::default();
    if current_hash != 0 {
        input = current_hash.to_le_bytes().to_vec();
        input.extend_from_slice(address.as_ref());
    } else {
        input = address.as_ref().to_vec();
    }
    hasher.hash(input.as_slice());
    let hash = hasher.result();
    u64::from_le_bytes(*array_ref!(hash.as_ref(), 0, 8))
}