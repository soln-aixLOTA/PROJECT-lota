import { create } from 'zustand';

export interface AgentTemplate {
    id: string;
    name: string;
    description: string;
    icon: string;
    category: string;
    capabilities: string[];
}

export interface Agent {
    id: string;
    name: string;
    description: string;
    templateId: string;
    status: 'online' | 'offline' | 'training' | 'error';
    createdAt: string;
    lastActive: string;
    metrics: {
        conversations: number;
        successRate: number;
        averageResponseTime: number;
        userSatisfaction: number;
    };
    configuration: Record<string, any>;
}

interface AgentState {
    agents: Agent[];
    templates: AgentTemplate[];
    selectedAgent: Agent | null;
    isCreatingAgent: boolean;
    isLoading: boolean;
    error: string | null;
    fetchAgents: () => Promise<void>;
    fetchTemplates: () => Promise<void>;
    createAgent: (templateId: string, configuration: Record<string, any>) => Promise<void>;
    updateAgent: (agentId: string, updates: Partial<Agent>) => Promise<void>;
    deleteAgent: (agentId: string) => Promise<void>;
    setSelectedAgent: (agent: Agent | null) => void;
}

// Mock templates data
const mockTemplates: AgentTemplate[] = [
    {
        id: 'customer-support',
        name: 'Customer Support Agent',
        description: 'AI agent specialized in handling customer inquiries and support tickets.',
        icon: '🎮',
        category: 'Support',
        capabilities: ['Ticket Resolution', 'FAQ Handling', 'User Assistance'],
    },
    {
        id: 'sales-assistant',
        name: 'Sales Assistant',
        description: 'AI agent designed to assist with sales inquiries and product recommendations.',
        icon: '💼',
        category: 'Sales',
        capabilities: ['Product Recommendations', 'Price Quotes', 'Lead Qualification'],
    },
    {
        id: 'data-analyst',
        name: 'Data Analysis Assistant',
        description: 'AI agent that helps analyze data and generate insights.',
        icon: '📊',
        category: 'Analytics',
        capabilities: ['Data Analysis', 'Report Generation', 'Trend Detection'],
    },
];

// Mock agents data
const mockAgents: Agent[] = [
    {
        id: '1',
        name: 'Support Bot Alpha',
        description: 'Primary customer support agent for technical issues',
        templateId: 'customer-support',
        status: 'online',
        createdAt: '2024-01-15T10:00:00Z',
        lastActive: '2024-01-20T15:30:00Z',
        metrics: {
            conversations: 1250,
            successRate: 94.5,
            averageResponseTime: 8.2,
            userSatisfaction: 4.8,
        },
        configuration: {
            language: 'en',
            timezone: 'UTC',
            responseStyle: 'professional',
        },
    },
];

// Simulated API calls
const api = {
    fetchAgents: async (): Promise<Agent[]> => {
        await new Promise(resolve => setTimeout(resolve, 1000));
        return mockAgents;
    },
    fetchTemplates: async (): Promise<AgentTemplate[]> => {
        await new Promise(resolve => setTimeout(resolve, 1000));
        return mockTemplates;
    },
    createAgent: async (templateId: string, configuration: Record<string, any>): Promise<Agent> => {
        await new Promise(resolve => setTimeout(resolve, 1500));
        const template = mockTemplates.find(t => t.id === templateId);
        if (!template) throw new Error('Template not found');

        return {
            id: Math.random().toString(36).substr(2, 9),
            name: `${template.name} ${Math.floor(Math.random() * 1000)}`,
            description: template.description,
            templateId,
            status: 'training',
            createdAt: new Date().toISOString(),
            lastActive: new Date().toISOString(),
            metrics: {
                conversations: 0,
                successRate: 0,
                averageResponseTime: 0,
                userSatisfaction: 0,
            },
            configuration,
        };
    },
};

export const useAgentStore = create<AgentState>((set, get) => ({
    agents: [],
    templates: [],
    selectedAgent: null,
    isCreatingAgent: false,
    isLoading: false,
    error: null,

    fetchAgents: async () => {
        set({ isLoading: true, error: null });
        try {
            const agents = await api.fetchAgents();
            set({ agents, isLoading: false });
        } catch (error) {
            set({ error: 'Failed to fetch agents', isLoading: false });
        }
    },

    fetchTemplates: async () => {
        set({ isLoading: true, error: null });
        try {
            const templates = await api.fetchTemplates();
            set({ templates, isLoading: false });
        } catch (error) {
            set({ error: 'Failed to fetch templates', isLoading: false });
        }
    },

    createAgent: async (templateId: string, configuration: Record<string, any>) => {
        set({ isCreatingAgent: true, error: null });
        try {
            const newAgent = await api.createAgent(templateId, configuration);
            set(state => ({
                agents: [...state.agents, newAgent],
                isCreatingAgent: false,
            }));
        } catch (error) {
            set({ error: 'Failed to create agent', isCreatingAgent: false });
        }
    },

    updateAgent: async (agentId: string, updates: Partial<Agent>) => {
        set({ isLoading: true, error: null });
        try {
            // Simulate API call
            await new Promise(resolve => setTimeout(resolve, 1000));
            set(state => ({
                agents: state.agents.map(agent =>
                    agent.id === agentId ? { ...agent, ...updates } : agent
                ),
                isLoading: false,
            }));
        } catch (error) {
            set({ error: 'Failed to update agent', isLoading: false });
        }
    },

    deleteAgent: async (agentId: string) => {
        set({ isLoading: true, error: null });
        try {
            // Simulate API call
            await new Promise(resolve => setTimeout(resolve, 1000));
            set(state => ({
                agents: state.agents.filter(agent => agent.id !== agentId),
                isLoading: false,
            }));
        } catch (error) {
            set({ error: 'Failed to delete agent', isLoading: false });
        }
    },

    setSelectedAgent: (agent) => {
        set({ selectedAgent: agent });
    },
})); 