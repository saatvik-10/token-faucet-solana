import {
  Connection,
  PublicKey,
  Transaction,
  TransactionInstruction,
  SystemProgram,
} from '@solana/web3.js';
import type { WalletContextState } from '@solana/wallet-adapter-react';
// import * as borsh from 'borsh';
import * as borsh from '@coral-xyz/borsh'; //raw blockchain data -> readable js
import { toast } from 'react-hot-toast';
import { Buffer } from 'buffer';

const PROGRAM_ID = new PublicKey(import.meta.env.VITE_PROGRAM_ID || '');

//matching rust config
export class FaucetConfig {
  admin!: Uint8Array;
  token_mint!: Uint8Array;
  tokens_per_claim!: bigint;
  cooldown_seconds!: bigint;
  is_active!: boolean;

  constructor(field: FaucetConfig) {
    Object.assign(this, field);
  }
}

const faucetConfigSchema = borsh.struct([
  borsh.array(borsh.u8(), 32, 'admin'),
  borsh.array(borsh.u8(), 32, 'token_mint'),
  borsh.u64('tokens_per_claim'),
  borsh.i64('cooldown_seconds'),
  borsh.bool('is_active'),
]);

export class FaucetService {
  private connection: Connection;
  private wallet: WalletContextState;

  constructor(connection: Connection, wallet: WalletContextState) {
    this.connection = connection;
    this.wallet = wallet;
  }

  //derive pda -> returns [address, bump_seed]
  getFaucetConfigPDA(): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from('faucet_config', 'utf8')], //same rust seed
      PROGRAM_ID
    );
  }

  async initializeFaucet(
    tokenMint: PublicKey,
    tokensPerClaim: number,
    cooldownSeconds: number
  ): Promise<string> {
    if (!this.wallet.publicKey) {
      throw new Error('Wallet public key not available');
    }

    if (!this.wallet.signTransaction) {
      throw new Error('Wallet does not support transaction signing');
    }

    if (!this.wallet.connected) {
      throw new Error('Wallet is not connected');
    }

    console.log('🚀 Wallet validation passed');
    console.log('Public Key:', this.wallet.publicKey.toString());
    console.log('Connected:', this.wallet.connected);

    console.log('🚀 Initializing faucet with:');
    console.log('Token Mint:', tokenMint.toString());
    console.log('Tokens per claim:', tokensPerClaim);
    console.log('Cooldown:', cooldownSeconds);

    const [faucetConfigPDA] = this.getFaucetConfigPDA();

    //instruction data
    const instructionData = Buffer.alloc(1 + 8 + 8);

    let offset = 0;

    //initializing faucet to byte 0
    instructionData.writeUInt8(0, offset);
    offset += 1;

    //tokens per claim
    instructionData.writeBigUInt64LE(
      BigInt(tokensPerClaim * 1_000_000),
      offset
    );
    offset += 8;

    //cooldown seconds
    instructionData.writeBigInt64LE(BigInt(cooldownSeconds), offset);

    console.log('📦 Instruction data length:', instructionData.length);

    const initTransaction = new TransactionInstruction({
      keys: [
        // Account 0: Admin (signer)
        { pubkey: this.wallet.publicKey, isSigner: true, isWritable: true },

        // Account 1: Faucet config PDA (writable)
        { pubkey: faucetConfigPDA, isSigner: false, isWritable: true },

        // Account 2: Token mint account
        { pubkey: tokenMint, isSigner: false, isWritable: false },

        // Account 3: System program (for PDA creation)
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      programId: PROGRAM_ID,
      data: instructionData,
    });

    console.log('Instruction Created!');

    //creating and sending transaction
    const transaction = new Transaction().add(initTransaction);

    // get recent blockhash before signing
    console.log('Getting recent blockhash...');
    const { blockhash } = await this.connection.getLatestBlockhash('confirmed');
    transaction.recentBlockhash = blockhash;

    // set fee payer
    transaction.feePayer = this.wallet.publicKey;

    console.log('Signing transaction...');
    const signedTransaction = await this.wallet.signTransaction(transaction);

    const txid = await this.connection.sendRawTransaction(
      signedTransaction.serialize(),
      {
        skipPreflight: false,
        preflightCommitment: 'processed',
      }
    );
    await this.connection.confirmTransaction(txid);

    toast.success('Faucet initialized successfully!');

    console.log('🎉 Faucet initialized! Signature:', txid);
    return txid;
  }

  //get faucet config from the blockchain
  async getFaucetConfig(): Promise<FaucetConfig | null> {
    try {
      //pda address
      const [faucetConfigPDA] = this.getFaucetConfigPDA();

      //get account data
      const accountInfo = await this.connection.getAccountInfo(faucetConfigPDA);
      console.log(accountInfo);

      if (!accountInfo) {
        return null;
      }

      const decoded = faucetConfigSchema.decode(accountInfo.data);
      const config = new FaucetConfig(decoded);

      toast.success('Faucet config loaded successfully!');

      console.log('✅ Faucet config loaded:', {
        tokensPerClaim: config.tokens_per_claim.toString(),
        cooldownSeconds: config.cooldown_seconds.toString(),
        isActive: config.is_active,
      });

      return config;
    } catch (err) {
      toast.error('Failed to load faucet config');
      console.log('Failed to load the faucet', err);
      return null;
    }
  }
}
