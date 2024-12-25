'use client';

import { AgentTemplate } from '@/lib/stores/agentStore';
import { motion } from 'framer-motion';

interface AgentTemplateCardProps {
    template: AgentTemplate;
    onClick: () => void;
    isSelected?: boolean;
}

export default function AgentTemplateCard({
    template,
    onClick,
    isSelected = false,
}: AgentTemplateCardProps) {
    return (
        <motion.div
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
            className={`glass-panel p-6 cursor-pointer transition-all duration-300 ${isSelected ? 'ring-2 ring-accent bg-white/10' : 'hover:bg-white/5'
                }`}
            onClick={onClick}
        >
            {/* Header */}
            <div className="flex items-center space-x-4 mb-4">
                <div className="text-4xl">{template.icon}</div>
                <div>
                    <h3 className="text-lg font-semibold text-white">{template.name}</h3>
                    <span className="text-sm text-white/60">{template.category}</span>
                </div>
            </div>

            {/* Description */}
            <p className="text-white/80 mb-6">{template.description}</p>

            {/* Capabilities */}
            <div className="space-y-2">
                <h4 className="text-sm font-medium text-white/60">Capabilities</h4>
                <div className="flex flex-wrap gap-2">
                    {template.capabilities.map((capability) => (
                        <span
                            key={capability}
                            className="px-2 py-1 rounded-full bg-white/5 text-sm text-white/80"
                        >
                            {capability}
                        </span>
                    ))}
                </div>
            </div>

            {/* Selection Indicator */}
            {isSelected && (
                <motion.div
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    className="absolute top-3 right-3"
                >
                    <div className="w-6 h-6 rounded-full bg-accent flex items-center justify-center">
                        <svg
                            className="w-4 h-4 text-white"
                            fill="none"
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth="2"
                            viewBox="0 0 24 24"
                            stroke="currentColor"
                        >
                            <path d="M5 13l4 4L19 7" />
                        </svg>
                    </div>
                </motion.div>
            )}
        </motion.div>
    );
} 