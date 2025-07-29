use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    clock::Clock,
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

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
            todo!("Initialize Faucet")
        }
        FaucetInstruction::ClaimTokens => {
            todo!("Claim Tokens");
        }
    }
    Ok(())
}
