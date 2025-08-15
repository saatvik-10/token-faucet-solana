use std::io::Cursor;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    clock::Clock,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{Sysvar, rent::Rent},
};
use spl_token::{ID as TOKEN_PROGRAM_ID, instruction::transfer, state::Mint};

//use claimed records stored in PDA
#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct UserClaimedRecord {
    pub user: Pubkey,
    pub last_claim_time: i64,
    pub total_claims: u64,
}

//faucet config
#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct FaucetConfig {
    pub admin: Pubkey,
    pub token_mint: Pubkey, //which token this faucet distributes
    pub tokens_per_claim: u64,
    pub cooldown_seconds: i64,
    pub is_active: bool,
}

//instructions program will accept
#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub enum FaucetInstruction {
    //initialize faucet
    //accounts :
    //signer -> admin account
    //writable -> faucet config account
    //token mint account
    InitializeFaucet {
        tokens_per_claim: u64,
        cooldown_seconds: i64,
    },
    //claims tokens from faucet
    //accounts :
    //signer -> user req tokens
    //writable -> user claim record PDA
    //writable -> user token account
    //writable -> faucet treasury token account
    //faucet config account
    //token program
    ClaimTokens,
    UpdateFaucetConfig {
        new_tokens_per_claim: Option<u64>,
        new_cooldown_seconds: Option<u64>,
        new_is_active: Option<bool>,
    },
    EmergencyPause,
}

#[derive(Debug)]
pub enum FaucetError {
    CooldownNotMet,
    FaucetInactive,
    InsufficientFunds,
    UnauthorizedAdmin,
}

impl From<FaucetError> for ProgramError {
    fn from(e: FaucetError) -> Self {
        match e {
            FaucetError::CooldownNotMet => ProgramError::Custom(1000),
            FaucetError::FaucetInactive => ProgramError::Custom(1001),
            FaucetError::InsufficientFunds => ProgramError::Custom(1002),
            FaucetError::UnauthorizedAdmin => ProgramError::Custom(1003),
        }
    }
}

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = FaucetInstruction::try_from_slice(instruction_data)?;

    match instruction {
        FaucetInstruction::InitializeFaucet {
            tokens_per_claim,
            cooldown_seconds,
        } => {
            msg!(
                "Initializing faucet with {} tokens per claim, {} second cooldown",
                tokens_per_claim,
                cooldown_seconds
            );

            let accounts_iter = &mut accounts.iter();

            //signer (admin account)
            let admin_account = next_account_info(accounts_iter)?;
            if !admin_account.is_signer {
                msg!("Admin account must be a signer");
                return Err(ProgramError::MissingRequiredSignature);
            }

            //faucet config account (PDA creator)
            let faucet_config_account = next_account_info(accounts_iter)?;

            //token mint account (valid SOL token mint)
            let token_mint_account = next_account_info(accounts_iter)?;

            //system program (required to create accounts)
            let system_program = next_account_info(accounts_iter)?;

            //validate that token mint is actually a mint account (account that stores global metadata about a token)
            let mint_data = Mint::unpack(&token_mint_account.data.borrow())
                .map_err(|_| ProgramError::InvalidAccountData)?;
            msg!("Token mint validated. Supply {}", mint_data.supply);

            //create PDA for faucet config
            let faucet_config_seed = b"faucet_config";
            let (faucet_config_pda, bump_seed) =
                Pubkey::find_program_address(&[faucet_config_seed], program_id);

            // Verify the passed account is the correct PDA
            if faucet_config_pda != *faucet_config_account.key {
                msg!("Faucet config account is not the correct PDA");
                return Err(ProgramError::InvalidAccountData);
            }

            //calculate required space for FaucetConfig
            let config_data = FaucetConfig {
                admin: *admin_account.key,
                token_mint: *token_mint_account.key,
                tokens_per_claim,
                cooldown_seconds,
                is_active: true,
            };

            let required_space = borsh::to_vec(&config_data)?.len();

            //lamports for rent exemption
            let rent = Rent::get()?;
            let required_lamports = rent.minimum_balance(required_space);

            //creating the faucet config
            let create_account_instruction = system_instruction::create_account(
                admin_account.key,         //payer
                faucet_config_account.key, //new account
                required_lamports,         //lamports to transfer
                required_space as u64,     //allocate the space
                program_id,                //owner of new account
            );

            //invoke system program to create the new account (CPI)
            invoke_signed(
                &create_account_instruction,
                &[
                    admin_account.clone(),
                    faucet_config_account.clone(),
                    system_program.clone(),
                ],
                &[&[faucet_config_seed, &[bump_seed]]], //PDA sign
            )?;

            //serialize and store the config of faucet
            config_data.serialize(&mut &mut faucet_config_account.data.borrow_mut()[..])?;

            msg!("Faucet initialized successfully!");
            msg!("Admin: {}", admin_account.key);
            msg!("Token Mint: {}", token_mint_account.key);
            msg!("PDA of Faucet: {}", faucet_config_pda);
        }

        FaucetInstruction::ClaimTokens => {
            msg!("Processing claim tokens request");

            //account iterator
            let accounts_iter = &mut accounts.iter();

            //user requesting tokens (must be a signer)
            let user_account = next_account_info(accounts_iter)?;
            if !user_account.is_signer {
                msg!("User account must be a signer");
                return Err(ProgramError::MissingRequiredSignature);
            }

            //user's claim record PDA(will create and update)
            let user_claim_record_account = next_account_info(accounts_iter)?;

            //token account of user (to receive the tokens)
            let user_token_account = next_account_info(accounts_iter)?;

            // faucet treasury account token (tokens come from this)
            let faucet_treasury_account = next_account_info(accounts_iter)?;

            //faucet config account (contain settings)
            let faucet_account_config = next_account_info(accounts_iter)?;

            // token program
            let token_program = next_account_info(accounts_iter)?;

            //system program (needed to create user claim record if first time)
            let system_program = next_account_info(accounts_iter)?;

            //load faucet config
            let faucet_config = FaucetConfig::try_from_slice(&faucet_account_config.data.borrow())?;

            //if faucet active or not
            if !faucet_config.is_active {
                msg!("Faucet is currently inactive");
                return Err(FaucetError::FaucetInactive.into());
            }

            //PDA for user claim record
            let user_claim_seed = b"user_claim";
            let (user_claim_pda, user_bump_seed) = Pubkey::find_program_address(
                &[user_claim_seed, user_account.key.as_ref()], //converts Pubkey to &[u8]
                program_id,
            );

            //verifying the passed account is the correct PDA
            if user_claim_pda != *user_claim_record_account.key {
                msg!("User claim record account is not the correct PDA");
                return Err(ProgramError::InvalidAccountData);
            }

            //create PDA for faucet config
            let faucet_config_seed = b"faucet_config";
            let (faucet_config_pda, faucet_bump_seed) =
                Pubkey::find_program_address(&[faucet_config_seed], program_id);

            //getting the current timestamp
            let clock = Clock::get()?;
            let current_time = clock.unix_timestamp;

            //to check if the user's claim record exist
            let mut user_record = if user_claim_record_account.data_len() == 0 {
                msg!("User is claiming for the first time... Creating a new account!");

                //creating the user claim record account
                let user_record = UserClaimedRecord {
                    user: *user_account.key,
                    last_claim_time: 0, //no prev claim for the first timers
                    total_claims: 0,
                };

                let required_space = borsh::to_vec(&user_record)?.len();
                let rent = Rent::get()?;
                let required_lamports = rent.minimum_balance(required_space);

                let create_account_instruction = system_instruction::create_account(
                    user_account.key,
                    user_claim_record_account.key,
                    required_lamports,
                    required_space as u64,
                    program_id,
                );

                invoke_signed(
                    &create_account_instruction,
                    &[
                        user_account.clone(),
                        user_claim_record_account.clone(),
                        system_program.clone(),
                    ],
                    &[&[
                        user_claim_seed,
                        user_account.key.as_ref(),
                        &[user_bump_seed],
                    ]],
                )?;
                user_record
            } else {
                //load existing user record
                UserClaimedRecord::try_from_slice(&user_claim_record_account.data.borrow())?
            };

            //checking cooldown period
            let time_since_last_claim = current_time - user_record.last_claim_time;

            if time_since_last_claim < faucet_config.cooldown_seconds {
                let remaining_cooldown = faucet_config.cooldown_seconds - time_since_last_claim;
                msg!(
                    "Cooldown period not met! Please wait for {} seconds",
                    remaining_cooldown
                );
                return Err(FaucetError::CooldownNotMet.into());
            }

            msg!(
                "Cooldown check passed! Checking if {} tokens available...",
                faucet_config.tokens_per_claim
            );

            let treasury_data =
                spl_token::state::Account::unpack(&faucet_treasury_account.data.borrow())
                    .map_err(|_| ProgramError::InvalidAccountData)?;

            if treasury_data.amount < faucet_config.tokens_per_claim {
                msg!(
                    "Treasury has {} tokens, but {} tokens requested!",
                    treasury_data.amount,
                    faucet_config.tokens_per_claim
                );
                return Err(FaucetError::InsufficientFunds.into());
            }

            //creating token transfer instruction i.e. CPI
            let transfer_instruction = transfer(
                &TOKEN_PROGRAM_ID,
                faucet_treasury_account.key, //source token account
                user_token_account.key,      //destination token account
                &faucet_config_pda,          //authority (faucet config PDA)
                &[],
                faucet_config.tokens_per_claim, //amount to transfer
            )
            .map_err(|e| {
                msg!("Failed to create transfer instruction: {:?}", e);
                ProgramError::InvalidInstructionData
            })?;

            //execute the token transfer via CPI with PDA signature
            invoke_signed(
                &transfer_instruction,
                &[
                    faucet_treasury_account.clone(),
                    user_token_account.clone(),
                    faucet_account_config.clone(), // Use faucet config PDA as authority
                    token_program.clone(),
                ],
                &[&[faucet_config_seed, &[faucet_bump_seed]]], //PDA signature
            )?;

            //updating user's claim records
            user_record.last_claim_time = current_time;
            user_record.total_claims += 1;

            //saving the updated record
            user_record.serialize(&mut &mut user_claim_record_account.data.borrow_mut()[..])?;

            msg!("Tokens have been transferred successfully!");
            msg!("User: {}", user_account.key);
            msg!("Amount: {}", faucet_config.tokens_per_claim);
            msg!("Total user claims: {}", user_record.total_claims);
        }

        FaucetInstruction::UpdateFaucetConfig {
            new_tokens_per_claim,
            new_cooldown_seconds,
            new_is_active,
        } => {
            msg!("Processing faucet update request!");

            let accounts_iter = &mut accounts.iter();

            let admin_account = next_account_info(accounts_iter)?;
            let faucet_config_account = next_account_info(accounts_iter)?;

            // Validate admin is signer
            if !admin_account.is_signer {
                msg!("Admin must be the signer!");
                return Err(ProgramError::MissingRequiredSignature);
            }

            //loading the current config
            let mut faucet_config =
                FaucetConfig::try_from_slice((&faucet_config_account.data.borrow()))?;

            //checking if the caller is the real admin
            if admin_account.key != &faucet_config.admin {
                msg!("Unauthorized admin access attempt!");
                msg!("Expected admin: {}", faucet_config.admin);
                msg!("Actual caller: {}", admin_account.key);
                return Err(FaucetError::UnauthorizedAdmin.into());
            }

            msg!("Admin account verified!");

            if let Some(tokens) = new_tokens_per_claim {
                faucet_config.tokens_per_claim = tokens;
                msg!("Updated tokens per claim to: {}", tokens);
            }

            if let Some(cooldown) = new_cooldown_seconds {
                faucet_config.cooldown_seconds = cooldown as i64;
                msg!("Updated cooldown to: {} seconds", cooldown);
            }

            if let Some(active) = new_is_active {
                faucet_config.is_active = active;
                msg!(
                    "Updated faucet status to: {}",
                    if active { "ACTIVE" } else { "INACTIVE" }
                );
            }

            faucet_config.serialize(&mut &mut faucet_config_account.data.borrow_mut()[..])?;

            msg!("Faucet Configuration updated successfully!")
        }

        FaucetInstruction::EmergencyPause => {
            msg!("Emergency Pause has been requested!");

            let accounts_iter = &mut accounts.iter();

            let admin_account = next_account_info(accounts_iter)?;
            let faucet_config_account = next_account_info(accounts_iter)?;

            //validating admin signature
            if !admin_account.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }

            //loading config and verifying the admin
            let mut faucet_config =
                FaucetConfig::try_from_slice(&faucet_config_account.data.borrow())?;

            if admin_account.key != &faucet_config.admin {
                msg!(
                    "Unauthorized attempt to pause the faucet from: {}",
                    admin_account.key
                );

                return Err(FaucetError::UnauthorizedAdmin.into());
            }

            faucet_config.is_active = false;
            faucet_config.serialize(&mut &mut faucet_config_account.data.borrow_mut()[..])?;

            msg!("Faucet has been pause by the admin!");
            msg!("Admin: {}", admin_account.key);
            msg!("All tokens have been blocked until the Faucet resumes!");
        }
    }
    Ok(())
}
