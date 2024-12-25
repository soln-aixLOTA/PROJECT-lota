import { AgentBulkActions } from '@/components/agents/AgentBulkActions';
import { AgentDetailsModal } from '@/components/agents/AgentDetailsModal';
import { AgentFilters } from '@/components/agents/AgentFilters';
import { CreateAgentForm } from '@/components/agents/CreateAgentForm';
import { AppShell } from '@/components/layout/AppShell';
import { MainNav } from '@/components/navigation/MainNav';
import { Button } from '@/components/ui/Button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/Dropdown';
import { Modal, ModalContent, ModalDescription, ModalHeader, ModalTitle, ModalTrigger } from '@/components/ui/Modal';
import { cn } from '@/lib/utils';
import React from 'react';

// Mock data for agents
const agents = [
    {
        id: 1,
        name: 'Customer Support Agent',
        status: 'Active',
        type: 'Support',
        conversations: 156,
        successRate: 98.2,
        description: 'An AI agent specialized in handling customer inquiries and providing support across multiple channels. Capable of understanding and resolving common customer issues quickly and efficiently.',
        createdAt: new Date('2023-10-26T10:00:00'),
        lastActive: new Date(),
        config: {
            responseTime: 'fast',
            language: 'english',
            supportedChannels: ['email', 'chat', 'voice'],
            knowledgeBase: 'customer-support-v2',
            maxConcurrentChats: 5,
        },
    },
    {
        id: 2,
        name: 'Sales Assistant',
        status: 'Active',
        type: 'Sales',
        conversations: 89,
        successRate: 95.7,
        description: 'A specialized sales agent designed to qualify leads, handle product inquiries, and assist in the sales process. Trained on our product catalog and sales methodologies.',
        createdAt: new Date('2023-10-25T14:30:00'),
        lastActive: new Date('2023-10-27T09:00:00'),
        config: {
            salesRegion: 'north-america',
            productLines: ['enterprise', 'smb'],
            leadQualificationModel: 'v3',
            integrations: ['salesforce', 'hubspot'],
        },
    },
    {
        id: 3,
        name: 'Data Analysis Agent',
        status: 'Inactive',
        type: 'Analytics',
        conversations: 45,
        successRate: 99.1,
        description: 'Advanced analytics agent capable of processing large datasets, generating insights, and creating detailed reports. Specializes in trend analysis and anomaly detection.',
        createdAt: new Date('2023-10-20T09:15:00'),
        lastActive: new Date('2023-10-22T18:00:00'),
        config: {
            dataSources: ['database', 'logs', 'metrics'],
            analysisTypes: ['trend', 'anomaly', 'forecast'],
            reportingFrequency: 'weekly',
            alertThreshold: 0.85,
        },
    },
];

export default function AgentsPage() {
    const [selectedAgents, setSelectedAgents] = React.useState<number[]>([]);
    const [searchQuery, setSearchQuery] = React.useState('');
    const [statusFilter, setStatusFilter] = React.useState('all');
    const [typeFilter, setTypeFilter] = React.useState('all');
    const [sortField, setSortField] = React.useState('name');
    const [isCreateModalOpen, setIsCreateModalOpen] = React.useState(false);
    const [isDetailsModalOpen, setIsDetailsModalOpen] = React.useState(false);
    const [selectedAgent, setSelectedAgent] = React.useState<typeof agents[0] | null>(null);

    // Filter and sort agents
    const filteredAgents = React.useMemo(() => {
        return agents
            .filter((agent) => {
                const matchesSearch = agent.name.toLowerCase().includes(searchQuery.toLowerCase());
                const matchesStatus = statusFilter === 'all' || agent.status.toLowerCase() === statusFilter;
                const matchesType = typeFilter === 'all' || agent.type.toLowerCase() === typeFilter;
                return matchesSearch && matchesStatus && matchesType;
            })
            .sort((a, b) => {
                if (sortField === 'name') return a.name.localeCompare(b.name);
                if (sortField === 'conversations') return b.conversations - a.conversations;
                if (sortField === 'successRate') return b.successRate - a.successRate;
                return 0;
            });
    }, [searchQuery, statusFilter, typeFilter, sortField]);

    // Handle bulk actions
    const handleBulkActivate = () => {
        console.log('Activating agents:', selectedAgents);
    };

    const handleBulkDeactivate = () => {
        console.log('Deactivating agents:', selectedAgents);
    };

    const handleBulkDelete = () => {
        console.log('Deleting agents:', selectedAgents);
    };

    // Toggle agent selection
    const toggleAgentSelection = (agentId: number) => {
        setSelectedAgents((prev) =>
            prev.includes(agentId)
                ? prev.filter((id) => id !== agentId)
                : [...prev, agentId]
        );
    };

    // Handle create agent form submission
    const handleCreateAgent = async (data: any) => {
        try {
            console.log('Creating agent:', data);
            // Here you would typically make an API call to create the agent
            await new Promise((resolve) => setTimeout(resolve, 1000)); // Simulate API call
            setIsCreateModalOpen(false);
        } catch (error) {
            console.error('Error creating agent:', error);
        }
    };

    // Handle opening the details modal
    const handleOpenDetailsModal = (agent: typeof agents[0]) => {
        setSelectedAgent(agent);
        setIsDetailsModalOpen(true);
    };

    return (
        <AppShell>
            <div className="min-h-screen">
                {/* Navigation */}
                <header className="sticky top-0 z-50 w-full border-b border-white/10 bg-primary/50 backdrop-blur-xl">
                    <div className="container py-4">
                        <MainNav />
                    </div>
                </header>

                {/* Main Content */}
                <main className="container py-8">
                    {/* Header Section */}
                    <div className="flex items-center justify-between mb-8">
                        <div>
                            <h1 className="text-4xl font-bold mb-2">AI Agents</h1>
                            <p className="text-white/60">Manage and monitor your AI agents</p>
                        </div>

                        {/* Create Agent Modal */}
                        <Modal open={isCreateModalOpen} onOpenChange={setIsCreateModalOpen}>
                            <ModalTrigger asChild>
                                <Button>Create New Agent</Button>
                            </ModalTrigger>
                            <ModalContent>
                                <ModalHeader>
                                    <ModalTitle>Create New AI Agent</ModalTitle>
                                    <ModalDescription>Configure your new AI agent's settings and capabilities.</ModalDescription>
                                </ModalHeader>
                                <CreateAgentForm
                                    onSubmit={handleCreateAgent}
                                    onCancel={() => setIsCreateModalOpen(false)}
                                />
                            </ModalContent>
                        </Modal>
                    </div>

                    {/* Filters */}
                    <AgentFilters
                        onSearch={setSearchQuery}
                        onFilterStatus={setStatusFilter}
                        onFilterType={setTypeFilter}
                        onSort={setSortField}
                    />

                    {/* Agents Grid */}
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                        {filteredAgents.map((agent) => (
                            <Card
                                key={agent.id}
                                hover
                                className={cn(
                                    'transition-all duration-200',
                                    selectedAgents.includes(agent.id) && 'ring-2 ring-accent'
                                )}
                                onClick={() => toggleAgentSelection(agent.id)}
                            >
                                <CardHeader>
                                    <div className="flex items-center justify-between">
                                        <CardTitle>{agent.name}</CardTitle>
                                        <DropdownMenu>
                                            <DropdownMenuTrigger asChild>
                                                <CustomDropdownButton
                                                    variant="ghost"
                                                    className="h-8 w-8 p-0"
                                                    onClick={(e) => e.stopPropagation()}
                                                >
                                                    <span className="sr-only">Open menu</span>
                                                    <svg
                                                        className="h-4 w-4"
                                                        fill="none"
                                                        stroke="currentColor"
                                                        viewBox="0 0 24 24"
                                                    >
                                                        <path
                                                            strokeLinecap="round"
                                                            strokeLinejoin="round"
                                                            strokeWidth={2}
                                                            d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z"
                                                        />
                                                    </svg>
                                                </CustomDropdownButton>
                                            </DropdownMenuTrigger>
                                            <DropdownMenuContent onClick={(e: React.MouseEvent<HTMLDivElement>) => e.stopPropagation()}>
                                                <DropdownMenuItem>Edit Agent</DropdownMenuItem>
                                                <DropdownMenuItem onClick={() => handleOpenDetailsModal(agent)}>
                                                    View Details
                                                </DropdownMenuItem>
                                                <DropdownMenuItem className="text-red-500">
                                                    Delete Agent
                                                </DropdownMenuItem>
                                            </DropdownMenuContent>
                                        </DropdownMenu>
                                    </div>
                                    <CardDescription>Type: {agent.type}</CardDescription>
                                </CardHeader>
                                <CardContent>
                                    <div className="space-y-4">
                                        <div className="flex items-center justify-between">
                                            <span className="text-sm text-white/60">Status</span>
                                            <span
                                                className={cn(
                                                    'px-2 py-1 rounded-full text-xs font-medium',
                                                    agent.status === 'Active'
                                                        ? 'bg-green-500/20 text-green-400'
                                                        : 'bg-red-500/20 text-red-400'
                                                )}
                                            >
                                                {agent.status}
                                            </span>
                                        </div>
                                        <div className="flex items-center justify-between">
                                            <span className="text-sm text-white/60">Conversations</span>
                                            <span>{agent.conversations}</span>
                                        </div>
                                        <div className="flex items-center justify-between">
                                            <span className="text-sm text-white/60">Success Rate</span>
                                            <span>{agent.successRate}%</span>
                                        </div>
                                    </div>
                                </CardContent>
                            </Card>
                        ))}
                    </div>

                    {/* Bulk Actions */}
                    <AgentBulkActions
                        selectedCount={selectedAgents.length}
                        onActivate={handleBulkActivate}
                        onDeactivate={handleBulkDeactivate}
                        onDelete={handleBulkDelete}
                    />

                    {/* Agent Details Modal */}
                    <AgentDetailsModal
                        isOpen={isDetailsModalOpen}
                        onClose={() => setIsDetailsModalOpen(false)}
                        agent={selectedAgent}
                    />
                </main>
            </div>
        </AppShell>
    );
} 