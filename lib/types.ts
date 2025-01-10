export interface Agent {
    id: string;
    name: string;
    type: string;
    status: 'Active' | 'Inactive';
    conversations: number;
    successRate: number;
    description: string;
    config?: Record<string, any>;
    createdAt: Date;
    lastActive: Date;
}

export interface AgentConfig {
    id: string;
    name: string;
    config?: Record<string, any>;
}

export interface AgentTemplate {
    id: string;
    name: string;
    description: string;
    icon: string;
    category: string;
    capabilities: string[];
}

export interface AgentState {
    agents: Agent[];
    selectedAgent: AgentConfig | null;
    isModalOpen: boolean;
    isLoading: boolean;
    error: string | null;
    setModalOpen: (isOpen: boolean) => void;
    setSelectedAgent: (agent: AgentConfig | null) => void;
    fetchAgents: () => Promise<void>;
} 