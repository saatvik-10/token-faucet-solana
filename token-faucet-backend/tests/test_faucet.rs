use borsh::BorshSerialize;
use solana_program::{
    example_mocks::solana_sdk::system_instruction,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};
use solana_program_test::*;
use solana_sdk::{
    //provides transaction building tools
    account::Account,
    program_pack::Pack,
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

    //creating the SPL token mint

    //calculating how much SOL we need for the mint account
    let mint_rent = banks_client.get_rent().await.unwrap();
    let mint_lamports = mint_rent.minimum_balance(Mint::LEN);

    //creating the mint account i.e. allocating the space in blockchain
    let create_mint_ix = system_instruction::create_account(
        &payer.pubkey(),        //paying for the account
        &mint_keypair.pubkey(), //address of the new account
        mint_lamports,          //SOL for rent exemption
        Mint::LEN as u64,       //space to allocate
        &spl_token::id(),       //SPL token program -> who owns the account
    );

    //initializing the mint
    let init_mint_ix = initialize_mint(
        &spl_token::id(),        //SPL
        &mint_keypair.pubkey(),  //mint that I just created
        &admin_keypair.pubkey(), //mint authority who can mint new tokens
        None,                    //users can't be frozen
        6,                       //USDC like decimals
    )
    .unwrap();

    //sending both instructions inside one transaction
    let mut transaction = Transaction::new_with_payer(
        &[create_mint_ix, init_mint_ix],
        Some(&payer.pubkey()), //one who will pay the fees
    );
    transaction.sign(&[&payer, &mint_keypair], recent_blockhash);

    //execting the transaction
    let res = banks_client.process_transaction(transaction).await;
    assert!(res.is_ok(), "Failed to create token mint: {:?}", res);

    println!("Token mint has been created successfully");
    println!("Mint authority is: {}", admin_keypair.pubkey());
    println!("Decimals: 6");
}
