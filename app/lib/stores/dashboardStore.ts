import { create } from 'zustand';

export interface Metric {
    name: string;
    value: string;
    change: string;
    data: number[];
}

export interface Activity {
    id: number;
    type: string;
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

// Simulated API call - replace with actual API integration
const fetchData = async (period: string): Promise<{ metrics: Metric[]; activities: Activity[] }> => {
    // Simulate API latency
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Mock data
    return {
        metrics: [
            {
                name: 'Active Agents',
                value: '24',
                change: '+12%',
                data: [10, 15, 12, 18, 24, 22, 24],
            },
            {
                name: 'Total Conversations',
                value: '14,582',
                change: '+23.1%',
                data: [8500, 9200, 10500, 11800, 13200, 14000, 14582],
            },
            {
                name: 'Active Users',
                value: '2,338',
                change: '+15.3%',
                data: [1500, 1800, 1950, 2100, 2250, 2300, 2338],
            },
            {
                name: 'Success Rate',
                value: '94.2%',
                change: '+4.1%',
                data: [88, 89, 90, 92, 93, 93.5, 94.2],
            },
        ],
        activities: [
            {
                id: 1,
                type: 'agent_created',
                message: 'New agent "Customer Support Bot" created',
                timestamp: '2 minutes ago',
            },
            {
                id: 2,
                type: 'conversation_milestone',
                message: 'Reached 10,000 successful conversations',
                timestamp: '1 hour ago',
            },
            {
                id: 3,
                type: 'performance_alert',
                message: 'Agent "Sales Assistant" achieved 96% satisfaction rate',
                timestamp: '3 hours ago',
            },
        ],
    };
};

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