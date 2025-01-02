import { Metadata } from 'next';
import { Inter } from 'next/font/google';
import './globals.css';
import { Providers } from './providers';

const inter = Inter({ subsets: ['latin'] });

export const metadata: Metadata = {
    title: 'LotaBots Platform',
    description: 'AI-powered automation platform',
};

export default function RootLayout({
    children,
}: {
    children: React.ReactNode;
}) {
    return (
        <html lang="en" suppressHydrationWarning>
            <body className={inter.className}>
                <Providers>
                    <main className="min-h-screen bg-background text-white">
                        {children}
                    </main>
                </Providers>
            </body>
        </html>
    );
} 