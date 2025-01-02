'use client';

import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Label } from '@/components/ui/Label';
import { Modal, ModalContent, ModalDescription, ModalFooter, ModalHeader, ModalTitle } from '@/components/ui/Modal';
import React, { useState } from 'react';

interface AgentFormData {
    name: string;
    type: string;
    description: string;
    config?: Record<string, any>;
}

interface AgentDetailsModalProps {
    isOpen: boolean;
    onClose: () => void;
    onSubmit: (data: AgentFormData) => Promise<void>;
    initialData?: Partial<AgentFormData>;
}

const agentTypes = [
    { value: 'chat', label: 'Chat Agent' },
    { value: 'task', label: 'Task Agent' },
    { value: 'assistant', label: 'Assistant Agent' },
];

export default function AgentDetailsModal({
    isOpen,
    onClose,
    onSubmit,
    initialData,
}: AgentDetailsModalProps) {
    const [formData, setFormData] = useState<AgentFormData>({
        name: initialData?.name || '',
        type: initialData?.type || '',
        description: initialData?.description || '',
        config: initialData?.config || {},
    });
    const [isSubmitting, setIsSubmitting] = useState(false);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setIsSubmitting(true);
        try {
            await onSubmit(formData);
            onClose();
        } catch (error) {
            console.error('Error submitting form:', error);
        } finally {
            setIsSubmitting(false);
        }
    };

    const handleChange = (field: keyof AgentFormData, value: string) => {
        setFormData(prev => ({ ...prev, [field]: value }));
    };

    return (
        <Modal open={isOpen} onOpenChange={onClose}>
            <ModalContent data-testid="modal-content">
                <form onSubmit={handleSubmit}>
                    <ModalHeader data-testid="modal-header">
                        <ModalTitle data-testid="modal-title">
                            {initialData ? 'Edit Agent' : 'Create New Agent'}
                        </ModalTitle>
                        <ModalDescription data-testid="modal-description">
                            {initialData
                                ? 'Update the agent details below'
                                : 'Fill in the details to create a new agent'}
                        </ModalDescription>
                    </ModalHeader>

                    <div className="space-y-4 py-4">
                        <div className="space-y-2">
                            <Label htmlFor="name">Name</Label>
                            <Input
                                id="name"
                                placeholder="Enter agent name"
                                value={formData.name}
                                onChange={(e) => handleChange('name', e.target.value)}
                                required
                            />
                        </div>

                        <div className="space-y-2">
                            <Label htmlFor="type">Type</Label>
                            <select
                                id="type"
                                className="w-full rounded-md border border-input bg-background px-3 py-2"
                                value={formData.type}
                                onChange={(e) => handleChange('type', e.target.value)}
                                required
                            >
                                <option value="">Select agent type</option>
                                {agentTypes.map((type) => (
                                    <option key={type.value} value={type.value}>
                                        {type.label}
                                    </option>
                                ))}
                            </select>
                        </div>

                        <div className="space-y-2">
                            <Label htmlFor="description">Description</Label>
                            <textarea
                                id="description"
                                className="w-full rounded-md border border-input bg-background px-3 py-2"
                                placeholder="Enter agent description"
                                value={formData.description}
                                onChange={(e) => handleChange('description', e.target.value)}
                                required
                            />
                        </div>
                    </div>

                    <ModalFooter data-testid="modal-footer">
                        <Button
                            type="button"
                            onClick={onClose}
                            disabled={isSubmitting}
                            className="bg-gray-100 hover:bg-gray-200"
                        >
                            Cancel
                        </Button>
                        <Button
                            type="submit"
                            disabled={isSubmitting}
                            className="bg-blue-500 text-white hover:bg-blue-600"
                        >
                            {isSubmitting ? 'Saving...' : 'Save'}
                        </Button>
                    </ModalFooter>
                </form>
            </ModalContent>
        </Modal>
    );
} 