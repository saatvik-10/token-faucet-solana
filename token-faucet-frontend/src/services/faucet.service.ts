import { Connection, PublicKey } from '@solana/web3.js';
import type { WalletContextState } from '@solana/wallet-adapter-react';
// import * as borsh from 'borsh';
import * as borsh from '@coral-xyz/borsh'; //raw blockchain data -> readable js
import { toast } from 'react-hot-toast';

const PROGRAM_ID = new PublicKey(import.meta.env.PROGRAM_ID);

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
      [Uint8Array.from('faucet_config')], //same rust seed
      PROGRAM_ID
    );
  }

  //get faucet config from the blockchain
  async getFaucetConfig(): Promise<FaucetConfig | null> {
    try {
      //pda address
      const [faucetConfigPDA] = this.getFaucetConfigPDA();

      //get account data
      const accountInfo = await this.connection.getAccountInfo(faucetConfigPDA);
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
