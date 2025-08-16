import '@solana/wallet-adapter-react-ui/styles.css';

import React from 'react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { useWallet } from '@solana/wallet-adapter-react';

export const WalletConnection: React.FC = () => {
  const { publicKey, connected } = useWallet();

  return (
    <div className='min-h-screen bg-gradient-to-br from-indigo-900 via-purple-900 to-pink-800'>
      {/* Hero Section */}
      <div className='container mx-auto px-4 py-16'>
        <div className='text-center mb-12'>
          <h1 className='text-6xl font-bold text-white mb-4'>
            <span className='bg-gradient-to-r from-cyan-400 to-purple-400 bg-clip-text text-transparent'>
              Solana Faucet
            </span>
          </h1>
          <p className='text-xl text-gray-300 max-w-2xl mx-auto'>
            Get free test tokens for your Solana development projects. <br />
            Connect your wallet to claim your tokens instantly!
          </p>
        </div>

        {/* Wallet Connection Card */}
        <div className='max-w-md mx-auto'>
          <div className='bg-white/10 backdrop-blur-lg rounded-3xl shadow-2xl border border-white/20 p-8'>
            <div className='text-center'>
              <div className='mb-6'>
                <div className='w-20 h-20 bg-gradient-to-br from-cyan-400 to-purple-500 rounded-full mx-auto flex items-center justify-center text-3xl'>
                  👛
                </div>
              </div>

              <h2 className='text-2xl font-bold text-white mb-4'>
                {connected ? 'Wallet Connected' : 'Connect Your Wallet'}
              </h2>

              {/* Wallet Button */}
              <div className='mb-6'>
                <WalletMultiButton className='!bg-gradient-to-r !from-purple-500 !to-pink-500 hover:!from-purple-600 hover:!to-pink-600 !rounded-xl !font-semibold !px-8 !py-4 !text-lg !transition-all !duration-300 !transform hover:!scale-105' />
              </div>

              {/* Connection Status */}
              {connected ? (
                <div className='bg-green-500/20 border border-green-400/50 rounded-xl p-4'>
                  <div className='flex items-center justify-center text-green-400 mb-2'>
                    <svg
                      className='w-6 h-6 mr-2'
                      fill='currentColor'
                      viewBox='0 0 20 20'
                    >
                      <path
                        fillRule='evenodd'
                        d='M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z'
                        clipRule='evenodd'
                      />
                    </svg>
                    <span className='font-semibold'>Wallet Connected!</span>
                  </div>
                  <div className='text-gray-300 text-sm'>
                    <span className='font-mono bg-gray-800/50 px-3 py-1 rounded-lg'>
                      {publicKey?.toString().slice(0, 8)}...
                      {publicKey?.toString().slice(-8)}
                    </span>
                  </div>
                </div>
              ) : (
                <div className='bg-gray-600/20 border border-gray-500/50 rounded-xl p-4'>
                  <div className='flex items-center justify-center text-gray-400 mb-2'>
                    <svg
                      className='w-6 h-6 mr-2'
                      fill='none'
                      stroke='currentColor'
                      viewBox='0 0 24 24'
                    >
                      <path
                        strokeLinecap='round'
                        strokeLinejoin='round'
                        strokeWidth={2}
                        d='M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z'
                      />
                    </svg>
                    <span>No wallet connected</span>
                  </div>
                  <p className='text-gray-400 text-sm'>
                    Click the button above to connect your Phantom, Solflare, or
                    Backpack wallet
                  </p>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
