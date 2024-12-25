'use client';

import MetricsChart from '@/components/MetricsChart';
import Navigation from '@/components/Navigation';
import { useDashboardStore } from '@/lib/stores/dashboardStore';
import {
    ChartBarIcon,
    ChatBubbleLeftRightIcon,
    CpuChipIcon,
    UserGroupIcon
} from '@heroicons/react/24/outline';
import { motion } from 'framer-motion';
import { useEffect } from 'react';

const icons = {
    'Active Agents': CpuChipIcon,
    'Total Conversations': ChatBubbleLeftRightIcon,
    'Active Users': UserGroupIcon,
    'Success Rate': ChartBarIcon,
};

export default function Dashboard() {
    const {
        metrics,
        activities,
        selectedPeriod,
        selectedMetricIndex,
        isLoading,
        error,
        setSelectedPeriod,
        setSelectedMetricIndex,
        fetchDashboardData,
    } = useDashboardStore();

    useEffect(() => {
        fetchDashboardData();
    }, [fetchDashboardData]);

    const selectedMetric = metrics[selectedMetricIndex];

    if (error) {
        return (
            <div className="min-h-screen bg-primary flex items-center justify-center">
                <div className="text-white text-center">
                    <p className="text-xl">Error loading dashboard data</p>
                    <button
                        onClick={() => fetchDashboardData()}
                        className="button-primary mt-4"
                    >
                        Retry
                    </button>
                </div>
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-primary">
            <Navigation />

            <main className="pt-24 pb-8 px-4 sm:px-6 lg:px-8">
                <div className="max-w-7xl mx-auto">
                    {/* Header */}
                    <div className="md:flex md:items-center md:justify-between mb-8">
                        <motion.div
                            initial={{ opacity: 0, y: 20 }}
                            animate={{ opacity: 1, y: 0 }}
                            transition={{ duration: 0.5 }}
                            className="flex-1 min-w-0"
                        >
                            <h1 className="text-3xl font-display font-bold leading-7 text-white sm:text-4xl sm:truncate">
                                Dashboard
                            </h1>
                            <p className="mt-1 text-lg text-white/60">
                                Monitor and manage your AI agents in real-time
                            </p>
                        </motion.div>

                        <motion.div
                            initial={{ opacity: 0, x: -20 }}
                            animate={{ opacity: 1, x: 0 }}
                            transition={{ duration: 0.5, delay: 0.2 }}
                            className="mt-4 flex md:mt-0 md:ml-4"
                        >
                            <select
                                value={selectedPeriod}
                                onChange={(e) => setSelectedPeriod(e.target.value)}
                                className="input-field max-w-[150px]"
                            >
                                <option value="24h">Last 24 hours</option>
                                <option value="7d">Last 7 days</option>
                                <option value="30d">Last 30 days</option>
                                <option value="90d">Last 90 days</option>
                            </select>
                        </motion.div>
                    </div>

                    {/* Loading State */}
                    {isLoading && metrics.length === 0 ? (
                        <div className="flex items-center justify-center h-64">
                            <div className="text-white text-center">
                                <div className="w-8 h-8 border-2 border-accent border-t-transparent rounded-full animate-spin mx-auto mb-4" />
                                <p>Loading dashboard data...</p>
                            </div>
                        </div>
                    ) : (
                        <>
                            {/* Metrics Grid */}
                            <motion.div
                                initial={{ opacity: 0, y: 20 }}
                                animate={{ opacity: 1, y: 0 }}
                                transition={{ duration: 0.5, delay: 0.3 }}
                                className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4"
                            >
                                {metrics.map((metric, index) => {
                                    const Icon = icons[metric.name as keyof typeof icons];
                                    return (
                                        <motion.div
                                            key={metric.name}
                                            initial={{ opacity: 0, y: 20 }}
                                            animate={{ opacity: 1, y: 0 }}
                                            transition={{ duration: 0.5, delay: 0.4 + index * 0.1 }}
                                            className={`glass-panel p-6 hover:bg-white/10 transition-colors duration-300 cursor-pointer ${selectedMetricIndex === index ? 'ring-2 ring-accent' : ''
                                                }`}
                                            onClick={() => setSelectedMetricIndex(index)}
                                        >
                                            <div className="flex items-center justify-between mb-4">
                                                <div className="flex-1">
                                                    <dt className="text-sm font-medium text-white/60 truncate">
                                                        {metric.name}
                                                    </dt>
                                                    <dd className="mt-2 flex items-baseline">
                                                        <p className="text-2xl font-semibold text-white">
                                                            {metric.value}
                                                        </p>
                                                        <p className={`ml-2 text-sm font-medium text-green-400`}>
                                                            {metric.change}
                                                        </p>
                                                    </dd>
                                                </div>
                                                <div className="rounded-full bg-white/5 p-3">
                                                    <Icon
                                                        className="h-6 w-6 text-accent"
                                                        aria-hidden="true"
                                                    />
                                                </div>
                                            </div>
                                            <div className="h-16">
                                                <MetricsChart data={metric.data} height={64} />
                                            </div>
                                        </motion.div>
                                    );
                                })}
                            </motion.div>

                            {/* Detailed Chart */}
                            {selectedMetric && (
                                <motion.div
                                    initial={{ opacity: 0, y: 20 }}
                                    animate={{ opacity: 1, y: 0 }}
                                    transition={{ duration: 0.5, delay: 0.6 }}
                                    className="mt-8 glass-panel p-6"
                                >
                                    <h3 className="text-lg font-semibold text-white mb-4">
                                        {selectedMetric.name} - Detailed View
                                    </h3>
                                    <div className="h-64">
                                        <MetricsChart
                                            data={selectedMetric.data}
                                            height={256}
                                            animated={false}
                                        />
                                    </div>
                                </motion.div>
                            )}

                            {/* Recent Activity */}
                            <motion.div
                                initial={{ opacity: 0, y: 20 }}
                                animate={{ opacity: 1, y: 0 }}
                                transition={{ duration: 0.5, delay: 0.7 }}
                                className="mt-8"
                            >
                                <h2 className="text-xl font-display font-semibold text-white mb-6">
                                    Recent Activity
                                </h2>
                                <div className="glass-panel divide-y divide-white/10">
                                    {activities.map((activity) => (
                                        <div
                                            key={activity.id}
                                            className="p-4 flex items-center space-x-4 hover:bg-white/5 transition-colors"
                                        >
                                            <div className="flex-1">
                                                <p className="text-white">{activity.message}</p>
                                                <p className="text-sm text-white/60">{activity.timestamp}</p>
                                            </div>
                                        </div>
                                    ))}
                                </div>
                            </motion.div>
                        </>
                    )}
                </div>
            </main>
        </div>
    );
} 