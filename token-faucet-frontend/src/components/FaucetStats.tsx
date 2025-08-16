import React, { useEffect, useState } from 'react';
import { useWallet, useConnection } from '@solana/wallet-adapter-react';
import { FaucetService, type FaucetConfig } from '../services/faucet.service';
import toast from 'react-hot-toast';

const FaucetStats: React.FC = () => {
  const { publicKey } = useWallet();
  const { connection } = useConnection();

  const [faucetConfig, setFaucetConfig] = useState<FaucetConfig | null>(null);
  const [loading, setLoading] = useState<boolean>(false);

  useEffect(() => {
    if (!publicKey) {
      setFaucetConfig(null);
      return;
    }

    const faucetService = new FaucetService(connection, { publicKey } as any);

    async function fetchFaucetConfig() {
      setLoading(true);
      try {
        const config = await faucetService.getFaucetConfig();
        setFaucetConfig(config);
      } catch (err) {
        toast.error('Failed to fetch faucet config');
        console.log(err);
      } finally {
        setLoading(false);
      }
    }
    fetchFaucetConfig();
  }, [connection, publicKey]);

  console.log(publicKey, 'publicKey in FaucetStats');

  if (!publicKey) {
    return <p className='text-gray-400'>Connect wallet to view faucet stats</p>;
  }

  if (loading) {
    return <p className='text-gray-400'>Loading faucet configuration...</p>;
  }

  if (!faucetConfig) {
    return <p className='text-red-500'>Faucet not initialized yet.</p>;
  }

  return (
    <div className='p-6 bg-white/10 backdrop-blur-lg rounded-2xl border border-white/20'>
      <h2 className='text-2xl font-bold text-white mb-4'>
        🎯 Faucet Statistics
      </h2>

      <div className='space-y-3 text-gray-200'>
        <p>
          <span className='font-semibold'>Tokens per claim:</span>{' '}
          {(Number(faucetConfig.tokens_per_claim) / 1_000_000).toLocaleString()}
        </p>
        <p>
          <span className='font-semibold'>Cooldown:</span>{' '}
          {Number(faucetConfig.cooldown_seconds)} seconds
        </p>
        <p>
          <span className='font-semibold'>Status:</span>{' '}
          <span
            className={
              faucetConfig.is_active === true
                ? 'text-green-400'
                : 'text-red-400'
            }
          >
            {faucetConfig.is_active === true ? 'Active' : 'Inactive'}
          </span>
        </p>
      </div>
    </div>
  );
};

export { FaucetStats };
