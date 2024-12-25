'use client';

import { Agent } from '@/lib/stores/agentStore';
import {
    ArrowPathIcon,
    ChatBubbleLeftRightIcon,
    ClockIcon,
    HeartIcon,
    PauseCircleIcon,
    PlayCircleIcon,
    TrashIcon,
} from '@heroicons/react/24/outline';
import { motion } from 'framer-motion';

interface AgentCardProps {
    agent: Agent;
    onDelete: () => void;
    onToggleStatus: () => void;
    onClick: () => void;
}

export default function AgentCard({
    agent,
    onDelete,
    onToggleStatus,
    onClick,
}: AgentCardProps) {
    const statusColors = {
        online: 'bg-green-400',
        offline: 'bg-gray-400',
        training: 'bg-yellow-400',
        error: 'bg-red-400',
    };

    return (
        <motion.div
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
            className="glass-panel p-6 cursor-pointer hover:bg-white/5 transition-all duration-300"
            onClick={onClick}
        >
            {/* Header */}
            <div className="flex items-center justify-between mb-4">
                <div className="flex items-center space-x-3">
                    <div className={`w-3 h-3 rounded-full ${statusColors[agent.status]}`} />
                    <h3 className="text-lg font-semibold text-white">{agent.name}</h3>
                </div>
                <div className="flex items-center space-x-2">
                    <button
                        onClick={(e) => {
                            e.stopPropagation();
                            onToggleStatus();
                        }}
                        className="p-2 hover:bg-white/10 rounded-full transition-colors"
                    >
                        {agent.status === 'online' ? (
                            <PauseCircleIcon className="w-6 h-6 text-white/60" />
                        ) : (
                            <PlayCircleIcon className="w-6 h-6 text-white/60" />
                        )}
                    </button>
                    <button
                        onClick={(e) => {
                            e.stopPropagation();
                            onDelete();
                        }}
                        className="p-2 hover:bg-white/10 rounded-full transition-colors"
                    >
                        <TrashIcon className="w-6 h-6 text-white/60" />
                    </button>
                </div>
            </div>

            {/* Description */}
            <p className="text-white/60 text-sm mb-6">{agent.description}</p>

            {/* Metrics Grid */}
            <div className="grid grid-cols-2 gap-4">
                <div className="space-y-1">
                    <div className="flex items-center space-x-2 text-white/60">
                        <ChatBubbleLeftRightIcon className="w-4 h-4" />
                        <span className="text-sm">Conversations</span>
                    </div>
                    <p className="text-lg font-semibold text-white">
                        {agent.metrics.conversations.toLocaleString()}
                    </p>
                </div>

                <div className="space-y-1">
                    <div className="flex items-center space-x-2 text-white/60">
                        <ArrowPathIcon className="w-4 h-4" />
                        <span className="text-sm">Success Rate</span>
                    </div>
                    <p className="text-lg font-semibold text-white">
                        {agent.metrics.successRate}%
                    </p>
                </div>

                <div className="space-y-1">
                    <div className="flex items-center space-x-2 text-white/60">
                        <ClockIcon className="w-4 h-4" />
                        <span className="text-sm">Avg. Response</span>
                    </div>
                    <p className="text-lg font-semibold text-white">
                        {agent.metrics.averageResponseTime}s
                    </p>
                </div>

                <div className="space-y-1">
                    <div className="flex items-center space-x-2 text-white/60">
                        <HeartIcon className="w-4 h-4" />
                        <span className="text-sm">Satisfaction</span>
                    </div>
                    <p className="text-lg font-semibold text-white">
                        {agent.metrics.userSatisfaction}/5
                    </p>
                </div>
            </div>

            {/* Footer */}
            <div className="mt-6 pt-4 border-t border-white/10">
                <div className="flex items-center justify-between text-sm text-white/60">
                    <span>Created {new Date(agent.createdAt).toLocaleDateString()}</span>
                    <span>Last active {new Date(agent.lastActive).toLocaleDateString()}</span>
                </div>
            </div>
        </motion.div>
    );
} 