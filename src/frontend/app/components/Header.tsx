'use client';

import { cn } from '@/lib/utils';
import Link from 'next/link';

interface HeaderProps {
    className?: string;
}

export default function Header({ className }: HeaderProps) {
    return (
        <header className={cn("w-full border-b bg-background", className)}>
            <div className="container flex h-16 items-center space-x-4 sm:justify-between sm:space-x-0">
                <div className="flex gap-6 md:gap-10">
                    <Link href="/" className="flex items-center space-x-2">
                        <span className="inline-block font-bold">LotaBots</span>
                    </Link>
                    <nav className="flex gap-6">
                        <Link
                            href="/dashboard"
                            className="flex items-center text-sm font-medium text-muted-foreground hover:text-primary"
                        >
                            Dashboard
                        </Link>
                        <Link
                            href="/agents"
                            className="flex items-center text-sm font-medium text-muted-foreground hover:text-primary"
                        >
                            Agents
                        </Link>
                        <Link
                            href="/settings"
                            className="flex items-center text-sm font-medium text-muted-foreground hover:text-primary"
                        >
                            Settings
                        </Link>
                    </nav>
                </div>
            </div>
        </header>
    );
} 