use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum AirdropError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,
    /// Invalid instruction
    #[error("Invalid Data Input")]
    InvalidData,
    /// Not Rent Exempt
    #[error("Not Rent Exempt")]
    NotRentExempt,
    /// Expected Amount Mismatch
    #[error("Expected Amount Mismatch")]
    ExpectedAmountMismatch,
    /// Amount Overflow
    #[error("Amount Overflow")]
    AmountOverflow,
    //Not Creator
    #[error("Not Creator")]
    NotCreator,
    //Airdrop account not init
    #[error("Airdrop account not initilized")]
    AccountNotInit,
    // User already Collected
    #[error("User Already collected Airdrop")]
    UserAlreadyCollected,
}

impl From<AirdropError> for ProgramError {
    fn from(e: AirdropError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
