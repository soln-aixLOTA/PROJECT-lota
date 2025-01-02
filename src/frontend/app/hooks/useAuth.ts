import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useRouter } from 'next/navigation';
import {
    AuthResponse,
    LoginRequest,
    PasswordResetRequest,
    PasswordUpdateRequest,
    RegisterRequest,
    User,
} from '../types/auth';
import { api } from '../utils/api';

export function useAuth() {
    const queryClient = useQueryClient();
    const router = useRouter();

    const {
        data: session,
        isLoading,
        error,
    } = useQuery<AuthResponse>({
        queryKey: ['auth-session'],
        queryFn: () => api.get('/api/auth/session').then((res) => res.data),
        retry: false,
    });

    const loginMutation = useMutation({
        mutationFn: async (credentials: LoginRequest) => {
            const response = await api.post('/api/auth/login', credentials);
            return response.data;
        },
        onSuccess: (data) => {
            queryClient.setQueryData(['auth-session'], data);
            router.push('/dashboard');
        },
    });

    const registerMutation = useMutation({
        mutationFn: async (userData: RegisterRequest) => {
            const response = await api.post('/api/auth/register', userData);
            return response.data;
        },
        onSuccess: (data) => {
            queryClient.setQueryData(['auth-session'], data);
            router.push('/dashboard');
        },
    });

    const logoutMutation = useMutation({
        mutationFn: async () => {
            const response = await api.post('/api/auth/logout');
            return response.data;
        },
        onSuccess: () => {
            queryClient.clear();
            router.push('/login');
        },
    });

    const resetPasswordMutation = useMutation({
        mutationFn: async (data: PasswordResetRequest) => {
            const response = await api.post('/api/auth/reset-password', data);
            return response.data;
        },
    });

    const updatePasswordMutation = useMutation({
        mutationFn: async (data: PasswordUpdateRequest) => {
            const response = await api.post('/api/auth/update-password', data);
            return response.data;
        },
    });

    const updateProfileMutation = useMutation({
        mutationFn: async (data: Partial<User>) => {
            const response = await api.put('/api/auth/profile', data);
            return response.data;
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['auth-session'] });
        },
    });

    return {
        user: session?.user,
        isLoading,
        error,
        login: loginMutation,
        register: registerMutation,
        logout: logoutMutation,
        resetPassword: resetPasswordMutation,
        updatePassword: updatePasswordMutation,
        updateProfile: updateProfileMutation,
    };
} 