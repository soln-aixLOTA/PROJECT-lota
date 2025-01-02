'use client';

import AgentConfigModal from '@/components/AgentConfigModal';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { useAgentStore } from '@/lib/stores/agentStore';
import { Agent } from '@/lib/types';
import { cn } from '@/lib/utils';
import { useEffect } from 'react';

export default function AgentsPage() {
    const {
        agents,
        selectedAgent,
        isModalOpen,
        isLoading,
        error,
        setModalOpen,
        setSelectedAgent,
        fetchAgents,
    } = useAgentStore();

    useEffect(() => {
        fetchAgents();
    }, [fetchAgents]);

    const handleConfigSave = async (config: Record<string, any>) => {
        console.log('Saving config for agent:', selectedAgent?.id, config);
        // Here you would typically make an API call to update the agent's config
    };

    if (error) {
        return (
            <div className="container mx-auto py-10">
                <div className="bg-red-500/10 text-red-500 rounded-lg p-6">
                    <h2 className="text-lg font-semibold">Error loading agents</h2>
                    <p className="mt-2">{error}</p>
                </div>
            </div>
        );
    }

    return (
        <div className="container mx-auto py-10">
            <div className="flex justify-between items-center mb-8">
                <h1 className="text-3xl font-bold">AI Agents</h1>
                <button className="px-4 py-2 bg-primary text-primary-foreground rounded-md">
                    Create Agent
                </button>
            </div>
            
            {isLoading ? (
                <div className="flex items-center justify-center h-64">
                    <div className="text-white text-center">
                        <div className="w-8 h-8 border-2 border-accent border-t-transparent rounded-full animate-spin mx-auto mb-4" />
                        <p>Loading agents...</p>
                    </div>
                </div>
            ) : (
                <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
                    {agents.map((agent: Agent) => (
                        <Card 
                            key={agent.id} 
                            className="hover:shadow-lg transition-shadow cursor-pointer"
                            onClick={() => {
                                setSelectedAgent({
                                    id: agent.id,
                                    name: agent.name,
                                    config: agent.config,
                                });
                                setModalOpen(true);
                            }}
                        >
                            <CardHeader>
                                <CardTitle>{agent.name}</CardTitle>
                                <CardDescription>{agent.type}</CardDescription>
                            </CardHeader>
                            <CardContent>
                                <div className="space-y-2">
                                    <div className="flex justify-between">
                                        <span>Status</span>
                                        <span className={cn(
                                            "px-2 py-1 rounded-full text-xs font-medium",
                                            agent.status === 'Active' ? "bg-green-100 text-green-800" : "bg-red-100 text-red-800"
                                        )}>
                                            {agent.status}
                                        </span>
                                    </div>
                                    <div className="flex justify-between">
                                        <span>Conversations</span>
                                        <span>{agent.conversations}</span>
                                    </div>
                                    <div className="flex justify-between">
                                        <span>Success Rate</span>
                                        <span>{agent.successRate}%</span>
                                    </div>
                                    <p className="text-sm text-muted-foreground mt-4">
                                        {agent.description}
                                    </p>
                                </div>
                            </CardContent>
                        </Card>
                    ))}
                </div>
            )}

            {selectedAgent && (
                <AgentConfigModal />
            )}
        </div>
    );
} 