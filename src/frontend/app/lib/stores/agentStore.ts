import { create } from 'zustand';
import { AgentState } from '../types';

const useAgentStore = create<AgentState>((set) => ({
    agents: [],
    selectedAgent: null,
    isModalOpen: false,
    isLoading: false,
    error: null,
    setModalOpen: (isOpen) => set({ isModalOpen: isOpen }),
    setSelectedAgent: (agent) => set({ selectedAgent: agent }),
    fetchAgents: async () => {
        set({ isLoading: true, error: null });
        try {
            const response = await fetch('/api/agents');
            const data = await response.json();
            set({ agents: data, isLoading: false });
        } catch (error) {
            set({ error: (error as Error).message, isLoading: false });
        }
    },
}));

export default useAgentStore; 