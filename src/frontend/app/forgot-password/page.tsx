'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { useAuth } from '@/hooks/useAuth';
import { motion } from 'framer-motion';
import Link from 'next/link';
import { useState } from 'react';

export default function ForgotPasswordPage() {
    const { resetPassword } = useAuth();
    const [email, setEmail] = useState('');
    const [submitted, setSubmitted] = useState(false);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        resetPassword.mutate({ email });
        setSubmitted(true);
    };

    if (submitted && !resetPassword.error) {
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
                            <CardTitle>Check your email</CardTitle>
                            <CardDescription>
                                We&apos;ve sent you a password reset link. Please check your email.
                            </CardDescription>
                        </CardHeader>
                        <CardContent>
                            <div className="flex justify-center">
                                <Link
                                    href="/login"
                                    className="text-sm text-accent hover:text-accent/90"
                                >
                                    Back to login
                                </Link>
                            </div>
                        </CardContent>
                    </Card>
                </motion.div>
            </div>
        );
    }

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
                        <CardTitle>Reset your password</CardTitle>
                        <CardDescription>
                            Enter your email address and we&apos;ll send you a link to reset your password.
                        </CardDescription>
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
                            <div className="flex items-center justify-end">
                                <Link
                                    href="/login"
                                    className="text-sm text-accent hover:text-accent/90"
                                >
                                    Back to login
                                </Link>
                            </div>
                            <button
                                type="submit"
                                disabled={resetPassword.isPending}
                                className="w-full rounded-md bg-accent py-2 px-4 text-white hover:bg-accent/90 focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed"
                            >
                                {resetPassword.isPending ? 'Sending...' : 'Send reset link'}
                            </button>
                            {resetPassword.error && (
                                <p className="text-red-500 text-sm mt-2">
                                    {resetPassword.error.message || 'Failed to send reset link'}
                                </p>
                            )}
                        </form>
                    </CardContent>
                </Card>
            </motion.div>
        </div>
    );
} 