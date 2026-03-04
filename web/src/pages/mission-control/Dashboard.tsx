import React, { useEffect, useState } from 'react';
import { api, useWebSocket, createSystemWebSocket } from '../../api';
import { Card, StatCard, Badge } from '../../components/ui';
import type { Simulation, StatCardData } from '../../types';

interface SystemStats {
  cpu_usage: number;
  memory_usage_bytes: number;
  memory_usage_percent: number;
  total_memory_bytes: number;
}

interface DashboardProps {
  stats?: StatCardData[];
  simulations?: Simulation[];
}



const defaultSimulations: Simulation[] = [
  {
    id: '1',
    name: 'News Broadcast - Morning Edition',
    status: 'running',
    environment: 'News Studio',
    agents: ['Host-AI', 'Guest-AI', 'FactChecker'],
    startTime: '2024-01-15T07:00:00Z',
    progress: 65
  },
  {
    id: '2',
    name: 'Stock Market Simulation 1929',
    status: 'paused',
    environment: 'Trading Floor',
    agents: ['Quant-Bot', 'HistoryOracle'],
    startTime: '2024-01-14T14:30:00Z',
    progress: 42
  },
  {
    id: '3',
    name: 'Debate Championship Finals',
    status: 'completed',
    environment: 'Debate Hall',
    agents: ['Debater-Alpha', 'Debater-Beta', 'Moderator'],
    startTime: '2024-01-13T09:00:00Z',
    progress: 100
  }
];

export const Dashboard: React.FC<DashboardProps> = () => {
const [stats, setStats] = useState<StatCardData[]>([]);
  const [simulations, setSimulations] = useState<Simulation[]>(defaultSimulations);
  const [loading, setLoading] = useState(true);

  // System Stats WebSocket
  const { data: wsSystemStats } = useWebSocket<SystemStats>(
    () => createSystemWebSocket(),
    'system_stats'
  );

  useEffect(() => {
    const fetchData = async () => {
      try {
        const data = await api.dashboard.getStats();
        // Base stats from API
        const baseStats = data.dashboardStats;
        setStats(baseStats);
        setSimulations(data.activeSimulations);
      } catch (error) {
        console.error('Failed to fetch dashboard data:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, []);

  // Update stats with real-time websocket data
  useEffect(() => {
    if (wsSystemStats && stats.length > 0) {
      setStats(prevStats => {
        const newStats = [...prevStats];

        // Update System Status to show CPU
        newStats[1] = {
          ...newStats[1],
          value: `${wsSystemStats.cpu_usage.toFixed(1)}%`,
          label: 'CPU Usage',
          changeLabel: 'Engine Core Load',
          icon: 'speed'
        };

        // Update Memory Backend to show actual RAM
        const memMB = (wsSystemStats.memory_usage_bytes / 1024 / 1024).toFixed(0);
        newStats[2] = {
          ...newStats[2],
          value: `${memMB} MB`,
          label: 'RAM Usage',
          changeLabel: `${wsSystemStats.memory_usage_percent.toFixed(1)}% of System`,
          icon: 'memory'
        };

        return newStats;
      });
    }
  }, [wsSystemStats]);

  const getStatusBadge = (status: Simulation['status']) => {
    switch (status) {
      case 'running':
        return <Badge variant="info">Running</Badge>;
      case 'paused':
        return <Badge variant="warning">Paused</Badge>;
      case 'completed':
        return <Badge variant="success">Completed</Badge>;
      case 'error':
        return <Badge variant="error">Error</Badge>;
      default:
        return <Badge variant="default">Unknown</Badge>;
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-primary animate-pulse">Loading system data...</div>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-8 pb-10">
      {/* Welcome Banner */}
      <div className="relative w-full rounded-xl overflow-hidden min-h-[220px] flex items-end group shadow-2xl shadow-black/50">
        <div className="absolute inset-0 bg-gradient-to-br from-primary/30 to-accent-purple/30" />
        <div className="absolute inset-0 bg-gradient-to-t from-background-dark via-background-dark/60 to-transparent" />
        <div className="relative p-8 w-full">
          <h2 className="text-white text-3xl font-bold leading-tight mb-2">System Overview</h2>
          <p className="text-text-secondary max-w-2xl">
            Monitor active agent simulations and real-time knowledge graph expansion. Current operational load is optimal.
          </p>
        </div>
      </div>

      {/* Knowledge Supply Chain */}
      <section className="flex flex-col gap-4">
        <div className="flex items-center justify-between">
          <h3 className="text-white text-xl font-bold leading-tight">Knowledge Supply Chain</h3>
          <button className="text-primary text-sm font-medium hover:text-white transition-colors flex items-center">
            View Details <span className="material-symbols-outlined text-[16px] ml-1">arrow_forward</span>
          </button>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          {stats.map((stat, index) => (
            <StatCard key={index} data={stat} />
          ))}
        </div>
      </section>

      {/* Active Simulations */}
      <section className="flex flex-col gap-4">
        <div className="flex items-center justify-between">
          <h3 className="text-white text-xl font-bold leading-tight">Active Simulations</h3>
          <div className="flex gap-2">
            <button className="p-2 text-text-secondary hover:text-white rounded hover:bg-surface-dark transition-colors">
              <span className="material-symbols-outlined">filter_list</span>
            </button>
            <button className="p-2 text-text-secondary hover:text-white rounded hover:bg-surface-dark transition-colors">
              <span className="material-symbols-outlined">more_vert</span>
            </button>
          </div>
        </div>
        <Card>
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-border-dark">
                  <th className="text-left px-5 py-4 text-xs font-semibold text-text-secondary uppercase tracking-wider">Simulation</th>
                  <th className="text-left px-5 py-4 text-xs font-semibold text-text-secondary uppercase tracking-wider">Environment</th>
                  <th className="text-left px-5 py-4 text-xs font-semibold text-text-secondary uppercase tracking-wider">Agents</th>
                  <th className="text-left px-5 py-4 text-xs font-semibold text-text-secondary uppercase tracking-wider">Status</th>
                  <th className="text-left px-5 py-4 text-xs font-semibold text-text-secondary uppercase tracking-wider">Progress</th>
                </tr>
              </thead>
              <tbody>
                {simulations.map((sim) => (
                  <tr key={sim.id} className="border-b border-border-dark/50 hover:bg-surface-dark-light/50 transition-colors">
                    <td className="px-5 py-4">
                      <div className="flex flex-col">
                        <span className="text-white font-medium">{sim.name}</span>
                        <span className="text-text-secondary text-xs">Started {new Date(sim.startTime).toLocaleDateString()}</span>
                      </div>
                    </td>
                    <td className="px-5 py-4 text-text-secondary">{sim.environment}</td>
                    <td className="px-5 py-4">
                      <div className="flex gap-1">
                        {sim.agents.slice(0, 2).map((agent, idx) => (
                          <span key={idx} className="px-2 py-0.5 bg-surface-dark-light rounded text-xs text-text-secondary">
                            {agent}
                          </span>
                        ))}
                        {sim.agents.length > 2 && (
                          <span className="px-2 py-0.5 text-xs text-text-secondary">+{sim.agents.length - 2}</span>
                        )}
                      </div>
                    </td>
                    <td className="px-5 py-4">{getStatusBadge(sim.status)}</td>
                    <td className="px-5 py-4">
                      <div className="flex items-center gap-2">
                        <div className="flex-1 bg-border-dark rounded-full h-2 overflow-hidden">
                          <div
                            className={`h-full rounded-full ${sim.status === 'completed'
                              ? 'bg-emerald-500'
                              : sim.status === 'error'
                                ? 'bg-red-500'
                                : 'bg-primary'
                              }`}
                            style={{ width: `${sim.progress}%` }}
                          />
                        </div>
                        <span className="text-xs text-text-secondary w-10">{sim.progress}%</span>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>
      </section>
    </div>
  );
};
