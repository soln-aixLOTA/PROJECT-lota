import { cn } from '@/lib/utils';
import { motion } from 'framer-motion';
import React from 'react';

interface AppShellProps {
    children: React.ReactNode;
    className?: string;
}

export function AppShell({ children, className }: AppShellProps) {
    return (
        <div className="min-h-screen bg-primary text-white">
            {/* Background gradient effect */}
            <div className="fixed inset-0 bg-gradient-radial from-accent/20 via-transparent to-transparent opacity-30 pointer-events-none" />

            {/* Noise texture overlay */}
            <div className="fixed inset-0 bg-[url('/noise.png')] opacity-[0.015] pointer-events-none" />

            {/* Content wrapper */}
            <motion.main
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.5 }}
                className={cn(
                    "relative z-10 mx-auto max-w-7xl px-4 sm:px-6 lg:px-8",
                    className
                )}
            >
                {children}
            </motion.main>

            {/* Floating orbs effect */}
            <div className="fixed inset-0 overflow-hidden pointer-events-none">
                <div className="absolute -top-40 -right-40 w-80 h-80 bg-accent/30 rounded-full filter blur-[100px] animate-float" />
                <div className="absolute -bottom-40 -left-40 w-80 h-80 bg-accent/20 rounded-full filter blur-[100px] animate-float" style={{ animationDelay: '-2s' }} />
            </div>
        </div>
    );
} 