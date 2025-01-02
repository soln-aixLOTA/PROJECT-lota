import { Button } from '@/components/ui/Button';
import { Card, CardContent, CardHeader } from '@/components/ui/Card';
import { Agent } from '@/types/agent';
import { XMarkIcon } from '@heroicons/react/24/outline';

interface AgentCardProps {
    agent: Agent;
    onEdit?: () => void;
    onDelete?: () => void;
}

export function AgentCard({ agent, onEdit, onDelete }: AgentCardProps) {
    const formatDate = (dateString: string) => {
        return new Date(dateString).toLocaleDateString();
    };

    return (
        <Card className="w-full">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <div className="flex flex-col">
                    <h3 className="text-lg font-semibold">{agent.name}</h3>
                    <p className="text-sm text-muted-foreground">
                        {agent.description || 'No description available'}
                    </p>
                </div>
                {onDelete && (
                    <Button
                        variant="ghost"
                        size="icon"
                        onClick={onDelete}
                        aria-label="Delete agent"
                    >
                        <XMarkIcon className="h-4 w-4" />
                    </Button>
                )}
            </CardHeader>
            <CardContent>
                <div className="grid gap-2">
                    <div className="flex items-center justify-between">
                        <span className="text-sm font-medium">Type:</span>
                        <span className="text-sm">{agent.type}</span>
                    </div>
                    <div className="flex items-center justify-between">
                        <span className="text-sm font-medium">Status:</span>
                        <span className="text-sm">{agent.status}</span>
                    </div>
                    <div className="flex items-center justify-between">
                        <span className="text-sm font-medium">Last Active:</span>
                        <span className="text-sm">{formatDate(agent.lastActive)}</span>
                    </div>
                    {onEdit && (
                        <Button
                            variant="outline"
                            size="sm"
                            className="mt-4"
                            onClick={onEdit}
                        >
                            Edit Agent
                        </Button>
                    )}
                </div>
            </CardContent>
        </Card>
    );
} 