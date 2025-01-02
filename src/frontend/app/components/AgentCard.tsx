'use client';

import { Agent } from '@/lib/types';
import { cn } from '@/lib/utils';
import { CheckCircleIcon, ClockIcon, XCircleIcon } from '@heroicons/react/24/outline';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from './ui/Card';

interface AgentCardProps {
    agent: Agent;
    onClick?: () => void;
}

export default function AgentCard({ agent, onClick }: AgentCardProps) {
    const getStatusIcon = () => {
        switch (agent.status) {
            case 'Active':
                return <CheckCircleIcon className="h-5 w-5 text-green-500" />;
            case 'Inactive':
                return <XCircleIcon className="h-5 w-5 text-red-500" />;
            default:
                return <ClockIcon className="h-5 w-5 text-gray-500" />;
        }
    };

    return (
        <Card 
            className="hover:shadow-lg transition-shadow cursor-pointer"
            onClick={onClick}
        >
            <CardHeader>
                <div className="flex items-center justify-between">
                    <div>
                        <CardTitle>{agent.name}</CardTitle>
                        <CardDescription>{agent.type}</CardDescription>
                    </div>
                    {getStatusIcon()}
                </div>
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
    );
} 