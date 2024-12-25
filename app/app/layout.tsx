'use client';

import { GenAIProvider } from '@/lib/genai/GenAIContext';
import '@/lib/telemetry/init';
import { AnimatePresence, motion } from 'framer-motion';
import { Inter } from 'next/font/google';
import localFont from 'next/font/local';
import React, { useEffect } from 'react';
import './globals.css';

const inter = Inter({
    subsets: ['latin'],
    variable: '--font-inter',
    display: 'swap',
});

const monument = localFont({
    src: '../public/fonts/MonumentExtended-Regular.woff2',
    variable: '--font-monument',
    display: 'swap',
});

interface RootLayoutProps {
    children: React.ReactNode;
}

export default function RootLayout({ children }: RootLayoutProps) {
    useEffect(() => {
        // Log when the app is mounted
        console.log('App mounted - Telemetry initialized');
    }, []);

    return (
        <html lang="en" className={`${inter.variable} ${monument.variable}`}>
            <head>
                <title>GenAI Web App</title>
                <meta name="description" content="A Next.js web application with Google's Generative AI integration" />
            </head>
            <body className="bg-primary text-white">
                <GenAIProvider apiKey={process.env.NEXT_PUBLIC_GENAI_API_KEY}>
                    <AnimatePresence mode="wait">
                        <motion.div
                            initial={{ opacity: 0 }}
                            animate={{ opacity: 1 }}
                            exit={{ opacity: 0 }}
                            transition={{ duration: 0.5 }}
                            className="min-h-screen"
                        >
                            {children}
                        </motion.div>
                    </AnimatePresence>
                </GenAIProvider>
            </body>
        </html>
    );
}