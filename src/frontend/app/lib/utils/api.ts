import axios, { AxiosError, AxiosResponse } from 'axios';
import { useRouter } from 'next/navigation';
import { useEffect } from 'react';

const api = axios.create({
    baseURL: process.env.NEXT_PUBLIC_API_URL || '/api',
    headers: {
        'Content-Type': 'application/json',
    },
});

// Add request interceptor for authentication
api.interceptors.request.use(
    (config) => {
        // Only run on client side
        if (typeof window !== 'undefined') {
            const token = localStorage.getItem('auth_token');
            if (token) {
                config.headers.Authorization = `Bearer ${token}`;
            }
        }
        return config;
    },
    (error: AxiosError) => {
        return Promise.reject(new Error(error.message));
    }
);

// Add response interceptor for error handling
api.interceptors.response.use(
    (response: AxiosResponse) => response,
    async (error: AxiosError) => {
        if (error.response?.status === 401) {
            // Only run on client side
            if (typeof window !== 'undefined') {
                localStorage.removeItem('auth_token');
                window.location.href = '/login';
            }
        }
        return Promise.reject(error);
    }
);

// Custom hook for handling API errors
export function useApiErrorHandler() {
    const router = useRouter();

    useEffect(() => {
        const handleUnauthorized = () => {
            localStorage.removeItem('auth_token');
            router.push('/login');
        };

        // Add event listener for custom unauthorized event
        window.addEventListener('unauthorized', handleUnauthorized);

        return () => {
            window.removeEventListener('unauthorized', handleUnauthorized);
        };
    }, [router]);
}

// Helper function to format error messages
export function formatApiError(error: unknown): string {
    if (error instanceof AxiosError) {
        return error.response?.data?.message || error.message;
    }
    if (error instanceof Error) {
        return error.message;
    }
    return 'An unknown error occurred';
}

export { api };
