use borsh::BorshSerialize;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    signature::{Keypair, Signer},
    sysvar::recent_blockhashes,
    transaction::Transaction,
};
use spl_token::{
    instruction::{initialize_mint, mint_to},
    state::{Account as TokenAccount, Mint},
};
use token_faucet_backend::{FaucetConfig, FaucetInstruction, UserClaimedRecord};

#[tokio::test]
async fn test_initialize_faucet() {
    //creating the test env
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "token_faucet_backend",
        program_id,
        processor!(token_faucet_backend::process_instruction),
    );

    //starting the test env
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    //creating a test token mint
    let mint_keypair = Keypair::new();
    let admin_keypair = Keypair::new();

    println!("Ready to test faucet initialization!")
}
