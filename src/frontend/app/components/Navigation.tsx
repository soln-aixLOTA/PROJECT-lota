'use client';

import { motion } from 'framer-motion';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useState } from 'react';

const navItems = [
    { name: 'Dashboard', href: '/dashboard' },
    { name: 'Agents', href: '/agents' },
    { name: 'Analytics', href: '/analytics' },
    { name: 'Settings', href: '/settings' },
];

function NavigationLink({ href, name, isActive }: { href: string; name: string; isActive: boolean }) {
    const [isHovered, setIsHovered] = useState(false);

    return (
        <Link href={href} className="relative">
            <motion.div
                className={`px-4 py-2 rounded-md transition-colors relative ${
                    isActive ? 'text-accent' : 'text-white/60 hover:text-white'
                }`}
                onHoverStart={() => setIsHovered(true)}
                onHoverEnd={() => setIsHovered(false)}
            >
                {name}
                {isActive && (
                    <motion.div
                        layoutId="activeTab"
                        className="absolute inset-0 bg-accent/10 rounded-md"
                        initial={false}
                        transition={{
                            type: "spring",
                            stiffness: 500,
                            damping: 35
                        }}
                    />
                )}
                {isHovered && !isActive && (
                    <motion.div
                        layoutId="hoverTab"
                        className="absolute inset-0 bg-white/5 rounded-md"
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        exit={{ opacity: 0 }}
                        transition={{
                            type: "spring",
                            stiffness: 500,
                            damping: 35
                        }}
                    />
                )}
            </motion.div>
        </Link>
    );
}

export default function Navigation() {
    const pathname = usePathname();
    const [isMenuOpen, setIsMenuOpen] = useState(false);

    return (
        <nav className="sticky top-0 z-50 bg-secondary/50 backdrop-blur-lg border-b border-white/10">
            <div className="container mx-auto px-4">
                <div className="flex items-center justify-between h-16">
                    <div className="flex items-center">
                        <Link href="/" className="text-xl font-bold text-accent">
                            LotaBots
                        </Link>
                    </div>

                    {/* Desktop Navigation */}
                    <div className="hidden md:flex md:items-center md:space-x-4">
                        {navItems.map((item) => (
                            <NavigationLink
                                key={item.name}
                                href={item.href}
                                name={item.name}
                                isActive={pathname === item.href}
                            />
                        ))}
                    </div>

                    {/* Mobile Menu Button */}
                    <div className="md:hidden">
                        <button
                            onClick={() => setIsMenuOpen(!isMenuOpen)}
                            className="text-white/60 hover:text-white"
                            aria-label="Toggle menu"
                        >
                            <svg
                                className="h-6 w-6"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke="currentColor"
                            >
                                {isMenuOpen ? (
                                    <path
                                        strokeLinecap="round"
                                        strokeLinejoin="round"
                                        strokeWidth={2}
                                        d="M6 18L18 6M6 6l12 12"
                                    />
                                ) : (
                                    <path
                                        strokeLinecap="round"
                                        strokeLinejoin="round"
                                        strokeWidth={2}
                                        d="M4 6h16M4 12h16M4 18h16"
                                    />
                                )}
                            </svg>
                        </button>
                    </div>
                </div>

                {/* Mobile Navigation */}
                <motion.div
                    initial={false}
                    animate={{
                        height: isMenuOpen ? 'auto' : 0,
                        opacity: isMenuOpen ? 1 : 0
                    }}
                    transition={{
                        type: "spring",
                        stiffness: 500,
                        damping: 35
                    }}
                    className="md:hidden overflow-hidden"
                >
                    <div className="px-2 pt-2 pb-3 space-y-1">
                        {navItems.map((item) => (
                            <Link
                                key={item.name}
                                href={item.href}
                                className={`block px-3 py-2 rounded-md text-base font-medium ${
                                    pathname === item.href
                                        ? 'text-accent bg-accent/10'
                                        : 'text-white/60 hover:text-white hover:bg-white/5'
                                }`}
                                onClick={() => setIsMenuOpen(false)}
                            >
                                {item.name}
                            </Link>
                        ))}
                    </div>
                </motion.div>
            </div>
        </nav>
    );
} 