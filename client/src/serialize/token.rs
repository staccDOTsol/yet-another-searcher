use anchor_client::solana_sdk::program_error::ProgramError;
use anchor_client::solana_sdk::program_option::COption;
use anchor_client::solana_sdk::pubkey::Pubkey;
use arrayref::{array_refs};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use solana_program::program_pack::Pack;
use spl_token::state::Account;

use std::fmt;
use std::fmt::Debug;
use std::str::FromStr;

use serde;
use serde::{Deserialize, Serialize, Serializer};
use std::ops::{Deref, DerefMut};

// SERDE STUFF (FROM JSON)
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Token {
    pub tag: String,
    pub name: String,
    pub mint: WrappedPubkey,
    pub scale: u64,
    pub addr: WrappedPubkey,
}

#[derive(Deserialize, Serialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct WrappedString(pub String);

#[derive(Deserialize, PartialEq, Eq)]
#[serde(from = "WrappedString")]
pub struct WrappedPubkey(pub Pubkey);

impl Serialize for WrappedPubkey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.to_string().as_str())
    }
}

impl Clone for WrappedPubkey {
    fn clone(&self) -> Self {
        WrappedPubkey(self.0)
    }
}

impl fmt::Debug for WrappedPubkey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Deref for WrappedPubkey {
    type Target = Pubkey;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for WrappedPubkey {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<WrappedString> for WrappedPubkey {
    fn from(s: WrappedString) -> Self {
        let pubkey = Pubkey::from_str(&s.0);
        if pubkey.is_ok(){
            WrappedPubkey(pubkey.unwrap())
        } else {
            WrappedPubkey(Pubkey::default())
        }
    }
}

// ACCOUNT INFO -> TOKEN STUFF (SOLANA)

/// Account state.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, IntoPrimitive, TryFromPrimitive, Default)]
pub enum AccountState {
    /// Account is not yet initialized
    #[default]
    Uninitialized,
    /// Account is initialized; the account owner and/or delegate may perform permitted operations
    /// on this account
    Initialized,
    /// Account has been frozen by the mint freeze authority. Neither the account owner nor
    /// the delegate are able to perform operations on this account.
    Frozen,
}

/// Account data.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TokenAccount {
    /// The mint associated with this account
    pub mint: Pubkey,
    /// The owner of this account.
    pub owner: Pubkey,
    /// The amount of tokens this account holds.
    pub amount: u64,
    /// If `delegate` is `Some` then `delegated_amount` represents
    /// the amount authorized by the delegate
    pub delegate: COption<Pubkey>,
    /// The account's state
    pub state: AccountState,
    /// If is_some, this is a native token, and the value logs the rent-exempt reserve. An Account
    /// is required to be rent-exempt, so the value is used by the Processor to ensure that wrapped
    /// SOL accounts do not drop below this threshold.
    pub is_native: COption<u64>,
    /// The amount delegated
    pub delegated_amount: u64,
    /// Optional authority to close the account.
    pub close_authority: COption<Pubkey>,
}

fn unpack_coption_key(src: &[u8; 36]) -> Result<COption<Pubkey>, ProgramError> {
    let (tag, body) = array_refs![src, 4, 32];
    match *tag {
        [0, 0, 0, 0] => Ok(COption::None),
        [1, 0, 0, 0] => Ok(COption::Some(Pubkey::new_from_array(*body))),
        _ => Err(ProgramError::InvalidAccountData),
    }
}

fn unpack_coption_u64(src: &[u8; 12]) -> Result<COption<u64>, ProgramError> {
    let (tag, body) = array_refs![src, 4, 8];
    match *tag {
        [0, 0, 0, 0] => Ok(COption::None),
        [1, 0, 0, 0] => Ok(COption::Some(u64::from_le_bytes(*body))),
        _ => Err(ProgramError::InvalidAccountData),
    }
}

pub fn unpack_token_account(data: &[u8]) -> Account {
    if data.len() != 165 {
        return Account::default();
    }
    spl_token::state::Account::unpack_from_slice(data).unwrap()
}
