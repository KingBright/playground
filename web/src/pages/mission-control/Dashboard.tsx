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

  // Update stats with real-time websocket data, blending the backend's real numbers into the UI styling
  useEffect(() => {
    if (wsSystemStats && stats.length > 0) {
      setStats(prevStats => {
        const newStats = [...prevStats];

        // Ensure proper icons and styling are always applied even with dynamic data

        // Documents Processed
        if (newStats[0]) {
          newStats[0].icon = 'description';
          newStats[0].iconColor = 'bg-[#1152d4]';
        }

        // Graph Nodes (using System Status CPU for the moment if we lack graph data)
        if (wsSystemStats.cpu_usage !== undefined && wsSystemStats.cpu_usage !== null && newStats[1]) {
          newStats[1] = {
            ...newStats[1],
            value: `${Number(wsSystemStats.cpu_usage).toFixed(1)}%`,
            label: 'CPU Usage',
            changeLabel: 'Engine Core Load',
            icon: 'speed',
            iconColor: 'bg-purple-600'
          };
        }

        // Memory usage
        if (wsSystemStats.memory_usage_bytes !== undefined && wsSystemStats.memory_usage_percent !== undefined && newStats[2]) {
          const memMB = (Number(wsSystemStats.memory_usage_bytes) / 1024 / 1024).toFixed(0);
          newStats[2] = {
            ...newStats[2],
            value: `${memMB} MB`,
            label: 'RAM Usage',
            changeLabel: `${Number(wsSystemStats.memory_usage_percent).toFixed(1)}% of System`,
            icon: 'memory',
            iconColor: 'bg-orange-600'
          };
        }

        // Vector DB Storage
        if (newStats[3]) {
          newStats[3].icon = 'database';
          newStats[3].iconColor = 'bg-teal-600';
        }

        return newStats;
      });
    } else if (stats.length === 0 && !loading) {
        // Fallback styling if API completely failed
        setStats([
          {
            label: 'Documents Processed',
            value: '0',
            change: 0,
            trend: 'up',
            icon: 'description',
            iconColor: 'bg-[#1152d4]'
          },
          {
            label: 'CPU Usage',
            value: '0%',
            changeLabel: 'Loading...',
            icon: 'speed',
            iconColor: 'bg-purple-600'
          },
          {
            label: 'RAM Usage',
            value: '0 MB',
            changeLabel: 'Loading...',
            icon: 'memory',
            iconColor: 'bg-orange-600'
          },
          {
            label: 'Vector DB Storage',
            value: '0%',
            changeLabel: 'Initializing',
            icon: 'database',
            iconColor: 'bg-teal-600'
          }
        ]);
    }
  }, [wsSystemStats, loading]);

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
        <div className="absolute inset-0 bg-gradient-to-r from-[#0d1b32] to-[#12223a]" />
        {/* Added some decorative circles to match screenshot globe feel */}
        <div className="absolute inset-0 overflow-hidden flex items-center justify-center opacity-40 mix-blend-screen pointer-events-none">
          <div className="w-[600px] h-[600px] rounded-full border border-blue-500/30 absolute"></div>
          <div className="w-[500px] h-[500px] rounded-full border border-blue-400/20 absolute"></div>
          <div className="w-[400px] h-[400px] rounded-full border border-teal-500/20 absolute"></div>
          {/* Light flare effect */}
          <div className="w-96 h-96 bg-blue-500/20 rounded-full blur-[100px] absolute left-10"></div>
        </div>

        <div className="relative p-10 w-full z-10 flex flex-col justify-center h-full">
          <h2 className="text-white text-4xl font-bold leading-tight mb-3">System Overview</h2>
          <p className="text-[#92a4c9] text-base max-w-2xl leading-relaxed">
            Monitor active agent simulations and real-time knowledge graph expansion. Current<br/>
            operational load is optimal.
          </p>
        </div>
      </div>

      {/* Knowledge Supply Chain */}
      <section className="flex flex-col gap-5 mt-2">
        <div className="flex items-center justify-between">
          <h3 className="text-white text-2xl font-bold leading-tight">Knowledge Supply Chain</h3>
          <button className="text-blue-500 text-sm font-medium hover:text-white transition-colors flex items-center">
            View Details <span className="material-symbols-outlined text-[18px] ml-1">arrow_forward</span>
          </button>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-5">
          {stats.map((stat, index) => (
            <StatCard key={index} data={stat} />
          ))}
        </div>
      </section>

      {/* Active Simulations */}
      <section className="flex flex-col gap-5 mt-4">
        <div className="flex items-center justify-between">
          <h3 className="text-white text-2xl font-bold leading-tight">Active Simulations</h3>
          <div className="flex gap-2">
            <button className="p-2 text-[#92a4c9] hover:text-white rounded hover:bg-[#1a2333] transition-colors">
              <span className="material-symbols-outlined text-[20px]">filter_list</span>
            </button>
            <button className="p-2 text-[#92a4c9] hover:text-white rounded hover:bg-[#1a2333] transition-colors">
              <span className="material-symbols-outlined text-[20px]">more_vert</span>
            </button>
          </div>
        </div>

        <div className="w-full bg-[#161e2e] border border-[#232f48] rounded-xl overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-[#232f48] bg-[#161e2e]">
                  <th className="text-left px-6 py-4 text-xs font-bold text-[#92a4c9] uppercase tracking-wider w-[25%]">Simulation ID</th>
                  <th className="text-left px-6 py-4 text-xs font-bold text-[#92a4c9] uppercase tracking-wider w-[20%]">Agent Type</th>
                  <th className="text-left px-6 py-4 text-xs font-bold text-[#92a4c9] uppercase tracking-wider w-[20%]">Environment</th>
                  <th className="text-left px-6 py-4 text-xs font-bold text-[#92a4c9] uppercase tracking-wider w-[15%]">Status</th>
                  <th className="text-left px-6 py-4 text-xs font-bold text-[#92a4c9] uppercase tracking-wider w-[10%]">Runtime</th>
                  <th className="text-left px-6 py-4 text-xs font-bold text-[#92a4c9] uppercase tracking-wider w-[10%]">Load</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-[#232f48]/50">
                {simulations.map((sim, index) => (
                  <tr key={sim.id} className="hover:bg-[#1a2333] transition-colors">
                    <td className="px-6 py-4">
                      <div className="flex items-center gap-3">
                        {index % 4 === 0 ? <span className="material-symbols-outlined text-blue-500 text-[18px]">science</span> :
                         index % 4 === 1 ? <span className="material-symbols-outlined text-purple-500 text-[18px]">code</span> :
                         index % 4 === 2 ? <span className="material-symbols-outlined text-orange-500 text-[18px]">bar_chart</span> :
                         <span className="material-symbols-outlined text-teal-500 text-[18px]">shield</span>}
                        <span className="text-white font-semibold" title={sim.name}>SIM-{sim.id.substring(0, 4).toUpperCase()}</span>
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      {sim.agents && sim.agents.length > 0 ? (
                        <span className={`px-3 py-1 rounded text-xs font-medium ${
                          index % 4 === 0 ? 'bg-blue-500/20 text-blue-400' :
                          index % 4 === 1 ? 'bg-purple-500/20 text-purple-400' :
                          index % 4 === 2 ? 'bg-orange-500/20 text-orange-400' :
                          'bg-teal-500/20 text-teal-400'
                        }`}>
                          {sim.agents[0]}
                        </span>
                      ) : (
                        <span className="px-3 py-1 bg-slate-500/20 text-slate-400 rounded text-xs font-medium">None</span>
                      )}
                    </td>
                    <td className="px-6 py-4 text-[#92a4c9]">{sim.environment}</td>
                    <td className="px-6 py-4">
                      <div className="flex items-center gap-2">
                        <span className={`w-2 h-2 rounded-full ${
                          sim.status === 'running' ? 'bg-green-500' :
                          sim.status === 'paused' ? 'bg-yellow-500' :
                          sim.status === 'completed' ? 'bg-blue-500' : 'bg-red-500'
                        }`}></span>
                        <span className="text-white capitalize">{sim.status === 'completed' ? 'Completed' : sim.status}</span>
                      </div>
                    </td>
                    <td className="px-6 py-4 text-[#92a4c9] font-mono text-xs">
                       {/* Calculate a mock duration from startTime for display since real duration isn't in API yet */}
                       {(() => {
                          const start = new Date(sim.startTime).getTime();
                          const now = new Date().getTime();
                          const diff = isNaN(start) ? 0 : Math.max(0, now - start);
                          const h = Math.floor(diff / 3600000).toString().padStart(2, '0');
                          const m = Math.floor((diff % 3600000) / 60000).toString().padStart(2, '0');
                          const s = Math.floor((diff % 60000) / 1000).toString().padStart(2, '0');
                          return `${h}:${m}:${s}`;
                       })()}
                    </td>
                    <td className="px-6 py-4">
                      <div className="flex items-center gap-3">
                        <div className="w-16 h-1.5 bg-[#232f48] rounded-full overflow-hidden">
                          <div
                            className={`h-full rounded-full ${
                              sim.status === 'completed' ? 'bg-emerald-500' :
                              sim.status === 'error' ? 'bg-red-500' : 'bg-blue-500'
                            }`}
                            style={{ width: `${sim.progress}%` }}
                          ></div>
                        </div>
                        <span className="text-[#92a4c9] text-xs">{sim.progress}%</span>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </section>
    </div>
  );
};
