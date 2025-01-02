'use client';

import { cn } from '@/lib/utils';

interface Model {
    id: string;
    name: string;
    status: string;
    description: string;
    version: string;
    accuracy: number;
}

interface ModelCardProps {
    model: Model;
    selected: boolean;
    onSelect: () => void;
}

export default function ModelCard({ model, selected, onSelect }: ModelCardProps) {
    const getStatusColor = (status: string) => {
        switch (status.toLowerCase()) {
            case 'active':
                return 'bg-green-100 text-green-800';
            case 'training':
                return 'bg-yellow-100 text-yellow-800';
            case 'error':
                return 'bg-red-100 text-red-800';
            default:
                return 'bg-gray-100 text-gray-800';
        }
    };

    return (
        <div
            className={cn(
                'h-full flex flex-col rounded-lg border bg-card text-card-foreground shadow-sm transition-all',
                selected && 'ring-2 ring-primary',
                'hover:shadow-md cursor-pointer'
            )}
            onClick={onSelect}
        >
            <div className="p-6 flex-grow">
                <div className="flex justify-between items-center mb-4">
                    <h3 className="text-lg font-semibold">{model.name}</h3>
                    <span className={cn(
                        'px-2 py-1 rounded-full text-xs font-medium',
                        getStatusColor(model.status)
                    )}>
                        {model.status}
                    </span>
                </div>
                <p className="text-sm text-muted-foreground mb-4">{model.description}</p>
                <div className="space-y-2">
                    <div className="flex justify-between text-sm">
                        <span>Version</span>
                        <span>{model.version}</span>
                    </div>
                    <div className="flex justify-between text-sm">
                        <span>Accuracy</span>
                        <span>{model.accuracy}%</span>
                    </div>
                    <div className="w-full bg-secondary h-2 rounded-full overflow-hidden">
                        <div
                            className="bg-primary h-full transition-all"
                            style={{ width: `${model.accuracy}%` }}
                        />
                    </div>
                </div>
            </div>
        </div>
    );
} 