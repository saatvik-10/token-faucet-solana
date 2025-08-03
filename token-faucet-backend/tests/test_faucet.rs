use borsh::{BorshDeserialize, BorshSerialize};
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
    address_lookup_table::program,
    program_pack::Pack,
    signature::{Keypair, Signer},
    signer::keypair,
    sysvar::recent_blockhashes,
    transaction::Transaction,
    vote::instruction,
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

    //test faucet initialization

    //creating the faucet config PDA (same as the program)
    let faucet_config_seed = b"faucet_config";
    let (faucet_config_pda, _bump) =
        Pubkey::find_program_address(&[faucet_config_seed], &program_id);

    //initializde faucet instruction data
    let initialize_faucet = FaucetInstruction::InitializeFaucet {
        tokens_per_claim: 1000000000,
        cooldown_seconds: 60,
    };

    //instruction with all accounts that are required
    let initialize_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(admin_keypair.pubkey(), true),
            AccountMeta::new(faucet_config_pda, false),
            AccountMeta::new_readonly(mint_keypair.pubkey(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: borsh::to_vec(&initialize_faucet).unwrap(),
    };

    //creating and sending the transaction
    let mut transaction = Transaction::new_with_payer(&[initialize_ix], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &admin_keypair], recent_blockhash);

    //exectuing transactions and verifying if they go through
    let res = banks_client.process_transaction(transaction).await;
    assert!(res.is_ok(), "Failed tp initialize faucet: {:?}", res);

    println!("Transaction to Initialize Faucet succeeded!");

    //verifying is the faucet config was stored correctly
    let config_account = banks_client
        .get_account(faucet_config_pda)
        .await
        .unwrap()
        .unwrap();
    let faucet_config = FaucetConfig::try_from_slice(&config_account.data).unwrap();

    //checking all stored values
    assert_eq!(faucet_config.admin, admin_keypair.pubkey());
    assert_eq!(faucet_config.token_mint, mint_keypair.pubkey());
    assert_eq!(faucet_config.tokens_per_claim, 1000_000_000);
    assert_eq!(faucet_config.cooldown_seconds, 60);
    assert_eq!(faucet_config.is_active, true);

    println!("Faucet Configuration Verified");
    println!("Admin: {}", faucet_config.admin);
    println!("Token Mint: {}", faucet_config.token_mint);
    println!(
        "Tokens per claims: {} ({})",
        faucet_config.tokens_per_claim,
        faucet_config.tokens_per_claim as f64 / 1_000_000.0
    );
    println!("Cooldown: {} seconds", faucet_config.cooldown_seconds);
    println!("Active: {}", faucet_config.is_active);

    println!("\n Testing claiming tokens...");

    //user who wants to claim tokens
    let user_keypair = Keypair::new();

    //user token account
    let user_token_account = Keypair::new();

    println!("User created: {}", user_keypair.pubkey());
    println!("User token account: {}", user_token_account.pubkey());

    //creating user's token account

    //calculating rent for the token account
    let token_account_rent = banks_client.get_rent().await.unwrap();
    let token_account_lamports = token_account_rent.minimum_balance(TokenAccount::LEN);

    //creating user's token account (space allocation)
    let create_user_token_ix = system_instruction::create_account(
        &payer.pubkey(), //pays the account
        &user_token_account.pubkey(),
        token_account_lamports,
        TokenAccount::LEN as u64,
        &spl_token::id(),
    );

    //initializing user's token account
    let init_user_token_account_ix = spl_token::instruction::initialize_account(
        &spl_token::id(),
        &user_token_account.pubkey(),
        &mint_keypair.pubkey(), //type of token this account holds
        &user_keypair.pubkey(),
    )
    .unwrap();

    //sending both in one transaction
    let mut user_token_account_tx = Transaction::new_with_payer(
        &[create_user_token_ix, init_user_token_account_ix],
        Some(&payer.pubkey()),
    );
    user_token_account_tx.sign(&[&payer, &user_token_account], recent_blockhash);

    //executing the transaction
    let res = banks_client
        .process_transaction(user_token_account_tx)
        .await;
    assert!(
        res.is_ok(),
        "Failed to create user tokena account: {:?}",
        res
    );

    println!("User token account created and initialized successfully!");
    println!("Owner: {}", user_keypair.pubkey());
    println!("Token type: {}", mint_keypair.pubkey());

    // faucet treasury account (place for faucet to store the tokens)
    let faucet_treasury_account = Keypair::new();

    let create_faucet_treasury_ix = system_instruction::create_account(
        &payer.pubkey(),
        &faucet_treasury_account.pubkey(),
        token_account_lamports,
        TokenAccount::LEN as u64,
        &spl_token::id(),
    );

    let init_faucet_treasury_ix = spl_token::instruction::initialize_account(
        &spl_token::id(),
        &faucet_treasury_account.pubkey(),
        &mint_keypair.pubkey(),
        &admin_keypair.pubkey(),
    )
    .unwrap();

    let mut treasury_tx = Transaction::new_with_payer(
        &[create_faucet_treasury_ix, init_faucet_treasury_ix],
        Some(&payer.pubkey()),
    );
    treasury_tx.sign(&[&payer, &faucet_treasury_account], recent_blockhash);

    let res = banks_client.process_transaction(treasury_tx).await;
    assert!(res.is_ok(), "Failed to create treasury account: {:?}", res);

    println!("Faucet treasury account created successfully!");
    println!(
        "Treasury account address: {}",
        faucet_treasury_account.pubkey()
    );
    println!("Treasury account owner: {}", admin_keypair.pubkey());

    //minting tokens into the treasury to give them away

    //creating mint_to instruction to put tokens in the treasury
    let mint_to_treasury_ix = mint_to(
        &spl_token::id(),
        &mint_keypair.pubkey(),
        &faucet_treasury_account.pubkey(),
        &admin_keypair.pubkey(),
        &[],
        1000_000_000,
    )
    .unwrap();

    //sending the mint transaction
    let mut mint_tx = Transaction::new_with_payer(&[mint_to_treasury_ix], Some(&payer.pubkey()));
    mint_tx.sign(&[&payer, &admin_keypair], recent_blockhash);

    let res = banks_client.process_transaction(mint_tx).await;
    assert!(
        res.is_ok(),
        "Failed to mint tokens to the treasury: {:?}",
        res
    );

    println!("Token successfully minted to the treasury!");
    println!("Amount: 1,000 Tokens");
    println!("Treasury has enough amount to destribute the tokens");
}
