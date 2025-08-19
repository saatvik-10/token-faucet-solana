import React, { useState } from 'react';
import { useWallet, useConnection } from '@solana/wallet-adapter-react';
import { FaucetService } from '../services/faucet.service';
import toast from 'react-hot-toast';

export const ClaimTokens: React.FC = () => {
  const { publicKey } = useWallet();
  const wallet = useWallet();
  const { connection } = useConnection();
  const [loading, setLoading] = useState<boolean>(false);

  const handleClaim = async () => {
    if (!wallet.publicKey || !wallet.sendTransaction || !wallet.connected) {
      toast.error('Please ensure your wallet is connected');
      return;
    }
    setLoading(true);

    try {
      const faucetService = new FaucetService(connection, wallet);
      const signature = await faucetService.claimToken();

      toast.success(`Tokens claimed! Signature: ${signature.slice(0, 8)}...`);
    } catch (err: any) {
      toast.error('Failed to claim tokens');
      console.log(err);
      if (err.message.includes('cooldown')) {
        toast.error('Please wait for cooldown period to end');
      } else if (err.message.includes('insufficient')) {
        toast.error('Faucet has insufficient tokens');
      } else {
        toast.error(`Failed to claim: ${err.message}`);
      }
    } finally {
      setLoading(false);
    }
  };

  if (!publicKey) {
    return (
      <div className='p-6 bg-white/10 backdrop-blur-lg rounded-2xl border border-white/20'>
        <p className='text-gray-400 text-center'>
          Connect wallet to claim tokens
        </p>
      </div>
    );
  }

  return (
    <div className='p-6 bg-white/10 backdrop-blur-lg rounded-2xl border border-white/20'>
      <h2 className='text-2xl font-bold text-white mb-4'>Claim Tokens</h2>

      <p className='text-gray-300 mb-6'>
        Click to claim your free tokens! Check cooldown period in stats above.
      </p>

      <button
        onClick={handleClaim}
        disabled={loading}
        className='w-full bg-gradient-to-r from-blue-500 to-purple-500 hover:from-blue-600 hover:to-purple-600 disabled:from-gray-500 disabled:to-gray-600 text-white font-semibold py-4 px-6 rounded-xl transition-all duration-300 transform hover:scale-105 disabled:hover:scale-100 text-lg'
      >
        {loading ? 'Claiming Tokens...' : 'Claim Tokens!'}
      </button>
    </div>
  );
};

