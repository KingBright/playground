import re

with open('web/src/pages/mission-control/Dashboard.tsx', 'r') as f:
    content = f.read()

# Add WebSocket imports
content = content.replace("import { api } from '../../api';", "import { api, useWebSocket, createSystemWebSocket } from '../../api';")

# Add types
content = content.replace("interface DashboardProps {", """interface SystemStats {
  cpu_usage: number;
  memory_usage_bytes: number;
  memory_usage_percent: number;
  total_memory_bytes: number;
}

interface DashboardProps {""")

# Update Component Body
hook_code = """
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
"""

content = re.sub(
    r'  const \[stats, setStats\] = useState<StatCardData\[\]>\(defaultStats\);[\s\S]*?fetchData\(\);\n  \}, \[\]\);',
    hook_code.strip(),
    content
)

with open('web/src/pages/mission-control/Dashboard.tsx', 'w') as f:
    f.write(content)
