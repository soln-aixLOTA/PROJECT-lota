'use client';

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useEffect, useState } from 'react';
import { ErrorBoundary } from 'react-error-boundary';

function ErrorFallback({ error }: { error: Error }) {
    return (
        <div className="min-h-screen flex items-center justify-center bg-background">
            <div className="max-w-xl p-6 bg-secondary/30 backdrop-blur-xl rounded-lg">
                <h2 className="text-xl font-semibold text-red-500 mb-4">Something went wrong</h2>
                <pre className="text-sm text-white/60 overflow-auto">{error.message}</pre>
                <button
                    onClick={() => window.location.reload()}
                    className="mt-4 px-4 py-2 bg-accent hover:bg-accent/90 text-accent-foreground rounded-lg transition-colors"
                >
                    Try again
                </button>
            </div>
        </div>
    );
}

export function Providers({ children }: { children: React.ReactNode }) {
    const [queryClient] = useState(() => new QueryClient({
        defaultOptions: {
            queries: {
                staleTime: 60 * 1000, // 1 minute
                retry: 1,
                refetchOnWindowFocus: false,
            },
        },
    }));

    useEffect(() => {
        // Add theme detection
        const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
        document.documentElement.classList.toggle('dark', isDark);

        // Listen for theme changes
        const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
        const handleChange = (e: MediaQueryListEvent) => {
            document.documentElement.classList.toggle('dark', e.matches);
        };

        mediaQuery.addEventListener('change', handleChange);
        return () => mediaQuery.removeEventListener('change', handleChange);
    }, []);

    return (
        <ErrorBoundary FallbackComponent={ErrorFallback}>
            <QueryClientProvider client={queryClient}>
                {children}
            </QueryClientProvider>
        </ErrorBoundary>
    );
} 