import React, { useState } from 'react';
import { useWallet, useConnection } from '@solana/wallet-adapter-react';
import { PublicKey } from '@solana/web3.js';
import { FaucetService } from '../services/faucet.service';
import toast from 'react-hot-toast';

export const InitializeFaucet: React.FC = () => {
  const wallet = useWallet();
  const { connection } = useConnection();

  const [loading, setLoading] = useState<boolean>(false);

  const handleInitialize = async () => {
    if (!wallet.publicKey || !wallet.signTransaction || !wallet.connected) {
      toast.error('Please ensure your wallet is fully connected');
      return;
    }

    if (!wallet) {
      toast.error('Please connect your wallet first!');
      return;
    }

    setLoading(true);
    try {
      const faucetService = new FaucetService(connection, wallet);

      const tokenMint = new PublicKey(
        import.meta.env.VITE_TOKEN_MINT_ADDRESS || ''
      );
      const tokensPerClaim = 100;
      const cooldownSeconds = 3600;

      const signature = await faucetService.initializeFaucet(
        tokenMint,
        tokensPerClaim,
        cooldownSeconds
      );

      toast.success(
        `Faucet initialized! Signature: ${signature.slice(0, 8)}...`
      );
    } catch (err: any) {
      console.error('Initialize error:', err);
      toast.error(`Failed to initialize: ${err.message}`);
    } finally {
      setLoading(false);
    }
  };

  if (!wallet) {
    return <p className='text-gray-400'>Connect wallet to initialize faucet</p>;
  }

  return (
    <div className='p-6 bg-white/10 backdrop-blur-lg rounded-2xl border border-white/20'>
      <h2 className='text-2xl font-bold text-white mb-4'>
        🚀 Initialize Faucet
      </h2>

      <p className='text-gray-300 mb-4'>
        This will create the global faucet configuration. Only needs to be done
        once.
      </p>

      <button
        onClick={handleInitialize}
        disabled={loading}
        className='w-full bg-gradient-to-r from-green-500 to-emerald-500 hover:from-green-600 hover:to-emerald-600 disabled:from-gray-500 disabled:to-gray-600 text-white font-semibold py-3 px-6 rounded-xl transition-all duration-300 transform hover:scale-105 disabled:hover:scale-100'
      >
        {loading ? 'Initializing...' : 'Initialize Faucet'}
      </button>
    </div>
  );
};
