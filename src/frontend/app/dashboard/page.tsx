export default function Dashboard() {
    return (
        <div className="container mx-auto px-4 py-8">
            <h1 className="text-4xl font-bold mb-8">Dashboard</h1>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <div className="bg-white dark:bg-gray-800 p-6 rounded-lg shadow">
                    <h2 className="text-2xl font-semibold mb-4">Bots</h2>
                    <p className="text-gray-600 dark:text-gray-300">Manage your AI agents</p>
                </div>
                <div className="bg-white dark:bg-gray-800 p-6 rounded-lg shadow">
                    <h2 className="text-2xl font-semibold mb-4">Analytics</h2>
                    <p className="text-gray-600 dark:text-gray-300">View performance metrics</p>
                </div>
                <div className="bg-white dark:bg-gray-800 p-6 rounded-lg shadow">
                    <h2 className="text-2xl font-semibold mb-4">Settings</h2>
                    <p className="text-gray-600 dark:text-gray-300">Configure your workspace</p>
                </div>
            </div>
        </div>
    );
} 