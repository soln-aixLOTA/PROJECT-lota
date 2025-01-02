/// <reference types="jest" />
import { jest } from '@jest/globals';
import '@testing-library/jest-dom';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import AgentDetailsModal from '../AgentDetailsModal';

interface AgentFormData {
    name: string;
    type: string;
    description: string;
    config?: Record<string, any>;
}

// Mock the Modal component and its children
jest.mock('@/components/ui/Modal', () => ({
    __esModule: true,
    Modal: ({ children, open }: { children: React.ReactNode; open: boolean }) => (
        open ? children : null
    ),
    ModalContent: ({ children, ...props }: { children: React.ReactNode } & React.HTMLAttributes<HTMLDivElement>) => (
        <div role="dialog" data-testid="modal" {...props}>{children}</div>
    ),
    ModalHeader: ({ children, ...props }: { children: React.ReactNode } & React.HTMLAttributes<HTMLDivElement>) => (
        <div {...props}>{children}</div>
    ),
    ModalTitle: ({ children, ...props }: { children: React.ReactNode } & React.HTMLAttributes<HTMLDivElement>) => (
        <div {...props}>{children}</div>
    ),
    ModalDescription: ({ children, ...props }: { children: React.ReactNode } & React.HTMLAttributes<HTMLDivElement>) => (
        <div {...props}>{children}</div>
    ),
    ModalFooter: ({ children, ...props }: { children: React.ReactNode } & React.HTMLAttributes<HTMLDivElement>) => (
        <div {...props}>{children}</div>
    ),
}));

describe('AgentDetailsModal', () => {
    const mockOnSubmit = jest.fn((data: AgentFormData) => Promise.resolve());
    const mockOnClose = jest.fn();

    const defaultProps = {
        isOpen: true,
        onClose: mockOnClose,
        onSubmit: mockOnSubmit,
    };

    beforeEach(() => {
        jest.clearAllMocks();
    });

    it('renders correctly with default props', () => {
        render(<AgentDetailsModal {...defaultProps} />);

        // Check if modal components are rendered
        expect(screen.getByRole('dialog')).toBeInTheDocument();
        expect(screen.getByTestId('modal-content')).toBeInTheDocument();
        expect(screen.getByTestId('modal-header')).toBeInTheDocument();

        // Check if title and description are rendered
        expect(screen.getByText('Create New Agent')).toBeInTheDocument();
        expect(screen.getByText('Fill in the details to create a new agent')).toBeInTheDocument();

        // Check if form fields are rendered
        expect(screen.getByLabelText(/name/i)).toBeInTheDocument();
        expect(screen.getByLabelText(/type/i)).toBeInTheDocument();
        expect(screen.getByLabelText(/description/i)).toBeInTheDocument();

        // Check if buttons are rendered
        expect(screen.getByRole('button', { name: /cancel/i })).toBeInTheDocument();
        expect(screen.getByRole('button', { name: /save/i })).toBeInTheDocument();
    });

    it('renders correctly with initial data', () => {
        const initialData = {
            name: 'Test Agent',
            type: 'chat',
            description: 'Test Description',
        };

        render(<AgentDetailsModal {...defaultProps} initialData={initialData} />);

        // Check if title and description are updated
        expect(screen.getByText('Edit Agent')).toBeInTheDocument();
        expect(screen.getByText('Update the agent details below')).toBeInTheDocument();

        // Check if form fields are populated with initial data
        expect(screen.getByLabelText(/name/i)).toHaveValue(initialData.name);
        expect(screen.getByLabelText(/type/i)).toHaveValue(initialData.type);
        expect(screen.getByLabelText(/description/i)).toHaveValue(initialData.description);
    });

    it('handles form submission correctly', async () => {
        render(<AgentDetailsModal {...defaultProps} />);

        // Fill in form fields
        await userEvent.type(screen.getByLabelText(/name/i), 'New Agent');
        await userEvent.selectOptions(screen.getByLabelText(/type/i), 'chat');
        await userEvent.type(screen.getByLabelText(/description/i), 'New Description');

        // Submit form
        await userEvent.click(screen.getByRole('button', { name: /save/i }));

        // Wait for submission to complete
        await waitFor(() => {
            expect(mockOnSubmit).toHaveBeenCalledWith({
                name: 'New Agent',
                type: 'chat',
                description: 'New Description',
                config: {},
            });
        });

        // Check if modal is closed after submission
        expect(mockOnClose).toHaveBeenCalled();
    });

    it('handles form validation', async () => {
        render(<AgentDetailsModal {...defaultProps} />);

        // Try to submit form without filling required fields
        await userEvent.click(screen.getByRole('button', { name: /save/i }));

        // Check if form validation prevents submission
        expect(mockOnSubmit).not.toHaveBeenCalled();
    });

    it('handles cancel button click', async () => {
        render(<AgentDetailsModal {...defaultProps} />);

        await userEvent.click(screen.getByRole('button', { name: /cancel/i }));
        expect(mockOnClose).toHaveBeenCalled();
    });

    it('handles error during form submission', async () => {
        const mockError = new Error('Submission failed');
        const mockOnSubmitWithError = jest.fn((data: AgentFormData) => Promise.reject(mockError));

        const consoleSpy = jest.spyOn(console, 'error').mockImplementation(() => {});

        render(
            <AgentDetailsModal
                {...defaultProps}
                onSubmit={mockOnSubmitWithError}
            />
        );

        // Fill in form fields
        await userEvent.type(screen.getByLabelText(/name/i), 'New Agent');
        await userEvent.selectOptions(screen.getByLabelText(/type/i), 'chat');
        await userEvent.type(screen.getByLabelText(/description/i), 'New Description');

        // Submit form
        await userEvent.click(screen.getByRole('button', { name: /save/i }));

        // Wait for error to be handled
        await waitFor(() => {
            expect(mockOnSubmitWithError).toHaveBeenCalled();
            expect(consoleSpy).toHaveBeenCalledWith('Error submitting form:', mockError);
        });

        // Check if modal stays open after error
        expect(mockOnClose).not.toHaveBeenCalled();

        consoleSpy.mockRestore();
    });
}); 