import { Metadata } from 'next';
import { Inter } from 'next/font/google';
import './globals.css';

const inter = Inter({ subsets: ['latin'] });

export const metadata: Metadata = {
    title: 'LotaBots - Enterprise AI Agent Platform',
    description: 'Create, deploy, and manage AI-powered conversational agents for your enterprise.',
    keywords: ['AI', 'chatbot', 'enterprise', 'automation', 'machine learning'],
    authors: [{ name: 'LotaBots Team' }],
    viewport: 'width=device-width, initial-scale=1',
    icons: {
        icon: '/favicon.ico',
    },
};

export default function RootLayout({
    children,
}: {
    children: React.ReactNode;
}) {
    return (
        <html lang="en" className="dark">
            <body className={inter.className}>
                {children}
            </body>
        </html>
    );
} 