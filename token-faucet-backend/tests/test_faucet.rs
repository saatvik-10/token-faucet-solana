use borsh::BorshSerialize;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};
use solana_program_test::*;
use solana_sdk::{
    //provides transaction building tools
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

#[tokio::test] //handles async/await
async fn test_initialize_faucet() {
    //creating the test env
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        //mini Solana cluster just for testing so no need for solana-test-validator
        "token_faucet_backend",
        program_id,
        processor!(token_faucet_backend::process_instruction),
    );

    //creating a test token mint
    let mint_keypair = Keypair::new(); //for the tokens the faucet will distribute
    let admin_keypair = Keypair::new(); //admin who will initialize the faucet

    //funding the admin account to pay for transactions
    let admin_account = Account {
        lamports: 1_000_000_000,
        data: vec![],
        owner: system_program::id(),
        executable: false, //not a program
        rent_epoch: 0,
    };

    //adding admin account to test environment (b4 starting)
    program_test.add_account(admin_keypair.pubkey(), admin_account);

    //starting the test env
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    println!("Starting test blockchin via admin account");
    println!("Admin address: {}", admin_keypair.pubkey());
    println!("Token mint address: {}", mint_keypair.pubkey());
}
