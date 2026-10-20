'use client';

import { useMemo, type ReactNode } from 'react';
import { ConnectionProvider, WalletProvider } from '@solana/wallet-adapter-react';
import { WalletModalProvider } from '@solana/wallet-adapter-react-ui';
import { PhantomWalletAdapter } from '@solana/wallet-adapter-phantom';
import { SolflareWalletAdapter } from '@solana/wallet-adapter-solflare';
import { clusterApiUrl } from '@solana/web3.js';

export function Providers({ children }: { children: ReactNode }) {
  if (process.env.NEXT_PUBLIC_CLUSTER && process.env.NEXT_PUBLIC_CLUSTER !== 'devnet') {
    throw new Error('ENTRY Gate is restricted to Solana Devnet.');
  }
  const endpoint = process.env.NEXT_PUBLIC_RPC_URL || clusterApiUrl('devnet');
  const wallets = useMemo(
    () => [new PhantomWalletAdapter(), new SolflareWalletAdapter()],
    [],
  );

  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={wallets} autoConnect={false}>
        <WalletModalProvider>{children}</WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
}
