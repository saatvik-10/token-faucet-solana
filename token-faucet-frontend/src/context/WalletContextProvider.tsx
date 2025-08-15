import React from 'react';
import {
  ConnectionProvider, //connects app to Solana blockchain
  WalletProvider, //manages wallet state
} from '@solana/wallet-adapter-react';
import { WalletAdapterNetwork } from '@solana/wallet-adapter-base'; //chooses Solana network
import { WalletModalProvider } from '@solana/wallet-adapter-react-ui'; //creates wallet popup window
// import { PhantomWalletAdapter } from '@solana/wallet-adapter-wallets';
import { useMemo } from 'react';
import { clusterApiUrl } from '@solana/web3.js'; //gets the web address of Solana servers
import '@solana/wallet-adapter-react-ui/styles.css';

interface Props {
  children: React.ReactNode;
}

const WalletContextProvider: React.FC<Props> = ({ children }) => {
  const network = WalletAdapterNetwork.Devnet;
  const endpoint = clusterApiUrl(network);

  const wallets = useMemo(() => [], []); //adds support to all wallets

  return (
    <div>
      <ConnectionProvider endpoint={endpoint}>
        <WalletProvider wallets={wallets} autoConnect>
          <WalletModalProvider>{children}</WalletModalProvider>
        </WalletProvider>
      </ConnectionProvider>
    </div>
  );
};

export default WalletContextProvider;
