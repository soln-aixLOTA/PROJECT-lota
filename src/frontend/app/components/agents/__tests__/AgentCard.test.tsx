import { Agent, AgentStatus, AgentType } from '@/app/types/agent';
import { fireEvent, render, screen } from '@testing-library/react';
import { AgentCard } from '../AgentCard';

const mockAgent: Agent = {
    id: '1',
    name: 'Test Agent',
    description: 'Test Description',
    type: AgentType.READER,
    status: AgentStatus.ACTIVE,
    lastActive: '2024-01-01T00:00:00.000Z',
    configuration: {},
    capabilities: []
};

describe('AgentCard', () => {
    it('renders agent information correctly', () => {
        render(<AgentCard agent={mockAgent} />);

        expect(screen.getByText('Test Agent')).toBeInTheDocument();
        expect(screen.getByText('Test Description')).toBeInTheDocument();
        expect(screen.getByText(AgentType.READER)).toBeInTheDocument();
        expect(screen.getByText(AgentStatus.ACTIVE)).toBeInTheDocument();
        expect(screen.getByText('2023/12/31')).toBeInTheDocument();
    });

    it('renders "No description available" when description is missing', () => {
        const agentWithoutDesc = { ...mockAgent, description: '' };
        render(<AgentCard agent={agentWithoutDesc} />);

        expect(screen.getByText('No description available')).toBeInTheDocument();
    });

    it('shows edit button when onEdit prop is provided', () => {
        const onEdit = jest.fn();
        render(<AgentCard agent={mockAgent} onEdit={onEdit} />);

        const editButton = screen.getByText('Edit Agent');
        expect(editButton).toBeInTheDocument();

        fireEvent.click(editButton);
        expect(onEdit).toHaveBeenCalledTimes(1);
    });

    it('shows delete button when onDelete prop is provided', () => {
        const onDelete = jest.fn();
        render(<AgentCard agent={mockAgent} onDelete={onDelete} />);

        const deleteButton = screen.getByLabelText('Delete agent');
        expect(deleteButton).toBeInTheDocument();

        fireEvent.click(deleteButton);
        expect(onDelete).toHaveBeenCalledTimes(1);
    });

    it('does not show edit and delete buttons when handlers are not provided', () => {
        render(<AgentCard agent={mockAgent} />);

        expect(screen.queryByText('Edit Agent')).not.toBeInTheDocument();
        expect(screen.queryByLabelText('Delete agent')).not.toBeInTheDocument();
    });
}); 