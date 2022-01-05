use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};



use spl_token::state::Account as TokenAccount;

use crate::{error::AirdropError, instruction::AirdropInstruction, state::Vault, state::User};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = AirdropInstruction::unpack(instruction_data)?;

        match instruction {
            AirdropInstruction::InitAirdrop{ spending_multiplier, txns_multiplier } => {

                Self::process_init_airdrop(accounts,spending_multiplier, txns_multiplier , program_id)
            }
            AirdropInstruction::DisableAirdrop { amount } => {

                Self::disable_airdrop(accounts, amount, program_id)
            }
            AirdropInstruction::CreateAccount { amount_spent } => {

                Self::process_create_account(accounts, amount_spent, program_id)
            }
            AirdropInstruction::DeliverAirdrop { amount_spent, total_transactions } => {

                Self::process_airdrop(accounts, amount_spent, total_transactions, program_id)
            }
        }
    }

    fn process_init_airdrop(
        accounts: &[AccountInfo],
        spending_multiplier: u64,
        txns_multiplier: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let temp_token_account = next_account_info(account_info_iter)?;

        let vault_account = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        if !rent.is_exempt(vault_account.lamports(), vault_account.data_len()) {
            return Err(AirdropError::NotRentExempt.into());
        }

        let mut vault_info = Vault::unpack_unchecked(&vault_account.try_borrow_data()?)?;

        if vault_info.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        Vault::pack(vault_info, &mut vault_account.try_borrow_mut_data()?)?;

        let (pda, _nonce) = Pubkey::find_program_address(&[b""], program_id);

        let token_program = next_account_info(account_info_iter)?;
        let owner_change_ix = spl_token::instruction::set_authority(
            token_program.key,
            temp_token_account.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            initializer.key,
            &[&initializer.key],
        )?;

        msg!("Calling the token program to transfer token account ownership...");
        invoke(
            &owner_change_ix,
            &[
                temp_token_account.clone(),
                initializer.clone(),
                token_program.clone(),
            ],
        )?;

        Ok(())
    }

    fn disable_airdrop(
        accounts: &[AccountInfo],
        amount_expected_by_taker: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let creator = next_account_info(account_info_iter)?;
        
        let creators_token_account = next_account_info(account_info_iter)?;

        if !creator.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let pdas_temp_token_account = next_account_info(account_info_iter)?;
        let pdas_temp_token_account_info =
            TokenAccount::unpack(&pdas_temp_token_account.try_borrow_data()?)?;

        let (pda, nonce) = Pubkey::find_program_address(&[b""], program_id);

        /*
        if amount_expected_by_taker > pdas_temp_token_account_info.amount {
            return Err(AirdropError::ExpectedAmountMismatch.into());
        } */

        let vault_account = next_account_info(account_info_iter)?;

        let mut vault_info = Vault::unpack_unchecked(&vault_account.try_borrow_data()?)?;

        if vault_info.temp_token_account_pubkey != *pdas_temp_token_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        if vault_info.initializer_pubkey != *creator.key {
            return Err(ProgramError::InvalidAccountData);
        }

        if !vault_info.is_initialized() {
            return Err(AirdropError::AccountNotInit.into());
        }

        vault_info.is_initialized = false;


        let token_program = next_account_info(account_info_iter)?;

        if token_program.key != &spl_token::id() {
            return Err(ProgramError::InvalidAccountData);
        }
        

        let pda_account = next_account_info(account_info_iter)?;

        if *creator.key == vault_info.initializer_pubkey {
            msg!("Cancelling program");
            let transfer_to_taker_ix = spl_token::instruction::transfer(
                token_program.key,
                pdas_temp_token_account.key,
                creators_token_account.key,
                &pda,
                &[&pda],
                pdas_temp_token_account_info.amount,
            )?;
            msg!("Calling the token program to transfer tokens to the taker...");
            invoke_signed(
                &transfer_to_taker_ix,
                &[
                    pdas_temp_token_account.clone(),
                    creators_token_account.clone(),
                    pda_account.clone(),
                    token_program.clone(),
                ],
                &[&[&b""[..], &[nonce]]],
            )?; 
            Vault::pack(vault_info, &mut vault_account.try_borrow_mut_data()?)?;
        } else {
            return Err(AirdropError::NotCreator.into());
        }

        Ok(())
    }

    fn process_create_account(
        accounts: &[AccountInfo],
        amount_spent: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let User = next_account_info(account_info_iter)?;

        if !User.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let airdrop_account = next_account_info(account_info_iter)?;

        let vault_account = next_account_info(account_info_iter)?;

        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        if !rent.is_exempt(airdrop_account.lamports(), airdrop_account.data_len()) {
            return Err(AirdropError::NotRentExempt.into());
        }

        let mut vault_info = User::unpack_unchecked(&airdrop_account.try_borrow_data()?)?;
        if vault_info.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        let vault_vault = Vault::unpack_unchecked(&vault_account.try_borrow_data()?)?;
        if !vault_vault.is_initialized() {
            return Err(AirdropError::AccountNotInit.into());
        }

        User::pack(vault_info, &mut airdrop_account.try_borrow_mut_data()?)?;

        Ok(())
    }

    fn process_airdrop(
        accounts: &[AccountInfo],
        amount_spent: u64,
        total_transactions: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {

        let amount_spent_string = amount_spent.to_string();
        let total_transactions_string = total_transactions.to_string();
        msg!(&amount_spent_string);
        msg!(&total_transactions_string);

        let account_info_iter = &mut accounts.iter();
        let User = next_account_info(account_info_iter)?;

        if !User.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let User_token_acc = next_account_info(account_info_iter)?;
        let airdrop_account = next_account_info(account_info_iter)?;
        let vault_account = next_account_info(account_info_iter)?;
        //Get the vault vault
        let vault_vault = Vault::unpack_unchecked(&vault_account.try_borrow_data()?)?;
        if !vault_vault.is_initialized() {
            return Err(AirdropError::AccountNotInit.into());
        }

        //airdrop Acc 
        let mut airdrop_info = User::unpack(&airdrop_account.try_borrow_data()?)?;
        if airdrop_info.last_withdraw != 0 {
            msg!("User has already collected their airdrop");
            return Err(AirdropError::UserAlreadyCollected.into());
        }

        //vault Acc token holdings
        let pdas_temp_token_account = next_account_info(account_info_iter)?;
        let pdas_temp_token_account_info =
            TokenAccount::unpack(&pdas_temp_token_account.try_borrow_data()?)?;

        let (pda, _nonce) = Pubkey::find_program_address(&[b""], program_id);

        let airdrop_main_account = next_account_info(account_info_iter)?;
        // ERROR Checking
        let token_program = next_account_info(account_info_iter)?;

        if token_program.key != &spl_token::id() {
            return Err(ProgramError::InvalidAccountData);
        }

        let pda_account = next_account_info(account_info_iter)?;

        let mut calculations = amount_spent * vault_vault.spending_multiplier;
        calculations += total_transactions * vault_vault.txns_multiplier;
        calculations = calculations * 100000000;

        if calculations <= 0 {
            msg!("No tokens to collect");
            return Err(ProgramError::InvalidAccountData);
        }

        let transfer_to_taker_ix = spl_token::instruction::transfer(
            token_program.key,
            pdas_temp_token_account.key,
            User_token_acc.key,
            &pda,
            &[&pda],
            calculations,
        )?;
        invoke_signed(
            &transfer_to_taker_ix,
            &[
                pdas_temp_token_account.clone(),
                User_token_acc.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b""[..], &[_nonce]]],
        )?; 

        airdrop_info.last_withdraw = 1;
        User::pack(airdrop_info, &mut airdrop_account.try_borrow_mut_data()?)?;

        Ok(())
    }

}
