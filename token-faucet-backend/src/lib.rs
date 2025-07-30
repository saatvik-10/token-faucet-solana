use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    // program_pack::Pack,
    pubkey::Pubkey,
    sysvar::{Sysvar, rent::Rent},
};
use spl_token::pack::Pack;
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
            let mint_data = Mint::unpack(&token_mint_account.data.borrow())?;
            msg!("Token mint validated. Supply {}", mint_data.supply)
        }
        FaucetInstruction::ClaimTokens => {
            todo!("Claim Tokens");
        }
    }
    Ok(())
}
