import { client } from './client';
export {
    WebSocketClient,
    createSessionWebSocket,
    createMissionsWebSocket,
    useWebSocket,
    type WebSocketMessage
} from './websocket';
import type {
    Simulation,
    Agent,
    KnowledgeSlice,
    ScheduledTask,
    StatCardData
} from '../types';

// Types matching Backend Responses
interface SystemStatsResponse {
    dashboardStats: StatCardData[];
    activeSimulations: Simulation[];
}

interface KnowledgeListResponse {
    slices: KnowledgeSlice[];
}

interface SimulationListResponse {
    sessions: Simulation[];
}

interface AgentListResponse {
    agents: Agent[];
}

interface TaskListResponse {
    tasks: ScheduledTask[];
}

export const api = {
    dashboard: {
        getStats: () => client.get<SystemStatsResponse>('/dashboard/stats'),
    },
    brain: {
        getKnowledgeSlices: () => client.get<KnowledgeListResponse>('/brain/knowledge'),
    },
    engine: {
        getSessions: () => client.get<SimulationListResponse>('/engine/sessions'),
        getEnvironments: () => client.get<{ environments: { id: string; name: string }[] }>('/engine/environments'),
    },
    synergy: {
        getAgents: () => client.get<AgentListResponse>('/synergy/agents'),
        getTasks: () => client.get<TaskListResponse>('/synergy/tasks'),
    },
};
