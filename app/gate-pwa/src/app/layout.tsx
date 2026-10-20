import './globals.css';
import '@solana/wallet-adapter-react-ui/styles.css';
import type { ReactNode } from 'react';
import { Providers } from './providers';

export const metadata = { title: 'ENTRY Gate', description: 'Verify and redeem event tickets.' };
export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en">
      <body>
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
