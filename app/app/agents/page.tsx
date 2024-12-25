'use client';

import AgentCard from '@/components/AgentCard';
import AgentConfigModal from '@/components/AgentConfigModal';
import AgentTemplateCard from '@/components/AgentTemplateCard';
import Navigation from '@/components/Navigation';
import { useAgentStore } from '@/lib/stores/agentStore';
import { PlusIcon } from '@heroicons/react/24/outline';
import { AnimatePresence, motion } from 'framer-motion';
import { useEffect, useState } from 'react';

export default function AgentsPage() {
    const {
        agents,
        templates,
        selectedAgent,
        isCreatingAgent,
        isLoading,
        error,
        fetchAgents,
        fetchTemplates,
        createAgent,
        updateAgent,
        deleteAgent,
        setSelectedAgent,
    } = useAgentStore();

    const [isCreatingNew, setIsCreatingNew] = useState(false);
    const [selectedTemplate, setSelectedTemplate] = useState<string | null>(null);
    const [searchQuery, setSearchQuery] = useState('');
    const [isConfigModalOpen, setIsConfigModalOpen] = useState(false);

    useEffect(() => {
        fetchAgents();
        fetchTemplates();
    }, [fetchAgents, fetchTemplates]);

    const filteredAgents = agents.filter(
        (agent) =>
            agent.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
            agent.description.toLowerCase().includes(searchQuery.toLowerCase())
    );

    const filteredTemplates = templates.filter(
        (template) =>
            template.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
            template.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
            template.category.toLowerCase().includes(searchQuery.toLowerCase())
    );

    const handleCreateAgent = async (templateId: string) => {
        await createAgent(templateId, {
            language: 'en',
            timezone: 'UTC',
            responseStyle: 'professional',
            useEmoji: false,
            maxResponseTime: 10,
            useAutoLearning: true,
            confidenceThreshold: 80,
        });
        setIsCreatingNew(false);
        setSelectedTemplate(null);
    };

    const handleToggleStatus = async (agentId: string, currentStatus: string) => {
        await updateAgent(agentId, {
            status: currentStatus === 'online' ? 'offline' : 'online',
        });
    };

    const handleAgentClick = (agent: typeof agents[0]) => {
        setSelectedAgent(agent);
        setIsConfigModalOpen(true);
    };

    const handleConfigSave = async (configuration: Record<string, any>) => {
        if (selectedAgent) {
            await updateAgent(selectedAgent.id, { configuration });
        }
    };

    if (error) {
        return (
            <div className="min-h-screen bg-primary flex items-center justify-center">
                <div className="text-white text-center">
                    <p className="text-xl">{error}</p>
                    <button
                        onClick={() => {
                            fetchAgents();
                            fetchTemplates();
                        }}
                        className="button-primary mt-4"
                    >
                        Retry
                    </button>
                </div>
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-primary">
            <Navigation />

            <main className="pt-24 pb-8 px-4 sm:px-6 lg:px-8">
                <div className="max-w-7xl mx-auto">
                    {/* Header */}
                    <div className="md:flex md:items-center md:justify-between mb-8">
                        <motion.div
                            initial={{ opacity: 0, y: 20 }}
                            animate={{ opacity: 1, y: 0 }}
                            transition={{ duration: 0.5 }}
                            className="flex-1 min-w-0"
                        >
                            <h1 className="text-3xl font-display font-bold leading-7 text-white sm:text-4xl sm:truncate">
                                AI Agents
                            </h1>
                            <p className="mt-1 text-lg text-white/60">
                                Create and manage your AI agents
                            </p>
                        </motion.div>

                        <motion.div
                            initial={{ opacity: 0, x: -20 }}
                            animate={{ opacity: 1, x: 0 }}
                            transition={{ duration: 0.5, delay: 0.2 }}
                            className="mt-4 flex md:mt-0 md:ml-4 space-x-4"
                        >
                            <input
                                type="text"
                                placeholder="Search agents..."
                                value={searchQuery}
                                onChange={(e) => setSearchQuery(e.target.value)}
                                className="input-field max-w-xs"
                            />
                            <button
                                onClick={() => setIsCreatingNew(true)}
                                className="button-primary flex items-center space-x-2"
                                disabled={isCreatingNew}
                            >
                                <PlusIcon className="w-5 h-5" />
                                <span>New Agent</span>
                            </button>
                        </motion.div>
                    </div>

                    {/* Loading State */}
                    {isLoading && agents.length === 0 ? (
                        <div className="flex items-center justify-center h-64">
                            <div className="text-white text-center">
                                <div className="w-8 h-8 border-2 border-accent border-t-transparent rounded-full animate-spin mx-auto mb-4" />
                                <p>Loading agents...</p>
                            </div>
                        </div>
                    ) : (
                        <>
                            {/* Create New Agent Interface */}
                            <AnimatePresence>
                                {isCreatingNew && (
                                    <motion.div
                                        initial={{ opacity: 0, y: 20 }}
                                        animate={{ opacity: 1, y: 0 }}
                                        exit={{ opacity: 0, y: -20 }}
                                        className="mb-8"
                                    >
                                        <div className="glass-panel p-6">
                                            <div className="flex items-center justify-between mb-6">
                                                <h2 className="text-xl font-semibold text-white">
                                                    Create New Agent
                                                </h2>
                                                <button
                                                    onClick={() => setIsCreatingNew(false)}
                                                    className="text-white/60 hover:text-white"
                                                >
                                                    Cancel
                                                </button>
                                            </div>

                                            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                                                {filteredTemplates.map((template) => (
                                                    <AgentTemplateCard
                                                        key={template.id}
                                                        template={template}
                                                        isSelected={selectedTemplate === template.id}
                                                        onClick={() => {
                                                            if (selectedTemplate === template.id) {
                                                                handleCreateAgent(template.id);
                                                            } else {
                                                                setSelectedTemplate(template.id);
                                                            }
                                                        }}
                                                    />
                                                ))}
                                            </div>
                                        </div>
                                    </motion.div>
                                )}
                            </AnimatePresence>

                            {/* Active Agents Grid */}
                            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                                {filteredAgents.map((agent) => (
                                    <AgentCard
                                        key={agent.id}
                                        agent={agent}
                                        onDelete={() => deleteAgent(agent.id)}
                                        onToggleStatus={() =>
                                            handleToggleStatus(agent.id, agent.status)
                                        }
                                        onClick={() => handleAgentClick(agent)}
                                    />
                                ))}
                            </div>

                            {/* Empty State */}
                            {filteredAgents.length === 0 && !isCreatingNew && (
                                <motion.div
                                    initial={{ opacity: 0 }}
                                    animate={{ opacity: 1 }}
                                    className="text-center py-12"
                                >
                                    <p className="text-white/60 mb-4">No agents found</p>
                                    <button
                                        onClick={() => setIsCreatingNew(true)}
                                        className="button-primary"
                                    >
                                        Create your first agent
                                    </button>
                                </motion.div>
                            )}
                        </>
                    )}
                </div>
            </main>

            {/* Configuration Modal */}
            {selectedAgent && (
                <AgentConfigModal
                    agent={selectedAgent}
                    isOpen={isConfigModalOpen}
                    onClose={() => {
                        setIsConfigModalOpen(false);
                        setSelectedAgent(null);
                    }}
                    onSave={handleConfigSave}
                />
            )}
        </div>
    );
} 