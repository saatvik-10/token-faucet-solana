import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import App from './App.tsx';
import './App.css';
import WalletContextProvider from './context/WalletContextProvider.tsx';
import { Toaster } from 'react-hot-toast';

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <WalletContextProvider>
      <Toaster />
      <App />
    </WalletContextProvider>
  </StrictMode>
);
