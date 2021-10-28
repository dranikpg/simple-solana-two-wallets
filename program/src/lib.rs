#![cfg(not(feature = "no-entrypoint"))]

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use std::convert::{TryFrom, TryInto};

/// Instruction data for a two-type token transfer
struct InstructionData {
    token_type: u8,
    amount: u64,
}

/// Parse InstructionData from [u8] slice
impl TryFrom<&[u8]> for InstructionData {
    type Error = ProgramError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            [token_type, am64 @ ..] => {
                let fixed_am64 = am64
                    .try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(InstructionData {
                    token_type: *token_type,
                    amount: u64::from_le_bytes(fixed_am64),
                })
            }
            _ => return Err(ProgramError::InvalidInstructionData),
        }
    }
}

entrypoint!(process_instruction);

/// Accepts only a single instruction
/// Takes four accounts owned by the system program:
/// `[write]` pda wallet for token X
/// `[write]` pda wallet for token Y
/// `[write]` user wallet for token X
/// `[write]` user wallet for token Y
pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let acc_iter = &mut accounts.iter();
    let debit_x_info = next_account_info(acc_iter)?;
    let debit_y_info = next_account_info(acc_iter)?;
    let user_x_info = next_account_info(acc_iter)?;
    let user_y_info = next_account_info(acc_iter)?;

    let InstructionData { token_type, amount } = input.try_into()?;

    let product = debit_x_info.lamports() * debit_y_info.lamports();
    let caculate_return_amount = |base, other, other_delta| base - product / (other + other_delta);

    let (token1_sender, token1_recv, token2_sender, token2_recv, return_amount) = match token_type {
        1 => {
            let return_amount =
                caculate_return_amount(debit_y_info.lamports(), debit_x_info.lamports(), amount);
            (
                user_x_info,
                debit_x_info,
                debit_y_info,
                user_y_info,
                return_amount,
            )
        }
        2 => {
            let return_amount =
                caculate_return_amount(debit_x_info.lamports(), debit_y_info.lamports(), amount);
            (
                user_y_info,
                debit_y_info,
                debit_x_info,
                user_x_info,
                return_amount,
            )
        }
        _ => return Err(ProgramError::InvalidInstructionData),
    };

    msg!("transferring {}, returning {}", amount, return_amount);

    **token1_sender.try_borrow_mut_lamports()? -= amount;
    **token1_recv.try_borrow_mut_lamports()? += amount;
    **token2_sender.try_borrow_mut_lamports()? -= return_amount;
    **token2_recv.try_borrow_mut_lamports()? += return_amount;

    Ok(())
}
