import type { NextRequest } from 'next/server';
import { NextResponse } from 'next/server';

// List of paths that don't require authentication
const publicPaths = [
    '/',
    '/login',
    '/register',
    '/forgot-password',
    '/api/auth/login',
    '/api/auth/register',
    '/api/auth/reset-password',
];

// List of paths that require authentication
const protectedPaths = [
    '/dashboard',
    '/agents',
    '/analytics',
    '/settings',
    '/api/agents',
    '/api/system',
];

export function middleware(request: NextRequest) {
    const { pathname } = request.nextUrl;
    const token = request.cookies.get('auth-token')?.value;

    // Check if the path is protected and user is not authenticated
    if (protectedPaths.some((path) => pathname.startsWith(path)) && !token) {
        const url = new URL('/login', request.url);
        url.searchParams.set('from', pathname);
        return NextResponse.redirect(url);
    }

    // Check if the user is authenticated and trying to access auth pages
    if (token && publicPaths.includes(pathname)) {
        return NextResponse.redirect(new URL('/dashboard', request.url));
    }

    return NextResponse.next();
}

export const config = {
    matcher: [
        /*
         * Match all request paths except for the ones starting with:
         * - _next/static (static files)
         * - _next/image (image optimization files)
         * - favicon.ico (favicon file)
         */
        '/((?!_next/static|_next/image|favicon.ico).*)',
    ],
}; 