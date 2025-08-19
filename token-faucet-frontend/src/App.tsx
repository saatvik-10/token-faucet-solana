import React from 'react';
import { WalletConnection } from './components/WalletConnection';
import { FaucetStats } from './components/FaucetStats';
import { InitializeFaucet } from './components/InitializeFaucet';
import { ClaimTokens } from './components/ClaimTokens';

const App: React.FC = () => {
  return (
    <div className='min-h-screen bg-gradient-to-br from-indigo-900 via-purple-900 to-pink-800'>
      <div className='container mx-auto px-4 py-8 space-y-8'>
        <WalletConnection />

        {/* <InitializeFaucet /> */}

        <FaucetStats />

        <ClaimTokens />
      </div>
    </div>
  );
};

export default App;
