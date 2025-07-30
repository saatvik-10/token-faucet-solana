use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint,
    entrypoint::ProgramResult,
    example_mocks::solana_sdk::system_instruction,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{Sysvar, rent::Rent},
};
use spl_token::solana_program::program_pack::Pack;
use spl_token::state::Mint;

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
}

entrypoint!(process_instruction);

fn process_instruction(
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
            todo!("Claim Tokens");
        }
    }
    Ok(())
}
