import { useQuery } from '@tanstack/react-query';
import { api } from '../utils/api';

interface SystemHealth {
  status: string;
  services: {
    database: string;
    redis: string;
    auth: string;
  };
  uptime: number;
  ssl?: {
    expires: string;
    issuer: string;
  };
  metrics: {
    cpu_usage: number;
    memory_usage: number;
    active_users: number;
    requests_per_minute: number;
    average_response_time: number;
  };
}

export function useSystemHealth() {
  const {
    data: health,
    isLoading,
    error,
    refetch,
  } = useQuery<SystemHealth>({
    queryKey: ['system-health'],
    queryFn: () => api.get('/api/system/health').then((res) => res.data),
    refetchInterval: 30000, // Refresh every 30 seconds
    retry: 3,
  });

  const useMetrics = (timeRange: string = '1h') => {
    return useQuery({
      queryKey: ['system-metrics', timeRange],
      queryFn: () =>
        api
          .get('/api/system/metrics', { params: { timeRange } })
          .then((res) => res.data),
      refetchInterval: 60000, // Refresh every minute
    });
  };

  const useAlerts = () => {
    return useQuery({
      queryKey: ['system-alerts'],
      queryFn: () => api.get('/api/system/alerts').then((res) => res.data),
      refetchInterval: 15000, // Refresh every 15 seconds
    });
  };

  return {
    health,
    isLoading,
    error,
    refetch,
    useMetrics,
    useAlerts,
  };
} 