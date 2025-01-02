import { api } from '@/app/lib/utils/api';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, renderHook } from '@testing-library/react';
import { useAuth } from '../useAuth';

// Mock next/router
jest.mock('next/router', () => ({
    useRouter: () => ({
        push: jest.fn(),
    }),
}));

// Mock api utility
jest.mock('@/app/lib/utils/api', () => ({
    api: {
        get: jest.fn().mockImplementation((url) => {
            if (url === '/api/auth/session') {
                return Promise.resolve({ data: { user: { id: 1, name: 'Test User' } } });
            }
            return Promise.resolve({ data: {} });
        }),
        post: jest.fn().mockImplementation((url, data) => {
            if (url === '/api/auth/login') {
                return Promise.resolve({ data: { token: 'test-token' } });
            }
            if (url === '/api/auth/register') {
                return Promise.resolve({ data: { user: { id: 1, name: 'Test User' } } });
            }
            if (url === '/api/auth/logout') {
                return Promise.resolve({ data: { success: true } });
            }
            if (url === '/api/auth/reset-password') {
                return Promise.resolve({ data: { success: true } });
            }
            return Promise.resolve({ data: {} });
        }),
        put: jest.fn().mockImplementation((url, data) => {
            if (url === '/api/auth/profile') {
                return Promise.resolve({ data: { success: true } });
            }
            return Promise.resolve({ data: {} });
        }),
    },
}));

describe('useAuth', () => {
    let queryClient: QueryClient;

    beforeEach(() => {
        queryClient = new QueryClient({
            defaultOptions: {
                queries: {
                    retry: false,
                },
            },
        });
        jest.clearAllMocks();
    });

    const wrapper = ({ children }: { children: React.ReactNode }) => (
        <QueryClientProvider client={queryClient}>
            {children}
        </QueryClientProvider>
    );

    it('should handle successful login', async () => {
        const mockUser = {
            id: '1',
            email: 'test@example.com',
            name: 'Test User',
        };

        (api.post as jest.Mock).mockResolvedValueOnce({
            data: { user: mockUser },
        });

        const { result } = renderHook(() => useAuth(), { wrapper });

        await act(async () => {
            await result.current.login.mutateAsync({
                email: 'test@example.com',
                password: 'password123',
            });
        });

        expect(api.post).toHaveBeenCalledWith('/api/auth/login', {
            email: 'test@example.com',
            password: 'password123',
        });
        expect(result.current.user).toEqual(mockUser);
    });

    it('should handle login failure', async () => {
        (api.post as jest.Mock).mockRejectedValueOnce(new Error('Invalid credentials'));

        const { result } = renderHook(() => useAuth(), { wrapper });

        try {
            await act(async () => {
                await result.current.login.mutateAsync({
                    email: 'test@example.com',
                    password: 'wrongpassword',
                });
            });
        } catch (error) {
            expect(error).toBeDefined();
        }

        expect(api.post).toHaveBeenCalledWith('/api/auth/login', {
            email: 'test@example.com',
            password: 'wrongpassword',
        });
        expect(result.current.user).toBeUndefined();
    });

    it('should handle successful registration', async () => {
        const mockUser = {
            id: '1',
            email: 'new@example.com',
            name: 'New User',
        };

        (api.post as jest.Mock).mockResolvedValueOnce({
            data: { user: mockUser },
        });

        const { result } = renderHook(() => useAuth(), { wrapper });

        await act(async () => {
            await result.current.register.mutateAsync({
                email: 'new@example.com',
                password: 'password123',
                name: 'New User',
            });
        });

        expect(api.post).toHaveBeenCalledWith('/api/auth/register', {
            email: 'new@example.com',
            password: 'password123',
            name: 'New User',
        });
        expect(result.current.user).toEqual(mockUser);
    });

    it('should handle successful logout', async () => {
        (api.post as jest.Mock).mockResolvedValueOnce({
            data: { success: true },
        });

        const { result } = renderHook(() => useAuth(), { wrapper });

        await act(async () => {
            await result.current.logout.mutateAsync();
        });

        expect(api.post).toHaveBeenCalledWith('/api/auth/logout');
        expect(result.current.user).toBeUndefined();
        expect(queryClient.getQueryData(['auth-session'])).toBeUndefined();
    });

    it('should handle password reset request', async () => {
        (api.post as jest.Mock).mockResolvedValueOnce({
            data: { message: 'Password reset email sent' },
        });

        const { result } = renderHook(() => useAuth(), { wrapper });

        await act(async () => {
            await result.current.resetPassword.mutateAsync({
                email: 'test@example.com',
            });
        });

        expect(api.post).toHaveBeenCalledWith('/api/auth/reset-password', {
            email: 'test@example.com',
        });
    });

    it('should handle password update', async () => {
        (api.post as jest.Mock).mockResolvedValueOnce({
            data: { success: true },
        });

        const { result } = renderHook(() => useAuth(), { wrapper });

        await act(async () => {
            await result.current.updatePassword.mutateAsync({
                currentPassword: 'oldpass123',
                newPassword: 'newpass123',
            });
        });

        expect(api.post).toHaveBeenCalledWith('/api/auth/update-password', {
            currentPassword: 'oldpass123',
            newPassword: 'newpass123',
        });
    });

    it('should handle profile update', async () => {
        const updatedUser = {
            id: '1',
            email: 'test@example.com',
            name: 'Updated Name',
        };

        (api.put as jest.Mock).mockResolvedValueOnce({
            data: { user: updatedUser },
        });

        const { result } = renderHook(() => useAuth(), { wrapper });

        await act(async () => {
            await result.current.updateProfile.mutateAsync({
                name: 'Updated Name',
            });
        });

        expect(api.put).toHaveBeenCalledWith('/api/auth/profile', {
            name: 'Updated Name',
        });
        expect(queryClient.getQueryState(['auth-session'])).toBeDefined();
    });

    it('should handle session loading state', async () => {
        (api.get as jest.Mock).mockImplementationOnce(
            () => new Promise((resolve) => setTimeout(resolve, 100))
        );

        const { result } = renderHook(() => useAuth(), { wrapper });

        expect(result.current.isLoading).toBe(true);

        await act(async () => {
            await new Promise((resolve) => setTimeout(resolve, 150));
        });

        expect(result.current.isLoading).toBe(false);
    });

    it('should handle session error state', async () => {
        const error = new Error('Failed to fetch session');
        (api.get as jest.Mock).mockRejectedValueOnce(error);

        const { result } = renderHook(() => useAuth(), { wrapper });

        await act(async () => {
            await new Promise((resolve) => setTimeout(resolve, 10));
        });

        expect(result.current.error).toBeDefined();
        expect(result.current.user).toBeUndefined();
    });
}); 