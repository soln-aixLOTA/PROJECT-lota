'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { useAuth } from '@/hooks/useAuth';
import { motion } from 'framer-motion';
import Link from 'next/link';
import { useState } from 'react';

export default function RegisterPage() {
    const { register } = useAuth();
    const [name, setName] = useState('');
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [confirmPassword, setConfirmPassword] = useState('');
    const [passwordError, setPasswordError] = useState('');

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        
        if (password !== confirmPassword) {
            setPasswordError('Passwords do not match');
            return;
        }
        
        setPasswordError('');
        register.mutate({ name, email, password });
    };

    return (
        <div className="min-h-screen flex items-center justify-center p-4">
            <motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.5 }}
                className="w-full max-w-md"
            >
                <Card>
                    <CardHeader>
                        <CardTitle>Create an account</CardTitle>
                        <CardDescription>Sign up to get started</CardDescription>
                    </CardHeader>
                    <CardContent>
                        <form onSubmit={handleSubmit} className="space-y-4">
                            <div>
                                <label
                                    htmlFor="name"
                                    className="block text-sm font-medium text-white/60"
                                >
                                    Name
                                </label>
                                <input
                                    id="name"
                                    type="text"
                                    value={name}
                                    onChange={(e) => setName(e.target.value)}
                                    className="mt-1 block w-full rounded-md bg-secondary/50 border border-white/10 px-3 py-2 text-white placeholder-white/40 focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent"
                                    placeholder="Enter your name"
                                    required
                                />
                            </div>
                            <div>
                                <label
                                    htmlFor="email"
                                    className="block text-sm font-medium text-white/60"
                                >
                                    Email
                                </label>
                                <input
                                    id="email"
                                    type="email"
                                    value={email}
                                    onChange={(e) => setEmail(e.target.value)}
                                    className="mt-1 block w-full rounded-md bg-secondary/50 border border-white/10 px-3 py-2 text-white placeholder-white/40 focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent"
                                    placeholder="Enter your email"
                                    required
                                />
                            </div>
                            <div>
                                <label
                                    htmlFor="password"
                                    className="block text-sm font-medium text-white/60"
                                >
                                    Password
                                </label>
                                <input
                                    id="password"
                                    type="password"
                                    value={password}
                                    onChange={(e) => setPassword(e.target.value)}
                                    className="mt-1 block w-full rounded-md bg-secondary/50 border border-white/10 px-3 py-2 text-white placeholder-white/40 focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent"
                                    placeholder="Create a password"
                                    required
                                />
                            </div>
                            <div>
                                <label
                                    htmlFor="confirmPassword"
                                    className="block text-sm font-medium text-white/60"
                                >
                                    Confirm Password
                                </label>
                                <input
                                    id="confirmPassword"
                                    type="password"
                                    value={confirmPassword}
                                    onChange={(e) => setConfirmPassword(e.target.value)}
                                    className="mt-1 block w-full rounded-md bg-secondary/50 border border-white/10 px-3 py-2 text-white placeholder-white/40 focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent"
                                    placeholder="Confirm your password"
                                    required
                                />
                            </div>
                            {passwordError && (
                                <p className="text-red-500 text-sm">{passwordError}</p>
                            )}
                            <div className="flex items-center justify-end">
                                <Link
                                    href="/login"
                                    className="text-sm text-accent hover:text-accent/90"
                                >
                                    Already have an account?
                                </Link>
                            </div>
                            <button
                                type="submit"
                                disabled={register.isPending}
                                className="w-full rounded-md bg-accent py-2 px-4 text-white hover:bg-accent/90 focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed"
                            >
                                {register.isPending ? 'Creating account...' : 'Create account'}
                            </button>
                            {register.error && (
                                <p className="text-red-500 text-sm mt-2">
                                    {register.error.message || 'Failed to create account'}
                                </p>
                            )}
                        </form>
                    </CardContent>
                </Card>
            </motion.div>
        </div>
    );
} 