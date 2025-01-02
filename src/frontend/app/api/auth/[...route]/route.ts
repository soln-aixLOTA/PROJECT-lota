import { randomUUID } from 'crypto';
import { sign, verify } from 'jsonwebtoken';
import { NextRequest, NextResponse } from 'next/server';

const JWT_SECRET = process.env.JWT_SECRET || 'your-secret-key';
const COOKIE_NAME = 'auth-token';

// Mock user database (replace with your actual database)
const users = new Map<string, any>();

if (!global.crypto) {
  global.crypto = {
    subtle: null,
    getRandomValues: (arr) => require('crypto').randomFillSync(arr),
    randomUUID: randomUUID,
  };
}

export async function POST(request: NextRequest) {
    const route = request.nextUrl.pathname.split('/').pop();

    switch (route) {
        case 'login':
            return handleLogin(request);
        case 'register':
            return handleRegister(request);
        case 'logout':
            return handleLogout();
        case 'reset-password':
            return handleResetPassword(request);
        case 'update-password':
            return handleUpdatePassword(request);
        default:
            return NextResponse.json({ error: 'Invalid route' }, { status: 404 });
    }
}

export async function GET(request: NextRequest) {
    const route = request.nextUrl.pathname.split('/').pop();

    if (route === 'session') {
        return handleSession(request);
    }

    return NextResponse.json({ error: 'Invalid route' }, { status: 404 });
}

async function handleLogin(request: NextRequest) {
    try {
        const { email, password } = await request.json();

        // Mock authentication (replace with your actual authentication logic)
        const user = users.get(email);
        if (!user || user.password !== password) {
            return NextResponse.json(
                { error: 'Invalid email or password' },
                { status: 401 }
            );
        }

        // Create JWT token
        const token = sign({ userId: user.id, email }, JWT_SECRET, {
            expiresIn: '7d',
        });

        // Create response
        const response = NextResponse.json({
            user: {
                id: user.id,
                email: user.email,
                name: user.name,
                role: user.role,
            },
            token,
        });

        // Set cookie
        response.cookies.set(COOKIE_NAME, token, {
            httpOnly: true,
            secure: process.env.NODE_ENV === 'production',
            sameSite: 'lax',
            maxAge: 7 * 24 * 60 * 60, // 7 days
            path: '/',
        });

        return response;
    } catch (error) {
        console.error('Login error:', error);
        return NextResponse.json(
            { error: 'Internal server error' },
            { status: 500 }
        );
    }
}

async function handleRegister(request: NextRequest) {
    try {
        const { name, email, password } = await request.json();

        // Check if user already exists
        if (users.has(email)) {
            return NextResponse.json(
                { error: 'Email already registered' },
                { status: 400 }
            );
        }

        // Create new user (replace with your actual user creation logic)
        const user = {
            id: crypto.randomUUID(),
            name,
            email,
            password,
            role: 'user',
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
        };
        users.set(email, user);

        // Create JWT token
        const token = sign({ userId: user.id, email }, JWT_SECRET, {
            expiresIn: '7d',
        });

        // Create response
        const response = NextResponse.json({
            user: {
                id: user.id,
                email: user.email,
                name: user.name,
                role: user.role,
            },
            token,
        });

        // Set cookie
        response.cookies.set(COOKIE_NAME, token, {
            httpOnly: true,
            secure: process.env.NODE_ENV === 'production',
            sameSite: 'lax',
            maxAge: 7 * 24 * 60 * 60, // 7 days
            path: '/',
        });

        return response;
    } catch (error) {
        console.error('Registration error:', error);
        return NextResponse.json(
            { error: 'Internal server error' },
            { status: 500 }
        );
    }
}

async function handleLogout() {
    const response = NextResponse.json({ success: true });
    response.cookies.delete(COOKIE_NAME);
    return response;
}

async function handleSession(request: NextRequest) {
    try {
        const token = request.cookies.get(COOKIE_NAME)?.value;

        if (!token) {
            return NextResponse.json({ user: null });
        }

        // Verify token
        const decoded = verify(token, JWT_SECRET) as { userId: string; email: string };
        const user = users.get(decoded.email);

        if (!user) {
            return NextResponse.json({ user: null });
        }

        return NextResponse.json({
            user: {
                id: user.id,
                email: user.email,
                name: user.name,
                role: user.role,
            },
        });
    } catch (error) {
        console.error('Session error:', error);
        return NextResponse.json({ user: null });
    }
}

async function handleResetPassword(request: NextRequest) {
    try {
        const { email } = await request.json();

        // Check if user exists
        if (!users.has(email)) {
            return NextResponse.json(
                { error: 'Email not found' },
                { status: 404 }
            );
        }

        // In a real application, you would:
        // 1. Generate a reset token
        // 2. Save it to the database with an expiration
        // 3. Send an email with the reset link

        return NextResponse.json({
            message: 'Password reset instructions sent to your email',
        });
    } catch (error) {
        console.error('Reset password error:', error);
        return NextResponse.json(
            { error: 'Internal server error' },
            { status: 500 }
        );
    }
}

async function handleUpdatePassword(request: NextRequest) {
    try {
        const { currentPassword, newPassword } = await request.json();
        const token = request.cookies.get(COOKIE_NAME)?.value;

        if (!token) {
            return NextResponse.json(
                { error: 'Unauthorized' },
                { status: 401 }
            );
        }

        // Verify token
        const decoded = verify(token, JWT_SECRET) as { userId: string; email: string };
        const user = users.get(decoded.email);

        if (!user || user.password !== currentPassword) {
            return NextResponse.json(
                { error: 'Invalid current password' },
                { status: 401 }
            );
        }

        // Update password
        user.password = newPassword;
        user.updatedAt = new Date().toISOString();
        users.set(decoded.email, user);

        return NextResponse.json({ message: 'Password updated successfully' });
    } catch (error) {
        console.error('Update password error:', error);
        return NextResponse.json(
            { error: 'Internal server error' },
            { status: 500 }
        );
    }
} 