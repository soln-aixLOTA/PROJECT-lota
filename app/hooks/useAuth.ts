import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useRouter } from 'next/router';
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

  const login = useMutation({
    mutationFn: (credentials: LoginRequest) =>
      api.post('/api/auth/login', credentials).then((res) => res.data),
    onSuccess: (data) => {
      queryClient.setQueryData(['auth-session'], data);
      router.push('/dashboard');
    },
  });

  const register = useMutation({
    mutationFn: (userData: RegisterRequest) =>
      api.post('/api/auth/register', userData).then((res) => res.data),
    onSuccess: (data) => {
      queryClient.setQueryData(['auth-session'], data);
      router.push('/dashboard');
    },
  });

  const logout = useMutation({
    mutationFn: () => api.post('/api/auth/logout').then((res) => res.data),
    onSuccess: () => {
      queryClient.clear();
      router.push('/login');
    },
  });

  const resetPassword = useMutation({
    mutationFn: (data: PasswordResetRequest) =>
      api.post('/api/auth/reset-password', data).then((res) => res.data),
  });

  const updatePassword = useMutation({
    mutationFn: (data: PasswordUpdateRequest) =>
      api.post('/api/auth/update-password', data).then((res) => res.data),
  });

  const updateProfile = useMutation({
    mutationFn: (data: Partial<User>) =>
      api.put('/api/auth/profile', data).then((res) => res.data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['auth-session'] });
    },
  });

  return {
    user: session?.user,
    isLoading,
    error,
    login,
    register,
    logout,
    resetPassword,
    updatePassword,
    updateProfile,
  };
} 