import { Button } from '@/components/ui/Button';
import { Modal, ModalContent, ModalDescription, ModalFooter, ModalHeader, ModalTitle, ModalTrigger } from '@/components/ui/Modal';

interface AgentBulkActionsProps {
    selectedCount: number;
    onActivate: () => void;
    onDeactivate: () => void;
    onDelete: () => void;
}

export function AgentBulkActions({
    selectedCount,
    onActivate,
    onDeactivate,
    onDelete,
}: AgentBulkActionsProps) {
    if (selectedCount === 0) return null;

    return (
        <div className="fixed bottom-6 left-1/2 transform -translate-x-1/2 flex items-center gap-2 px-4 py-2 bg-secondary/90 backdrop-blur-xl border border-white/10 rounded-lg shadow-lg z-50">
            <span className="text-sm text-white/60">
                {selectedCount} agent{selectedCount !== 1 ? 's' : ''} selected
            </span>

            <div className="h-4 w-px bg-white/10 mx-2" />

            <Button
                variant="ghost"
                size="sm"
                onClick={onActivate}
                className="text-green-400 hover:text-green-300 hover:bg-green-400/10"
            >
                Activate
            </Button>

            <Button
                variant="ghost"
                size="sm"
                onClick={onDeactivate}
                className="text-yellow-400 hover:text-yellow-300 hover:bg-yellow-400/10"
            >
                Deactivate
            </Button>

            <Modal>
                <ModalTrigger asChild>
                    <Button
                        variant="ghost"
                        size="sm"
                        className="text-red-400 hover:text-red-300 hover:bg-red-400/10"
                    >
                        Delete
                    </Button>
                </ModalTrigger>
                <ModalContent>
                    <ModalHeader>
                        <ModalTitle>Delete Selected Agents</ModalTitle>
                        <ModalDescription>
                            Are you sure you want to delete {selectedCount} selected agent
                            {selectedCount !== 1 ? 's' : ''}? This action cannot be undone.
                        </ModalDescription>
                    </ModalHeader>
                    <ModalFooter>
                        <Button variant="ghost">Cancel</Button>
                        <Button
                            onClick={onDelete}
                            className="bg-red-500 hover:bg-red-600 text-white"
                        >
                            Delete
                        </Button>
                    </ModalFooter>
                </ModalContent>
            </Modal>
        </div>
    );
} 