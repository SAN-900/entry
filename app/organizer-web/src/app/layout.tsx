import './globals.css';
import type { ReactNode } from 'react';
export const metadata = { title: 'ENTRY Organizer', description: 'Create fair, verifiable event tickets.' };
export default function RootLayout({ children }: { children: ReactNode }) { return <html lang="en"><body>{children}</body></html>; }
