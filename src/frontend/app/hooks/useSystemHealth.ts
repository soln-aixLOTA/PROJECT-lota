import { useMutation, useQuery } from '@tanstack/react-query';
import { api } from '../utils/api';

interface SystemHealth {
    status: 'healthy' | 'warning' | 'error';
    metrics: {
        cpu_usage: number;
        memory_usage: number;
        active_users: number;
        average_response_time: number;
    };
    services: {
        name: string;
        status: 'healthy' | 'warning' | 'error';
        latency: number;
    }[];
}

interface SystemMetrics {
    timeRange: string;
    data: {
        timestamp: string;
        value: number;
    }[];
}

export function useSystemHealth() {
    const {
        data: health,
        isLoading: isHealthLoading,
        error: healthError,
        refetch: refetchHealth,
    } = useQuery<SystemHealth>({
        queryKey: ['system-health'],
        queryFn: () => api.get('/api/system/health').then((res) => res.data),
        refetchInterval: 30000, // Refetch every 30 seconds
    });

    const {
        data: metrics,
        isPending: isMetricsLoading,
        error: metricsError,
        mutate: fetchMetrics,
    } = useMutation<SystemMetrics, Error, string>({
        mutationFn: async (timeRange: string = '1h') => {
            const response = await api.get('/api/system/metrics', { params: { timeRange } });
            return response.data;
        },
    });

    return {
        health,
        metrics,
        isLoading: isHealthLoading || isMetricsLoading,
        error: healthError || metricsError,
        refetch: refetchHealth,
        fetchMetrics,
    };
} 