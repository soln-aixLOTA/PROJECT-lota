'use client';

import { AgentTemplate } from '@/lib/types';
import { motion } from 'framer-motion';

interface AgentTemplateCardProps {
    template: AgentTemplate;
    onClick: () => void;
}

export default function AgentTemplateCard({ template, onClick }: AgentTemplateCardProps) {
    return (
        <motion.div
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
            className="bg-secondary/30 rounded-lg p-6 cursor-pointer"
            onClick={onClick}
        >
            <div className="flex items-center space-x-3 mb-4">
                <span className="text-2xl">{template.icon}</span>
                <h3 className="text-lg font-semibold">{template.name}</h3>
            </div>
            <p className="text-sm text-white/60 mb-4">{template.description}</p>
            <div className="flex flex-wrap gap-2">
                {template.capabilities.map((capability) => (
                    <span
                        key={capability}
                        className="px-2 py-1 bg-accent/10 text-accent rounded-full text-xs"
                    >
                        {capability}
                    </span>
                ))}
            </div>
        </motion.div>
    );
} 