use solana_program::program_error::ProgramError;
use std::convert::TryInto;

use crate::error::AirdropError::InvalidData;

pub enum AirdropInstruction {
    /// Despoits tokens that will be distributed to the stakers
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person initializing the Vault
    /// 2. `[writable]` Temporary token account that should be created prior to this instruction and owned by the initializer
    /// 3. `[]` The initializer's token account for the token they will receive should the trade go through
    /// 4. `[writable]` The Vault account, it will hold all necessary info about the trade.
    /// 5. `[]` The rent sysvar
    /// 6. `[]` The token program
    InitAirdrop {
        /// The amount party A expects to receive of token Y
        spending_multiplier: u64,
        txns_multiplier: u64
    },
        /// Returns all deposited token to the creator
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person initializing the Vault
    /// 1.  '[]' Creators token account
    /// 2. `[writable]` Temporary token account that should be created prior to this instruction and owned by the initializer
    /// 3. `[writable]` The Vault account, it will hold all necessary info about the trade
    /// 4. `[]` The token program
    /// 5. `[]` The PDA account
    DisableAirdrop {
        /// Not relevant
        amount: u64,
    },
            /// User withdraws the total collected tokens
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person Vault
    /// 1. '[writable]' The users account, used to determine if they have been given airdrop or not
    /// 2. `[writable]` The Vault account, it will hold all necessary info about the withdraw. -- ESCROW ACC
    /// 3. `[writable]` The initializer's main account to send their rent fees to
    /// 4. `[]` The token program
    /// 5. '[]' PDA Account
    CreateAccount {
        //Collection amount
        amount_spent: u64,
    },
        /// User withdraws the total collected tokens
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person Vault
    /// 1. '[writable]' token account of User to recieve tokens into
    /// 2. '[writable]' The users account, used to determine if they have been given airdrop or not
    /// 3. `[writable]` The Vault account, it will hold all necessary info about the withdraw. -- ESCROW ACC
    /// 4. `[writable]` The PDA's temp token account to get tokens from and eventually close
    /// 5. `[writable]` The initializer's main account to send their rent fees to
    /// 6. `[]` The token program
    /// 7. '[]' PDA Account
    DeliverAirdrop {
        //Collection amount
        amount_spent: u64,
        total_transactions: u64
    },

}

impl AirdropInstruction {
    /// Unpacks a byte buffer into a [AirdropInstruction](enum.AirdropInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidData)?;

        Ok(match tag {
            0 => {
                let spending_multiplier = rest
                    .get(..8)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .unwrap();
                let txns_multiplier = rest
                    .get(8..16)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(InvalidData)?;
                    Self::InitAirdrop {
                        spending_multiplier,
                        txns_multiplier
                    }
            }
            1 => Self::DisableAirdrop {
                amount: Self::unpack_amount(rest)?,
            },
            2 => Self::CreateAccount {
                amount_spent: Self::unpack_amount(rest)?,
            },
            3 => {
                let amount_spent = rest
                .get(..8)
                .and_then(|slice| slice.try_into().ok())
                .map(u64::from_le_bytes)
                .unwrap();
            let total_transactions = rest
                .get(8..16)
                .and_then(|slice| slice.try_into().ok())
                .map(u64::from_le_bytes)
                .ok_or(InvalidData)?;
                Self::DeliverAirdrop {
                    amount_spent,
                    total_transactions
                }

            }
            _ => return Err(InvalidData.into()),
        })
    }

    fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
        let amount = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidData)?;
        Ok(amount)
    }
}
