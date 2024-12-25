import { Button } from '@/components/ui/Button';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/Dropdown';

interface AgentFiltersProps {
    onSearch: (query: string) => void;
    onFilterStatus: (status: string) => void;
    onFilterType: (type: string) => void;
    onSort: (field: string) => void;
}

export function AgentFilters({
    onSearch,
    onFilterStatus,
    onFilterType,
    onSort,
}: AgentFiltersProps) {
    return (
        <div className="flex flex-col sm:flex-row gap-4 mb-6">
            {/* Search Bar */}
            <div className="relative flex-1">
                <input
                    type="text"
                    placeholder="Search agents..."
                    onChange={(e) => onSearch(e.target.value)}
                    className="w-full px-4 py-2 pl-10 bg-white/5 border border-white/10 rounded-lg focus:border-accent focus:ring-1 focus:ring-accent"
                />
                <svg
                    className="absolute left-3 top-2.5 h-5 w-5 text-white/40"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                >
                    <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                    />
                </svg>
            </div>

            {/* Filter by Status */}
            <DropdownMenu>
                <DropdownMenuTrigger asChild>
                    <Button variant="outline" className="min-w-[120px]">
                        Status
                        <svg
                            className="ml-2 h-4 w-4"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                strokeWidth={2}
                                d="M19 9l-7 7-7-7"
                            />
                        </svg>
                    </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent>
                    <DropdownMenuItem onClick={() => onFilterStatus('all')}>
                        All
                    </DropdownMenuItem>
                    <DropdownMenuItem onClick={() => onFilterStatus('active')}>
                        Active
                    </DropdownMenuItem>
                    <DropdownMenuItem onClick={() => onFilterStatus('inactive')}>
                        Inactive
                    </DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>

            {/* Filter by Type */}
            <DropdownMenu>
                <DropdownMenuTrigger asChild>
                    <Button variant="outline" className="min-w-[120px]">
                        Type
                        <svg
                            className="ml-2 h-4 w-4"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                strokeWidth={2}
                                d="M19 9l-7 7-7-7"
                            />
                        </svg>
                    </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent>
                    <DropdownMenuItem onClick={() => onFilterType('all')}>
                        All
                    </DropdownMenuItem>
                    <DropdownMenuItem onClick={() => onFilterType('support')}>
                        Support
                    </DropdownMenuItem>
                    <DropdownMenuItem onClick={() => onFilterType('sales')}>
                        Sales
                    </DropdownMenuItem>
                    <DropdownMenuItem onClick={() => onFilterType('analytics')}>
                        Analytics
                    </DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>

            {/* Sort By */}
            <DropdownMenu>
                <DropdownMenuTrigger asChild>
                    <Button variant="outline" className="min-w-[120px]">
                        Sort By
                        <svg
                            className="ml-2 h-4 w-4"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                strokeWidth={2}
                                d="M19 9l-7 7-7-7"
                            />
                        </svg>
                    </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent>
                    <DropdownMenuItem onClick={() => onSort('name')}>
                        Name
                    </DropdownMenuItem>
                    <DropdownMenuItem onClick={() => onSort('conversations')}>
                        Conversations
                    </DropdownMenuItem>
                    <DropdownMenuItem onClick={() => onSort('successRate')}>
                        Success Rate
                    </DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>
        </div>
    );
} 