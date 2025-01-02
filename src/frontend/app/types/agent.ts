export enum AgentType {
    READER = 'READER',
    ANALYZER = 'ANALYZER',
    DEVELOPER = 'DEVELOPER',
    TESTER = 'TESTER',
    REVIEWER = 'REVIEWER',
    ARCHITECT = 'ARCHITECT',
    SECURITY = 'SECURITY',
    PERFORMANCE = 'PERFORMANCE',
    DOCUMENTATION = 'DOCUMENTATION',
    DEVOPS = 'DEVOPS'
}

export enum AgentStatus {
    ACTIVE = 'ACTIVE',
    INACTIVE = 'INACTIVE',
    BUSY = 'BUSY',
    ERROR = 'ERROR'
}

export interface Agent {
    id: string;
    name: string;
    description?: string;
    type: AgentType;
    status: AgentStatus;
    lastActive: string;
    configuration?: Record<string, any>;
    capabilities: string[];
} 