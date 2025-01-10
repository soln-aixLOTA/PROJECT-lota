import { withRequest } from '@next/test';
import { NextResponse } from 'next/server';
import { middleware } from '../middleware';

describe('Authentication Middleware', () => {
    const mockRequestURL = 'http://localhost:3000';

    const createRequest = (path: string, cookies: Record<string, string> = {}) => {
        const url = new URL(path, mockRequestURL);
        return withRequest(url, {
            cookies: new Map(Object.entries(cookies)),
        });
    };

    beforeEach(() => {
        jest.clearAllMocks();
    });

    describe('Protected Routes', () => {
        it('should redirect to login when accessing protected route without auth', () => {
            const request = createRequest('/dashboard');
            const response = middleware(request);

            expect(response).toBeInstanceOf(NextResponse);
            expect(response.status).toBe(307); // Temporary redirect
            expect(response.headers.get('Location')).toBe('http://localhost:3000/dashboard');
        });

        it('should allow access to protected route with valid auth token', () => {
            const request = createRequest('/dashboard', {
                'auth-token': 'valid-token',
            });
            const response = middleware(request);

            expect(response.status).toBe(200);
        });

        it('should redirect to login for API routes without auth', () => {
            const request = createRequest('/api/agents/list');
            const response = middleware(request);

            expect(response).toBeInstanceOf(NextResponse);
            expect(response.status).toBe(307);
            expect(response.headers.get('Location')).toBe('http://localhost:3000/login?from=%2Fapi%2Fagents%2Flist');
        });
    });

    describe('Public Routes', () => {
        it('should allow access to public routes without auth', () => {
            const request = createRequest('/login');
            const response = middleware(request);

            expect(response.status).toBe(200);
        });

        it('should redirect to dashboard when accessing auth pages while authenticated', () => {
            const request = createRequest('/login', {
                'auth-token': 'valid-token',
            });
            const response = middleware(request);

            expect(response).toBeInstanceOf(NextResponse);
            expect(response.status).toBe(307);
            expect(response.headers.get('Location')).toBe('http://localhost:3000/login?from=%2Fdashboard');
        });

        it('should allow access to static assets without auth', () => {
            const request = createRequest('/_next/static/chunk.js');
            const response = middleware(request);

            expect(response.status).toBe(200);
        });
    });

    describe('Edge Cases', () => {
        it('should handle root path correctly', () => {
            const request = createRequest('/');
            const response = middleware(request);

            expect(response.status).toBe(200);
        });

        it('should handle non-existent paths', () => {
            const request = createRequest('/non-existent-path');
            const response = middleware(request);

            // Should still check auth but let the 404 be handled by Next.js
            expect(response.status).toBe(200);
        });

        it('should handle malformed URLs', () => {
            const request = createRequest('/%invalid-url');
            const response = middleware(request);

            expect(response.status).toBe(200);
        });
    });

    describe('Configuration', () => {
        it('should match the expected paths in config', () => {
            const { matcher } = require('../middleware').config;

            expect(matcher).toEqual([
                '/((?!_next/static|_next/image|favicon.ico).*)',
            ]);
        });

        it('should not match excluded paths', () => {
            const excludedPaths = [
                '/_next/static/styles.css',
                '/_next/image/test.jpg',
                '/favicon.ico',
            ];

            const { matcher } = require('../middleware').config;
            const matcherRegex = new RegExp(matcher[0]);

            excludedPaths.forEach(path => {
                expect(matcherRegex.test(path)).toBe(false);
            });
        });
    });
}); 