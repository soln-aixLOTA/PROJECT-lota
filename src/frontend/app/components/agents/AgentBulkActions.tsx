'use client';

import { Button } from '@/components/ui/Button';
import { Modal, ModalContent, ModalDescription, ModalFooter, ModalHeader, ModalTitle, ModalTrigger } from '@/components/ui/Modal';
import { useState } from 'react';

interface AgentBulkActionsProps {
  selectedAgents: string[];
  onAction: (action: 'start' | 'stop' | 'delete', agentIds: string[]) => Promise<void>;
}

export default function AgentBulkActions({ selectedAgents, onAction }: AgentBulkActionsProps) {
  const [isDeleteModalOpen, setIsDeleteModalOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

  const handleAction = async (action: 'start' | 'stop' | 'delete') => {
    setIsLoading(true);
    try {
      await onAction(action, selectedAgents);
      if (action === 'delete') {
        setIsDeleteModalOpen(false);
      }
    } catch (error) {
      console.error(`Error performing bulk ${action}:`, error);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="flex items-center gap-2">
      <Button
        variant="outline"
        size="sm"
        onClick={() => handleAction('start')}
        disabled={selectedAgents.length === 0 || isLoading}
      >
        Start Selected
      </Button>
      <Button
        variant="outline"
        size="sm"
        onClick={() => handleAction('stop')}
        disabled={selectedAgents.length === 0 || isLoading}
      >
        Stop Selected
      </Button>
      <Modal open={isDeleteModalOpen} onOpenChange={setIsDeleteModalOpen}>
        <ModalTrigger asChild>
          <Button
            variant="destructive"
            size="sm"
            disabled={selectedAgents.length === 0 || isLoading}
          >
            Delete Selected
          </Button>
        </ModalTrigger>
        <ModalContent>
          <ModalHeader>
            <ModalTitle>Delete Agents</ModalTitle>
            <ModalDescription>
              Are you sure you want to delete {selectedAgents.length} selected agents? This action cannot be undone.
            </ModalDescription>
          </ModalHeader>
          <ModalFooter>
            <Button
              variant="outline"
              onClick={() => setIsDeleteModalOpen(false)}
              disabled={isLoading}
            >
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={() => handleAction('delete')}
              disabled={isLoading}
            >
              {isLoading ? 'Deleting...' : 'Delete'}
            </Button>
          </ModalFooter>
        </ModalContent>
      </Modal>
    </div>
  );
} 