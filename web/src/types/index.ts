// Navigation types
export interface NavItem {
  id: string;
  label: string;
  icon: string;
  path: string;
  children?: NavItem[];
}

// Agent types
export interface Agent {
  id: string;
  name: string;
  type: 'universal' | 'local';
  description: string;
  capabilities: string[];
  status: 'active' | 'inactive' | 'error';
  version: string;
  icon?: string;
}

// Simulation types
export interface Simulation {
  id: string;
  name: string;
  status: 'running' | 'paused' | 'completed' | 'error';
  environment: string;
  agents: string[];
  startTime: string;
  progress: number;
}

// Knowledge types
export interface KnowledgeSlice {
  id: string;
  name: string;
  nodeCount: number;
  status: 'active' | 'inactive';
  lastUpdated: string;
  tags: string[];
}

export interface GraphNode {
  id: string;
  label: string;
  type: string;
  properties: Record<string, unknown>;
}

export interface GraphEdge {
  id: string;
  from: string;
  to: string;
  label: string;
}

// Flow types
export interface FlowNode {
  id: string;
  type: string;
  label: string;
  status: 'running' | 'completed' | 'error' | 'pending';
  description?: string;
  metadata?: Record<string, unknown>;
}

export interface FlowStep {
  id: string;
  title: string;
  description: string;
  status: 'success' | 'active' | 'pending';
  timestamp?: string;
  icon?: string;
}

// Task types
export interface ScheduledTask {
  id: string;
  name: string;
  type: 'manual' | 'scheduled' | 'event';
  status: 'active' | 'paused' | 'completed';
  schedule?: string;
  lastRun?: string;
  nextRun?: string;
}

// Stats types
export interface StatCardData {
  label: string;
  value: string | number;
  change?: number;
  changeLabel?: string;
  icon: string;
  iconColor: string;
  trend?: 'up' | 'down' | 'stable';
}

// System types
export interface SystemStatus {
  healthy: boolean;
  uptime: number;
  cpuUsage: number;
  memoryUsage: number;
  storageUsage: number;
}
