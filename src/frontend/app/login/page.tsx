'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { useAuth } from '@/hooks/useAuth';
import { motion } from 'framer-motion';
import Link from 'next/link';
import { useState } from 'react';

export default function LoginPage() {
    const { login } = useAuth();
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        login.mutate({ email, password });
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
                        <CardTitle>Welcome back</CardTitle>
                        <CardDescription>Sign in to your account</CardDescription>
                    </CardHeader>
                    <CardContent>
                        <form onSubmit={handleSubmit} className="space-y-4">
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
                                    placeholder="Enter your password"
                                    required
                                />
                            </div>
                            <div className="flex items-center justify-between">
                                <Link
                                    href="/forgot-password"
                                    className="text-sm text-accent hover:text-accent/90"
                                >
                                    Forgot password?
                                </Link>
                                <Link
                                    href="/register"
                                    className="text-sm text-accent hover:text-accent/90"
                                >
                                    Create account
                                </Link>
                            </div>
                            <button
                                type="submit"
                                disabled={login.isPending}
                                className="w-full rounded-md bg-accent py-2 px-4 text-white hover:bg-accent/90 focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed"
                            >
                                {login.isPending ? 'Signing in...' : 'Sign in'}
                            </button>
                            {login.error && (
                                <p className="text-red-500 text-sm mt-2">
                                    {login.error.message || 'Failed to sign in'}
                                </p>
                            )}
                        </form>
                    </CardContent>
                </Card>
            </motion.div>
        </div>
    );
} 