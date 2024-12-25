import { AppShell } from '@/components/layout/AppShell';
import { MainNav } from '@/components/navigation/MainNav';
import { Button } from '@/components/ui/Button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';

export default function DashboardPage() {
    return (
        <AppShell>
            <div className="min-h-screen">
                {/* Navigation */}
                <header className="sticky top-0 z-50 w-full border-b border-white/10 bg-primary/50 backdrop-blur-xl">
                    <div className="container py-4">
                        <MainNav />
                    </div>
                </header>

                {/* Main Content */}
                <main className="container py-8">
                    {/* Welcome Section */}
                    <div className="mb-8">
                        <h1 className="text-4xl font-bold mb-2">Welcome to LotaBots</h1>
                        <p className="text-white/60">Manage and monitor your AI agents from one central dashboard.</p>
                    </div>

                    {/* Stats Grid */}
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
                        <Card hover>
                            <CardHeader>
                                <CardTitle>Active Agents</CardTitle>
                                <CardDescription>Currently running AI agents</CardDescription>
                            </CardHeader>
                            <CardContent>
                                <div className="text-3xl font-bold">12</div>
                            </CardContent>
                        </Card>

                        <Card hover>
                            <CardHeader>
                                <CardTitle>Total Conversations</CardTitle>
                                <CardDescription>Across all agents today</CardDescription>
                            </CardHeader>
                            <CardContent>
                                <div className="text-3xl font-bold">1,234</div>
                            </CardContent>
                        </Card>

                        <Card hover>
                            <CardHeader>
                                <CardTitle>Success Rate</CardTitle>
                                <CardDescription>Average across all agents</CardDescription>
                            </CardHeader>
                            <CardContent>
                                <div className="text-3xl font-bold">98.5%</div>
                            </CardContent>
                        </Card>
                    </div>

                    {/* Recent Activity */}
                    <Card gradient className="mb-8">
                        <CardHeader>
                            <CardTitle>Recent Activity</CardTitle>
                            <CardDescription>Latest actions from your AI agents</CardDescription>
                        </CardHeader>
                        <CardContent>
                            <div className="space-y-4">
                                {[1, 2, 3].map((i) => (
                                    <div key={i} className="flex items-center justify-between p-4 rounded-lg bg-white/5">
                                        <div>
                                            <h4 className="font-medium">Agent #{i} Completed Task</h4>
                                            <p className="text-sm text-white/60">Successfully processed customer inquiry</p>
                                        </div>
                                        <span className="text-sm text-white/40">2 min ago</span>
                                    </div>
                                ))}
                            </div>
                        </CardContent>
                    </Card>

                    {/* Quick Actions */}
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                        <Card hover>
                            <CardHeader>
                                <CardTitle>Deploy New Agent</CardTitle>
                                <CardDescription>Create and configure a new AI agent</CardDescription>
                            </CardHeader>
                            <CardContent>
                                <Button>Create Agent</Button>
                            </CardContent>
                        </Card>

                        <Card hover>
                            <CardHeader>
                                <CardTitle>Performance Analytics</CardTitle>
                                <CardDescription>View detailed performance metrics</CardDescription>
                            </CardHeader>
                            <CardContent>
                                <Button variant="secondary">View Analytics</Button>
                            </CardContent>
                        </Card>
                    </div>
                </main>
            </div>
        </AppShell>
    );
} 