import { create } from 'zustand';
import { api } from '../utils/api';

interface Metric {
    value: number;
    history: number[];
}

interface Activity {
    id: string;
    type: 'success' | 'warning' | 'error';
    message: string;
    timestamp: string;
}

interface DashboardState {
    metrics: Metric[];
    activities: Activity[];
    selectedPeriod: string;
    selectedMetricIndex: number;
    isLoading: boolean;
    error: string | null;
    setSelectedPeriod: (period: string) => void;
    setSelectedMetricIndex: (index: number) => void;
    fetchDashboardData: () => Promise<void>;
}

async function fetchData(period: string) {
    const [metricsResponse, activitiesResponse] = await Promise.all([
        api.get('/api/dashboard/metrics', { params: { period } }),
        api.get('/api/dashboard/activities'),
    ]);

    return {
        metrics: metricsResponse.data,
        activities: activitiesResponse.data,
    };
}

export const useDashboardStore = create<DashboardState>((set, get) => ({
    metrics: [],
    activities: [],
    selectedPeriod: '7d',
    selectedMetricIndex: 0,
    isLoading: false,
    error: null,

    setSelectedPeriod: (period) => {
        set({ selectedPeriod: period });
        get().fetchDashboardData();
    },

    setSelectedMetricIndex: (index) => {
        set({ selectedMetricIndex: index });
    },

    fetchDashboardData: async () => {
        const { selectedPeriod } = get();
        set({ isLoading: true, error: null });

        try {
            const data = await fetchData(selectedPeriod);
            set({
                metrics: data.metrics,
                activities: data.activities,
                isLoading: false,
            });
        } catch (error) {
            set({
                error: 'Failed to fetch dashboard data',
                isLoading: false,
            });
        }
    },
})); 