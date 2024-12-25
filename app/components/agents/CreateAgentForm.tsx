import { Button } from '@/components/ui/Button';
import { cn } from '@/lib/utils';
import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';
import * as z from 'zod';

// Validation schema
const createAgentSchema = z.object({
    name: z
        .string()
        .min(3, 'Name must be at least 3 characters')
        .max(50, 'Name must be less than 50 characters'),
    type: z.enum(['support', 'sales', 'analytics'], {
        required_error: 'Please select an agent type',
    }),
    description: z
        .string()
        .max(200, 'Description must be less than 200 characters')
        .optional(),
    initialPrompt: z
        .string()
        .min(10, 'Initial prompt must be at least 10 characters')
        .max(1000, 'Initial prompt must be less than 1000 characters'),
});

type CreateAgentFormData = z.infer<typeof createAgentSchema>;

interface CreateAgentFormProps {
    onSubmit: (data: CreateAgentFormData) => void;
    onCancel: () => void;
}

export function CreateAgentForm({ onSubmit, onCancel }: CreateAgentFormProps) {
    const {
        register,
        handleSubmit,
        formState: { errors, isSubmitting },
    } = useForm<CreateAgentFormData>({
        resolver: zodResolver(createAgentSchema),
        defaultValues: {
            type: 'support',
            description: '',
        },
    });

    return (
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
            {/* Agent Name */}
            <div className="space-y-2">
                <label className="text-sm font-medium">
                    Agent Name <span className="text-red-400">*</span>
                </label>
                <input
                    {...register('name')}
                    className={cn(
                        'w-full px-3 py-2 rounded-md bg-white/5 border border-white/10 focus:border-accent focus:ring-1 focus:ring-accent transition-colors',
                        errors.name && 'border-red-400 focus:border-red-400 focus:ring-red-400'
                    )}
                    placeholder="Enter agent name..."
                />
                {errors.name && (
                    <p className="text-sm text-red-400">{errors.name.message}</p>
                )}
            </div>

            {/* Agent Type */}
            <div className="space-y-2">
                <label className="text-sm font-medium">
                    Agent Type <span className="text-red-400">*</span>
                </label>
                <select
                    {...register('type')}
                    className={cn(
                        'w-full px-3 py-2 rounded-md bg-white/5 border border-white/10 focus:border-accent focus:ring-1 focus:ring-accent transition-colors',
                        errors.type && 'border-red-400 focus:border-red-400 focus:ring-red-400'
                    )}
                >
                    <option value="support">Customer Support</option>
                    <option value="sales">Sales</option>
                    <option value="analytics">Analytics</option>
                </select>
                {errors.type && (
                    <p className="text-sm text-red-400">{errors.type.message}</p>
                )}
            </div>

            {/* Description */}
            <div className="space-y-2">
                <label className="text-sm font-medium">Description</label>
                <textarea
                    {...register('description')}
                    className={cn(
                        'w-full px-3 py-2 rounded-md bg-white/5 border border-white/10 focus:border-accent focus:ring-1 focus:ring-accent transition-colors min-h-[80px] resize-y',
                        errors.description && 'border-red-400 focus:border-red-400 focus:ring-red-400'
                    )}
                    placeholder="Enter agent description..."
                />
                {errors.description && (
                    <p className="text-sm text-red-400">{errors.description.message}</p>
                )}
            </div>

            {/* Initial Prompt */}
            <div className="space-y-2">
                <label className="text-sm font-medium">
                    Initial Prompt <span className="text-red-400">*</span>
                </label>
                <textarea
                    {...register('initialPrompt')}
                    className={cn(
                        'w-full px-3 py-2 rounded-md bg-white/5 border border-white/10 focus:border-accent focus:ring-1 focus:ring-accent transition-colors min-h-[120px] resize-y',
                        errors.initialPrompt && 'border-red-400 focus:border-red-400 focus:ring-red-400'
                    )}
                    placeholder="Enter the initial prompt for your agent..."
                />
                {errors.initialPrompt && (
                    <p className="text-sm text-red-400">{errors.initialPrompt.message}</p>
                )}
            </div>

            {/* Form Actions */}
            <div className="flex justify-end space-x-2 pt-4">
                <Button
                    type="button"
                    variant="ghost"
                    onClick={onCancel}
                    disabled={isSubmitting}
                >
                    Cancel
                </Button>
                <Button type="submit" disabled={isSubmitting}>
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
                            Creating...
                        </>
                    ) : (
                        'Create Agent'
                    )}
                </Button>
            </div>
        </form>
    );
} 