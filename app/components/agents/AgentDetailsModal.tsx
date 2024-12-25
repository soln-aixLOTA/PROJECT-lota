import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Label } from '@/components/ui/Label';
import { Modal, ModalContent, ModalDescription, ModalFooter, ModalHeader, ModalTitle } from '@/components/ui/Modal';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/Select';
import { Textarea } from '@/components/ui/Textarea';
import { cn } from '@/lib/utils';
import { zodResolver } from '@hookform/resolvers/zod';
import React from 'react';
import { useForm } from 'react-hook-form';
import * as z from 'zod';

const editAgentSchema = z.object({
    name: z
        .string()
        .min(3, 'Name must be at least 3 characters')
        .max(50, 'Name must be less than 50 characters'),
    description: z
        .string()
        .max(200, 'Description must be less than 200 characters')
        .optional(),
    status: z.enum(['Active', 'Inactive']),
});

type EditAgentFormData = z.infer<typeof editAgentSchema>;

interface AgentDetailsModalProps {
    isOpen: boolean;
    onClose: () => void;
    agent: {
        id: number;
        name: string;
        status: string;
        type: string;
        conversations: number;
        successRate: number;
        description?: string;
        createdAt?: Date;
        lastActive?: Date;
        config?: Record<string, any>;
    } | null;
}

export function AgentDetailsModal({ isOpen, onClose, agent }: AgentDetailsModalProps) {
    const [isEditMode, setIsEditMode] = React.useState(false);

    const {
        register,
        handleSubmit,
        reset,
        setValue,
        formState: { errors, isSubmitting },
    } = useForm<EditAgentFormData>({
        resolver: zodResolver(editAgentSchema),
        defaultValues: {
            name: agent?.name || '',
            description: agent?.description || '',
            status: (agent?.status as 'Active' | 'Inactive') || 'Inactive',
        },
    });

    React.useEffect(() => {
        if (agent) {
            reset({
                name: agent.name,
                description: agent.description || '',
                status: agent.status as 'Active' | 'Inactive',
            });
        }
    }, [agent, reset]);

    const handleEdit = () => {
        setIsEditMode(true);
    };

    const handleCancelEdit = () => {
        setIsEditMode(false);
        if (agent) {
            reset({
                name: agent.name,
                description: agent.description || '',
                status: agent.status as 'Active' | 'Inactive',
            });
        }
    };

    const onSubmit = async (data: EditAgentFormData) => {
        try {
            console.log('Updating agent:', data);
            // Here you would typically make an API call to update the agent
            await new Promise((resolve) => setTimeout(resolve, 1000)); // Simulate API call
            setIsEditMode(false);
        } catch (error) {
            console.error('Error updating agent:', error);
        }
    };

    if (!agent) return null;

    const metrics = [
        {
            label: 'Total Conversations',
            value: agent.conversations.toLocaleString(),
            description: 'Total number of conversations handled',
        },
        {
            label: 'Success Rate',
            value: `${agent.successRate}%`,
            description: 'Percentage of successful interactions',
        },
        {
            label: 'Status',
            value: agent.status,
            description: 'Current operational status',
            className: cn(
                'px-2 py-1 rounded-full text-xs font-medium w-fit',
                agent.status === 'Active'
                    ? 'bg-green-500/20 text-green-400'
                    : 'bg-red-500/20 text-red-400'
            ),
        },
    ];

    return (
        <Modal open={isOpen} onOpenChange={onClose}>
            <ModalContent className="max-w-2xl">
                <form onSubmit={handleSubmit(onSubmit)}>
                    <ModalHeader>
                        <ModalTitle className="text-2xl">
                            {isEditMode ? 'Edit Agent' : agent.name}
                        </ModalTitle>
                        <ModalDescription className="text-white/60">
                            {agent.type} Agent • Created{' '}
                            {agent.createdAt?.toLocaleDateString('en-US', {
                                year: 'numeric',
                                month: 'long',
                                day: 'numeric',
                            })}
                        </ModalDescription>
                    </ModalHeader>

                    <div className="grid gap-6 py-4">
                        {isEditMode ? (
                            <div className="grid gap-4">
                                <div className="space-y-2">
                                    <Label htmlFor="name">
                                        Name <span className="text-red-400">*</span>
                                    </Label>
                                    <Input
                                        id="name"
                                        placeholder="Enter agent name"
                                        error={!!errors.name}
                                        {...register('name')}
                                    />
                                    {errors.name && (
                                        <p className="text-sm text-red-400">{errors.name.message}</p>
                                    )}
                                </div>

                                <div className="space-y-2">
                                    <Label htmlFor="description">Description</Label>
                                    <Textarea
                                        id="description"
                                        placeholder="Enter agent description"
                                        error={!!errors.description}
                                        {...register('description')}
                                    />
                                    {errors.description && (
                                        <p className="text-sm text-red-400">
                                            {errors.description.message}
                                        </p>
                                    )}
                                </div>

                                <div className="space-y-2">
                                    <Label htmlFor="status">
                                        Status <span className="text-red-400">*</span>
                                    </Label>
                                    <Select
                                        defaultValue={agent.status}
                                        onValueChange={(value) => setValue('status', value as 'Active' | 'Inactive')}
                                    >
                                        <SelectTrigger>
                                            <SelectValue placeholder="Select status" />
                                        </SelectTrigger>
                                        <SelectContent>
                                            <SelectItem value="Active">Active</SelectItem>
                                            <SelectItem value="Inactive">Inactive</SelectItem>
                                        </SelectContent>
                                    </Select>
                                    {errors.status && (
                                        <p className="text-sm text-red-400">{errors.status.message}</p>
                                    )}
                                </div>
                            </div>
                        ) : (
                            <>
                                {/* Key Metrics */}
                                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                                    {metrics.map((metric) => (
                                        <div
                                            key={metric.label}
                                            className="p-4 rounded-lg bg-white/5 border border-white/10 space-y-2"
                                        >
                                            <div className="text-sm text-white/60">{metric.label}</div>
                                            <div className={cn('text-xl font-semibold', metric.className)}>
                                                {metric.value}
                                            </div>
                                            <div className="text-xs text-white/40">
                                                {metric.description}
                                            </div>
                                        </div>
                                    ))}
                                </div>

                                {/* Description */}
                                {agent.description && (
                                    <div className="space-y-2">
                                        <h3 className="text-sm font-medium text-white/80">
                                            Description
                                        </h3>
                                        <p className="text-white/60">{agent.description}</p>
                                    </div>
                                )}

                                {/* Activity */}
                                <div className="space-y-2">
                                    <h3 className="text-sm font-medium text-white/80">
                                        Recent Activity
                                    </h3>
                                    <div className="p-4 rounded-lg bg-white/5 border border-white/10">
                                        <div className="flex items-center justify-between text-sm">
                                            <span className="text-white/60">Last Active</span>
                                            <span>
                                                {agent.lastActive?.toLocaleString('en-US', {
                                                    year: 'numeric',
                                                    month: 'long',
                                                    day: 'numeric',
                                                    hour: 'numeric',
                                                    minute: 'numeric',
                                                })}
                                            </span>
                                        </div>
                                    </div>
                                </div>

                                {/* Configuration */}
                                {agent.config && Object.keys(agent.config).length > 0 && (
                                    <div className="space-y-2">
                                        <h3 className="text-sm font-medium text-white/80">
                                            Configuration
                                        </h3>
                                        <div className="p-4 rounded-lg bg-white/5 border border-white/10 overflow-hidden">
                                            <pre className="text-sm text-white/60 overflow-x-auto">
                                                {JSON.stringify(agent.config, null, 2)}
                                            </pre>
                                        </div>
                                    </div>
                                )}
                            </>
                        )}
                    </div>

                    <ModalFooter className="space-x-2">
                        <Button
                            type="button"
                            variant="ghost"
                            onClick={isEditMode ? handleCancelEdit : onClose}
                            disabled={isSubmitting}
                        >
                            {isEditMode ? 'Cancel' : 'Close'}
                        </Button>
                        {isEditMode ? (
                            <Button
                                type="submit"
                                className="bg-blue-500 hover:bg-blue-600 text-white"
                                disabled={isSubmitting}
                            >
                                {isSubmitting ? (
                                    <>
                                        <svg
                                            className="animate-spin -ml-1 mr-2 h-4 w-4"
                                            xmlns="http://www.w3.org/2000/svg"
                                            fill="none"
                                            viewBox="0 0 24 24"
                                        >
                                            <circle
                                                className="opacity-25"
                                                cx="12"
                                                cy="12"
                                                r="10"
                                                stroke="currentColor"
                                                strokeWidth="4"
                                            />
                                            <path
                                                className="opacity-75"
                                                fill="currentColor"
                                                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                                            />
                                        </svg>
                                        Saving...
                                    </>
                                ) : (
                                    'Save Changes'
                                )}
                            </Button>
                        ) : (
                            <>
                                <Button
                                    type="button"
                                    variant="outline"
                                    className="border-yellow-500/50 text-yellow-500 hover:bg-yellow-500/10"
                                    onClick={handleEdit}
                                >
                                    Edit Agent
                                </Button>
                                <Button
                                    type="button"
                                    variant="outline"
                                    className="border-red-500/50 text-red-500 hover:bg-red-500/10"
                                >
                                    Deactivate Agent
                                </Button>
                            </>
                        )}
                    </ModalFooter>
                </form>
            </ModalContent>
        </Modal>
    );
} 