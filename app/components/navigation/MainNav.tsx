"use client";
import { cn } from '@/lib/utils';
import { motion } from 'framer-motion';
import Link from 'next/link';
import { usePathname } from 'next/navigation';

const navItems = [
    {
        name: 'Dashboard',
        href: '/dashboard',
    },
    {
        name: 'Agents',
        href: '/agents',
    },
    {
        name: 'Analytics',
        href: '/analytics',
    },
    {
        name: 'Settings',
        href: '/settings',
    },
];

export function MainNav() {
    const pathname = usePathname();

    return (
        <nav className="flex items-center space-x-6">
            {/* Logo */}
            <Link href="/" className="flex items-center space-x-2">
                <motion.div
                    initial={{ opacity: 0, scale: 0.5 }}
                    animate={{ opacity: 1, scale: 1 }}
                    transition={{ duration: 0.5 }}
                >
                    <div className="w-8 h-8 rounded-lg bg-accent" />
                </motion.div>
                <span className="font-bold text-xl">LotaBots</span>
            </Link>

            {/* Navigation Items */}
            <div className="hidden md:flex items-center space-x-6">
                {navItems.map((item) => {
                    const isActive = pathname === item.href;
                    return (
                        <Link
                            key={item.href}
                            href={item.href}
                            className={cn(
                                'relative py-2 text-sm font-medium transition-colors hover:text-white/80',
                                isActive ? 'text-white' : 'text-white/60'
                            )}
                        >
                            {item.name}
                            {isActive && (
                                <motion.div
                                    className="absolute -bottom-px left-0 right-0 h-px bg-accent"
                                    layoutId="navbar-indicator"
                                    transition={{ type: "spring", bounce: 0.25 }}
                                />
                            )}
                        </Link>
                    );
                })}
            </div>

            {/* Mobile Menu Button */}
            <button className="md:hidden p-2 rounded-md hover:bg-white/10">
                <svg
                    className="w-6 h-6"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                >
                    <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M4 6h16M4 12h16M4 18h16"
                    />
                </svg>
            </button>
        </nav>
    );
} 