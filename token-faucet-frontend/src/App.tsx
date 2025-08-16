import React from 'react';
import { WalletConnection } from './components/WalletConnection';
import { FaucetStats } from './components/FaucetStats';

const App: React.FC = () => {
  return (
    <div>
      <WalletConnection />

      <div className='mt-8'>
        <FaucetStats />
      </div>
    </div>
  );
};

export default App;
