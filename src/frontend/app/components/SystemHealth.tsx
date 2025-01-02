'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { ArrowPathIcon } from '@heroicons/react/24/outline';
import { useQuery } from '@tanstack/react-query';

interface SystemMetrics {
    cpu_usage: number;
    memory_usage: number;
    disk_usage: number;
    network_latency: number;
    active_connections: number;
    requests_per_second: number;
}

async function fetchSystemMetrics(): Promise<SystemMetrics> {
    const response = await fetch('/api/system/metrics');
    if (!response.ok) {
        throw new Error('Failed to fetch system metrics');
    }
    return response.json();
}

export default function SystemHealth() {
    const { data, isLoading, error, refetch } = useQuery({
        queryKey: ['systemMetrics'],
        queryFn: fetchSystemMetrics,
        refetchInterval: 30000, // Refetch every 30 seconds
    });

    if (error) {
        return (
            <Card>
                <CardHeader>
                    <CardTitle>System Health</CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="text-red-500">
                        Error loading system metrics
                    </div>
                </CardContent>
            </Card>
        );
    }

    if (isLoading || !data) {
        return (
            <Card>
                <CardHeader>
                    <CardTitle>System Health</CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="animate-pulse space-y-4">
                        <div className="h-4 bg-secondary rounded w-3/4" />
                        <div className="h-4 bg-secondary rounded w-1/2" />
                        <div className="h-4 bg-secondary rounded w-2/3" />
                    </div>
                </CardContent>
            </Card>
        );
    }

    return (
        <Card>
            <CardHeader>
                <div className="flex justify-between items-center">
                    <CardTitle>System Health</CardTitle>
                    <button
                        onClick={() => refetch()}
                        className="p-2 hover:bg-secondary rounded-full transition-colors"
                    >
                        <ArrowPathIcon className="h-5 w-5" />
                    </button>
                </div>
            </CardHeader>
            <CardContent>
                <div className="space-y-4">
                    <div className="grid grid-cols-2 gap-4">
                        <div>
                            <div className="text-sm text-muted-foreground">CPU Usage</div>
                            <div className="text-2xl font-semibold">{data.cpu_usage}%</div>
                        </div>
                        <div>
                            <div className="text-sm text-muted-foreground">Memory Usage</div>
                            <div className="text-2xl font-semibold">{data.memory_usage}%</div>
                        </div>
                        <div>
                            <div className="text-sm text-muted-foreground">Disk Usage</div>
                            <div className="text-2xl font-semibold">{data.disk_usage}%</div>
                        </div>
                        <div>
                            <div className="text-sm text-muted-foreground">Network Latency</div>
                            <div className="text-2xl font-semibold">{data.network_latency}ms</div>
                        </div>
                    </div>
                    <div className="pt-4 border-t">
                        <div className="flex justify-between items-center">
                            <div>
                                <div className="text-sm text-muted-foreground">Active Connections</div>
                                <div className="text-2xl font-semibold">{data.active_connections}</div>
                            </div>
                            <div>
                                <div className="text-sm text-muted-foreground">Requests/Second</div>
                                <div className="text-2xl font-semibold">{data.requests_per_second}</div>
                            </div>
                        </div>
                    </div>
                </div>
            </CardContent>
        </Card>
    );
} 