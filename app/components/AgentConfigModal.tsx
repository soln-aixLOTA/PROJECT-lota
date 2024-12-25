'use client';

import { Agent } from '@/lib/stores/agentStore';
import { Dialog, Switch, Transition } from '@headlessui/react';
import { XMarkIcon } from '@heroicons/react/24/outline';
import { motion } from 'framer-motion';
import { Fragment, useState } from 'react';

interface AgentConfigModalProps {
    agent: Agent;
    isOpen: boolean;
    onClose: () => void;
    onSave: (configuration: Record<string, any>) => Promise<void>;
}

const configSections = [
    {
        id: 'general',
        title: 'General Settings',
        fields: [
            {
                id: 'language',
                label: 'Language',
                type: 'select',
                options: [
                    { value: 'en', label: 'English' },
                    { value: 'es', label: 'Spanish' },
                    { value: 'fr', label: 'French' },
                    { value: 'de', label: 'German' },
                ],
            },
            {
                id: 'timezone',
                label: 'Timezone',
                type: 'select',
                options: [
                    { value: 'UTC', label: 'UTC' },
                    { value: 'America/New_York', label: 'Eastern Time' },
                    { value: 'America/Los_Angeles', label: 'Pacific Time' },
                    { value: 'Europe/London', label: 'London' },
                ],
            },
        ],
    },
    {
        id: 'personality',
        title: 'Personality',
        fields: [
            {
                id: 'responseStyle',
                label: 'Response Style',
                type: 'select',
                options: [
                    { value: 'professional', label: 'Professional' },
                    { value: 'friendly', label: 'Friendly' },
                    { value: 'casual', label: 'Casual' },
                    { value: 'technical', label: 'Technical' },
                ],
            },
            {
                id: 'useEmoji',
                label: 'Use Emojis',
                type: 'toggle',
                description: 'Include emojis in responses when appropriate',
            },
        ],
    },
    {
        id: 'advanced',
        title: 'Advanced Settings',
        fields: [
            {
                id: 'maxResponseTime',
                label: 'Max Response Time (seconds)',
                type: 'number',
                min: 1,
                max: 30,
            },
            {
                id: 'useAutoLearning',
                label: 'Auto-Learning',
                type: 'toggle',
                description: 'Learn from conversations to improve responses',
            },
            {
                id: 'confidenceThreshold',
                label: 'Confidence Threshold (%)',
                type: 'number',
                min: 0,
                max: 100,
            },
        ],
    },
];

export default function AgentConfigModal({
    agent,
    isOpen,
    onClose,
    onSave,
}: AgentConfigModalProps) {
    const [config, setConfig] = useState(agent.configuration);
    const [isSaving, setIsSaving] = useState(false);

    const handleSave = async () => {
        setIsSaving(true);
        try {
            await onSave(config);
            onClose();
        } catch (error) {
            console.error('Failed to save configuration:', error);
        } finally {
            setIsSaving(false);
        }
    };

    const handleChange = (fieldId: string, value: any) => {
        setConfig((prev) => ({
            ...prev,
            [fieldId]: value,
        }));
    };

    return (
        <Transition appear show={isOpen} as={Fragment}>
            <Dialog as="div" className="relative z-50" onClose={onClose}>
                <Transition.Child
                    as={Fragment}
                    enter="ease-out duration-300"
                    enterFrom="opacity-0"
                    enterTo="opacity-100"
                    leave="ease-in duration-200"
                    leaveFrom="opacity-100"
                    leaveTo="opacity-0"
                >
                    <div className="fixed inset-0 bg-black/80" />
                </Transition.Child>

                <div className="fixed inset-0 overflow-y-auto">
                    <div className="flex min-h-full items-center justify-center p-4">
                        <Transition.Child
                            as={Fragment}
                            enter="ease-out duration-300"
                            enterFrom="opacity-0 scale-95"
                            enterTo="opacity-100 scale-100"
                            leave="ease-in duration-200"
                            leaveFrom="opacity-100 scale-100"
                            leaveTo="opacity-0 scale-95"
                        >
                            <Dialog.Panel className="w-full max-w-2xl glass-panel overflow-hidden">
                                <div className="p-6">
                                    <div className="flex items-center justify-between mb-6">
                                        <Dialog.Title className="text-xl font-semibold text-white">
                                            Configure {agent.name}
                                        </Dialog.Title>
                                        <button
                                            onClick={onClose}
                                            className="text-white/60 hover:text-white"
                                        >
                                            <XMarkIcon className="w-6 h-6" />
                                        </button>
                                    </div>

                                    <div className="space-y-8">
                                        {configSections.map((section) => (
                                            <motion.div
                                                key={section.id}
                                                initial={{ opacity: 0, y: 20 }}
                                                animate={{ opacity: 1, y: 0 }}
                                                transition={{ duration: 0.5 }}
                                            >
                                                <h3 className="text-lg font-medium text-white mb-4">
                                                    {section.title}
                                                </h3>
                                                <div className="space-y-4">
                                                    {section.fields.map((field) => (
                                                        <div key={field.id} className="flex items-center justify-between">
                                                            <div>
                                                                <label
                                                                    htmlFor={field.id}
                                                                    className="block text-sm font-medium text-white"
                                                                >
                                                                    {field.label}
                                                                </label>
                                                                {field.description && (
                                                                    <p className="text-sm text-white/60">
                                                                        {field.description}
                                                                    </p>
                                                                )}
                                                            </div>
                                                            <div className="ml-4">
                                                                {field.type === 'select' && (
                                                                    <select
                                                                        id={field.id}
                                                                        value={config[field.id] || ''}
                                                                        onChange={(e) =>
                                                                            handleChange(field.id, e.target.value)
                                                                        }
                                                                        className="input-field"
                                                                    >
                                                                        {field.options?.map((option) => (
                                                                            <option
                                                                                key={option.value}
                                                                                value={option.value}
                                                                            >
                                                                                {option.label}
                                                                            </option>
                                                                        ))}
                                                                    </select>
                                                                )}
                                                                {field.type === 'toggle' && (
                                                                    <Switch
                                                                        checked={config[field.id] || false}
                                                                        onChange={(checked) =>
                                                                            handleChange(field.id, checked)
                                                                        }
                                                                        className={`${config[field.id]
                                                                            ? 'bg-accent'
                                                                            : 'bg-white/10'
                                                                            } relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none`}
                                                                    >
                                                                        <span
                                                                            className={`${config[field.id]
                                                                                ? 'translate-x-6'
                                                                                : 'translate-x-1'
                                                                                } inline-block h-4 w-4 transform rounded-full bg-white transition-transform`}
                                                                        />
                                                                    </Switch>
                                                                )}
                                                                {field.type === 'number' && (
                                                                    <input
                                                                        type="number"
                                                                        id={field.id}
                                                                        min={field.min}
                                                                        max={field.max}
                                                                        value={config[field.id] || ''}
                                                                        onChange={(e) =>
                                                                            handleChange(
                                                                                field.id,
                                                                                parseInt(e.target.value)
                                                                            )
                                                                        }
                                                                        className="input-field w-24"
                                                                    />
                                                                )}
                                                            </div>
                                                        </div>
                                                    ))}
                                                </div>
                                            </motion.div>
                                        ))}
                                    </div>
                                </div>

                                <div className="border-t border-white/10 p-6 bg-white/5">
                                    <div className="flex justify-end space-x-4">
                                        <button
                                            onClick={onClose}
                                            className="button-secondary"
                                            disabled={isSaving}
                                        >
                                            Cancel
                                        </button>
                                        <button
                                            onClick={handleSave}
                                            className="button-primary"
                                            disabled={isSaving}
                                        >
                                            {isSaving ? (
                                                <div className="flex items-center space-x-2">
                                                    <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                                                    <span>Saving...</span>
                                                </div>
                                            ) : (
                                                'Save Changes'
                                            )}
                                        </button>
                                    </div>
                                </div>
                            </Dialog.Panel>
                        </Transition.Child>
                    </div>
                </div>
            </Dialog>
        </Transition>
    );
} 