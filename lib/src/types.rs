//! Types of on-chain objects

use bytemuck::{try_from_bytes, try_from_bytes_mut, Pod, Zeroable};
use solana_program::clock::Slot;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::{IsInitialized, Pack, Sealed};
use solana_program::pubkey::Pubkey;

/// Lending market reserve state
#[derive(Clone, Debug, Default, PartialEq, Eq, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Reserve {
    /// Version of the struct
    pub version: u8,
    _padding: [u8; 7],

    /// Last slot when liquidity supply and\or LP tokens supply updated
    pub last_update: Slot,
    /// Lending market address
    pub lending_market: Pubkey,
    /// Reserve liquidity
    pub liquidity: ReserveLiquidity,
    /// Reserve LP
    pub lp_tokens_info: ReserveLpTokens,
    /// Reserve configuration values
    pub config: ReserveConfig,

    pub _future_padding: [u64; 5],
}

/// Reserve liquidity
#[derive(Clone, Debug, Default, PartialEq, Eq, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct ReserveLiquidity {
    /// Reserve liquidity mint address
    pub mint_pubkey: Pubkey,
    /// Reserve liquidity mint decimals
    pub mint_decimals: u64,
    /// Reserve liquidity supply address
    pub supply_pubkey: Pubkey,
    /// Reserve liquidity available
    pub available_amount: u64,
}

/// Reserve Liquidity Provider (LP) tokens info.
#[derive(Clone, Debug, Default, PartialEq, Eq, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct ReserveLpTokens {
    /// Reserve LP mint address
    pub mint_pubkey: Pubkey,
    /// Reserve LP mint supply, used for exchange rate
    pub mint_total_supply: u64,
    /// Reserve LP supply address
    pub supply_pubkey: Pubkey,
}

/// Reserve configuration values
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct ReserveConfig {
    /// Fees for providing flash loan
    pub fees: ReserveFees,
    /// Maximum deposit limit of liquidity in native units, u64::MAX for inf
    pub deposit_limit: u64,
    /// Flash loan fee receiver address (usually Texture treasury wallet)
    pub fee_receiver: Pubkey,

    // Space for future expansion
    pub _future_padding1: [u8; 32],
    pub _future_padding2: [u8; 32],
}

/// Fee information on a reserve
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct ReserveFees {
    /// Total fee for flash loan, expressed as a Wad.
    /// 0.3% = 3_000_000_000_000_000
    pub flash_loan_fee_wad: u64,
    /// Expressed in % from flash_loan_fee_wad.
    /// Amount of flash loan fee going to Texture. Rest of flash_loan_fee_wad goes to reserve's liquidity pool
    /// 1% = 1
    pub texture_fee_percentage: u8,
    pub _padding: [u8; 7],
}

const RESERVE_LEN: usize = std::mem::size_of::<Reserve>();

impl Sealed for Reserve {}
impl IsInitialized for Reserve {
    fn is_initialized(&self) -> bool {
        self.version != 0
    }
}

impl Pack for Reserve {
    const LEN: usize = RESERVE_LEN;

    /// Packs a byte buffer into a Reserve.
    fn pack_into_slice(&self, output: &mut [u8]) {
        let reserve =
            try_from_bytes_mut::<Reserve>(output).expect("Failed to pack Reserve in to slice");

        *reserve = *self;
    }

    /// Unpacks a byte buffer into a Reserve.
    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
        let reserve =
            try_from_bytes::<Reserve>(input).map_err(|_| ProgramError::InvalidAccountData)?;

        Ok(*reserve)
    }
}
