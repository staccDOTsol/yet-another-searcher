//! Instruction types

use std::mem::size_of;

use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::{Pubkey, PUBKEY_BYTES},
    sysvar,
};

/// Instructions supported by the Flash Loan program.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FlashLoanInstruction {
    // 5
    /// Make a CPI style flash loan.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` Source liquidity token account. One from Reserve.liquidity.supply_pubkey
    ///                     Minted by reserve liquidity mint.
    ///                     Must match the reserve liquidity supply.
    ///   1. `[writable]` Destination liquidity token account.
    ///                     Minted by reserve liquidity mint.
    ///   2. `[writable]` Reserve account.
    ///   3. `[writable]` Flash loan fee receiver account.
    ///                     Must match the reserve liquidity fee receiver.
    ///   4. `[]` Lending market account.
    ///   5. `[]` Derived lending market authority.
    ///   6. `[]` Token program id.
    ///   7. `[]` Flash loan receiver program id.
    ///             Must implement an instruction that has tag of 0 and a signature of `(amount: u64)`
    ///             This instruction must return the amount to the source liquidity account.
    ///   .. `[any]` Additional accounts expected by the receiving program's `ReceiveFlashLoan` instruction.
    ///
    ///   The flash loan receiver program that is to be invoked should contain an instruction with
    ///   tag `0` and accept the total amount (including fee) that needs to be returned back after
    ///   its execution has completed.
    ///
    ///   Flash loan receiver should have an instruction with the following signature:
    ///
    ///   0. `[writable]` Source liquidity (matching the destination from above).
    ///   1. `[writable]` Destination liquidity (matching the source from above).
    ///   2. `[]` Token program id
    ///   .. `[any]` Additional accounts provided to the lending program's `FlashLoan` instruction above.
    ///   ReceiveFlashLoan {
    ///       // Amount that must be repaid by the receiver program
    ///       amount: u64
    ///   }
    FlashLoan {
        /// The amount that is to be borrowed - u64::MAX for up to 100% of available liquidity
        amount: u64,

        /// Instruction tag in loan receiving program to be called
        receive_flash_loan_instruction_tag: u8,
    },

    // 7
    /// Flash borrow reserve liquidity
    //
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` Source liquidity token account. One from Reserve.liquidity.supply_pubkey
    ///   1. `[writable]` Destination liquidity token account. This is user's account to receive borrowed tokens.
    ///   2. `[writable]` Reserve account.
    ///   3. `[]` Lending market account.
    ///   4. `[]` Derived lending market authority.
    ///   5. `[]` Instructions sysvar.
    ///   6. `[]` Token program.
    FlashBorrow {
        /// Amount of liquidity to flash borrow
        amount: u64,
    },

    // 8
    /// Flash repay reserve liquidity
    //
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` Source liquidity token account. This is user's account to repay from.
    ///                     $authority can transfer $amount.
    ///   1. `[writable]` Destination liquidity token account. One from Reserve.liquidity.supply_pubkey
    ///   2. `[writable]` Flash loan fee receiver account.
    ///                     Must match the reserve liquidity fee receiver.
    ///   3. `[writable]` Reserve account.
    ///   4. `[]` Lending market account.
    ///   5. `[signer]` User transfer authority ($authority).
    ///   6. `[]` Instructions sysvar.
    ///   7. `[]` Token program id.
    FlashRepay {
        /// Amount of liquidity to flash repay. Must be the same as in paired FlashBorrow IX.
        amount: u64,
    },
}

impl FlashLoanInstruction {
    /// Packs a [FlashLoanInstruction](enum.FlashLoanInstruction.html) into a byte buffer.
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match *self {
            Self::FlashLoan {
                amount,
                receive_flash_loan_instruction_tag,
            } => {
                buf.push(5);
                buf.extend_from_slice(&amount.to_le_bytes());
                buf.extend_from_slice(&receive_flash_loan_instruction_tag.to_le_bytes());
            }
            Self::FlashBorrow { amount } => {
                buf.push(7);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            Self::FlashRepay { amount } => {
                buf.push(8);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
        }
        buf
    }
}

/// Creates a `FlashLoan` instruction.
#[allow(clippy::too_many_arguments)]
pub fn flash_loan(
    program_id: Pubkey,
    amount: u64,
    receive_flash_loan_instruction_tag: u8,
    source_liquidity_pubkey: Pubkey,
    destination_liquidity_pubkey: Pubkey,
    reserve_pubkey: Pubkey,
    reserve_liquidity_fee_receiver_pubkey: Pubkey,
    lending_market_pubkey: Pubkey,
    flash_loan_receiver_program_id: Pubkey,
    flash_loan_receiver_program_accounts: Vec<AccountMeta>,
) -> Instruction {
    let (lending_market_authority_pubkey, _bump_seed) = Pubkey::find_program_address(
        &[&lending_market_pubkey.to_bytes()[..PUBKEY_BYTES]],
        &program_id,
    );
    let mut accounts = vec![
        AccountMeta::new(source_liquidity_pubkey, false),
        AccountMeta::new(destination_liquidity_pubkey, false),
        AccountMeta::new(reserve_pubkey, false),
        AccountMeta::new(reserve_liquidity_fee_receiver_pubkey, false),
        AccountMeta::new_readonly(lending_market_pubkey, false),
        AccountMeta::new_readonly(lending_market_authority_pubkey, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(flash_loan_receiver_program_id, false),
    ];
    accounts.extend(flash_loan_receiver_program_accounts);
    Instruction {
        program_id,
        accounts,
        data: FlashLoanInstruction::FlashLoan {
            amount,
            receive_flash_loan_instruction_tag,
        }
        .pack(),
    }
}

/// Creates a 'FlashBorrow' instruction.
pub fn flash_borrow(
    program_id: Pubkey,
    amount: u64,
    source_liquidity_pubkey: Pubkey,
    destination_liquidity_pubkey: Pubkey,
    reserve_pubkey: Pubkey,
    lending_market_pubkey: Pubkey,
) -> Instruction {
    let (lending_market_authority_pubkey, _bump_seed) = Pubkey::find_program_address(
        &[&lending_market_pubkey.to_bytes()[..PUBKEY_BYTES]],
        &program_id,
    );

    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(source_liquidity_pubkey, false),
            AccountMeta::new(destination_liquidity_pubkey, false),
            AccountMeta::new(reserve_pubkey, false),
            AccountMeta::new_readonly(lending_market_pubkey, false),
            AccountMeta::new_readonly(lending_market_authority_pubkey, false),
            AccountMeta::new_readonly(sysvar::instructions::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: FlashLoanInstruction::FlashBorrow { amount }.pack(),
    }
}

/// Creates a 'FlashRepay' instruction.
#[allow(clippy::too_many_arguments)]
pub fn flash_repay(
    program_id: Pubkey,
    amount: u64,
    source_liquidity_pubkey: Pubkey,
    destination_liquidity_pubkey: Pubkey,
    reserve_liquidity_fee_receiver_pubkey: Pubkey,
    reserve_pubkey: Pubkey,
    lending_market_pubkey: Pubkey,
    user_transfer_authority_pubkey: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(source_liquidity_pubkey, false),
            AccountMeta::new(destination_liquidity_pubkey, false),
            AccountMeta::new(reserve_liquidity_fee_receiver_pubkey, false),
            AccountMeta::new(reserve_pubkey, false),
            AccountMeta::new_readonly(lending_market_pubkey, false),
            AccountMeta::new_readonly(user_transfer_authority_pubkey, true),
            AccountMeta::new_readonly(sysvar::instructions::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: FlashLoanInstruction::FlashRepay { amount }.pack(),
    }
}
