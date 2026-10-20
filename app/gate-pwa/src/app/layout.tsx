import './globals.css'; import type { ReactNode } from 'react';
export const metadata = { title: 'ENTRY Gate', description: 'Verify and redeem event tickets.' };
export default function RootLayout({children}:{children:ReactNode}) { return <html lang="en"><body>{children}</body></html>; }
