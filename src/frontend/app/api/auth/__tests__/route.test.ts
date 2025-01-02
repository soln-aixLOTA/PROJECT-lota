import { sign } from 'jsonwebtoken';
import { NextRequest } from 'next/server';
import { GET, POST } from '../[...route]/route';

const JWT_SECRET = process.env.JWT_SECRET || 'your-secret-key';

describe('Auth API Routes', () => {
    beforeEach(() => {
        // Clear cookies between tests
        jest.clearAllMocks();
    });

    describe('POST /api/auth/register', () => {
        it('should register a new user successfully', async () => {
            const request = new NextRequest('http://localhost/api/auth/register', {
                method: 'POST',
                body: JSON.stringify({
                    email: 'test@example.com',
                    password: 'Password123!',
                    name: 'Test User',
                }),
            });

            const response = await POST(request);
            const data = await response.json();

            expect(response.status).toBe(200);
            expect(data).toHaveProperty('user');
            expect(data.user).toHaveProperty('email', 'test@example.com');
            expect(response.cookies.get('auth-token')).toBeDefined();
        });

        it('should validate registration input', async () => {
            const request = new NextRequest('http://localhost/api/auth/register', {
                method: 'POST',
                body: JSON.stringify({
                    email: 'invalid-email',
                    password: 'short',
                }),
            });

            const response = await POST(request);
            const data = await response.json();

            expect(response.status).toBe(400);
            expect(data).toHaveProperty('error');
        });
    });

    describe('POST /api/auth/login', () => {
        it('should login user successfully', async () => {
            // First register a user
            const registerRequest = new NextRequest('http://localhost/api/auth/register', {
                method: 'POST',
                body: JSON.stringify({
                    email: 'login@example.com',
                    password: 'Password123!',
                    name: 'Login User',
                }),
            });
            await POST(registerRequest);

            // Then try to login
            const loginRequest = new NextRequest('http://localhost/api/auth/login', {
                method: 'POST',
                body: JSON.stringify({
                    email: 'login@example.com',
                    password: 'Password123!',
                }),
            });

            const response = await POST(loginRequest);
            const data = await response.json();

            expect(response.status).toBe(200);
            expect(data).toHaveProperty('user');
            expect(data.user).toHaveProperty('email', 'login@example.com');
            expect(response.cookies.get('auth-token')).toBeDefined();
        });

        it('should reject invalid credentials', async () => {
            const request = new NextRequest('http://localhost/api/auth/login', {
                method: 'POST',
                body: JSON.stringify({
                    email: 'wrong@example.com',
                    password: 'WrongPass123!',
                }),
            });

            const response = await POST(request);
            const data = await response.json();

            expect(response.status).toBe(401);
            expect(data).toHaveProperty('error');
        });
    });

    describe('GET /api/auth/session', () => {
        it('should return user session when authenticated', async () => {
            // Create a valid token
            const token = sign(
                { userId: '123', email: 'session@example.com' },
                JWT_SECRET,
                { expiresIn: '1h' }
            );

            const request = new NextRequest('http://localhost/api/auth/session', {
                headers: {
                    cookie: `auth-token=${token}`,
                },
            });

            const response = await GET(request);
            const data = await response.json();

            expect(response.status).toBe(200);
            expect(data).toHaveProperty('user');
        });

        it('should return null when not authenticated', async () => {
            const request = new NextRequest('http://localhost/api/auth/session');

            const response = await GET(request);
            const data = await response.json();

            expect(response.status).toBe(200);
            expect(data).toHaveProperty('user', null);
        });
    });

    describe('POST /api/auth/logout', () => {
        it('should clear auth token cookie', async () => {
            const request = new NextRequest('http://localhost/api/auth/logout');

            const response = await POST(request);
            const data = await response.json();

            expect(response.status).toBe(200);
            expect(data).toHaveProperty('success', true);
            expect(response.cookies.get('auth-token')).toBeUndefined();
        });
    });

    describe('POST /api/auth/reset-password', () => {
        it('should handle password reset request', async () => {
            // First register a user
            const registerRequest = new NextRequest('http://localhost/api/auth/register', {
                method: 'POST',
                body: JSON.stringify({
                    email: 'reset@example.com',
                    password: 'Password123!',
                    name: 'Reset User',
                }),
            });
            await POST(registerRequest);

            // Then request password reset
            const request = new NextRequest('http://localhost/api/auth/reset-password', {
                method: 'POST',
                body: JSON.stringify({
                    email: 'reset@example.com',
                }),
            });

            const response = await POST(request);
            const data = await response.json();

            expect(response.status).toBe(200);
            expect(data).toHaveProperty('message');
        });

        it('should handle non-existent email', async () => {
            const request = new NextRequest('http://localhost/api/auth/reset-password', {
                method: 'POST',
                body: JSON.stringify({
                    email: 'nonexistent@example.com',
                }),
            });

            const response = await POST(request);
            const data = await response.json();

            expect(response.status).toBe(404);
            expect(data).toHaveProperty('error');
        });
    });
}); 