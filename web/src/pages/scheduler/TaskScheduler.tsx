import React, { useEffect, useState } from 'react';
import { api } from '../../api';
import { Card, Button, Badge } from '../../components/ui';
import type { ScheduledTask } from '../../types';

const defaultTasks: ScheduledTask[] = [
  {
    id: '1',
    name: 'Daily Knowledge Maintenance',
    type: 'scheduled',
    status: 'active',
    schedule: '0 2 * * *',
    lastRun: '2024-01-14T02:00:00Z',
    nextRun: '2024-01-15T02:00:00Z'
  },
  {
    id: '2',
    name: 'News Broadcast Preparation',
    type: 'scheduled',
    status: 'active',
    schedule: '30 6 * * *',
    lastRun: '2024-01-15T06:30:00Z',
    nextRun: '2024-01-16T06:30:00Z'
  },
  {
    id: '3',
    name: 'Vector Index Optimization',
    type: 'manual',
    status: 'paused',
    lastRun: '2024-01-10T14:00:00Z'
  }
];

export const TaskScheduler: React.FC = () => {
  const [tasks, setTasks] = useState<ScheduledTask[]>(defaultTasks);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const data = await api.synergy.getTasks();
        setTasks(data.tasks);
      } catch (error) {
        console.error('Failed to fetch tasks:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, []);


  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-primary animate-pulse">Loading scheduled tasks...</div>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-6 h-full">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-white text-2xl font-bold">Task Scheduler</h1>
          <p className="text-text-secondary text-sm">
            Manage scheduled tasks and automation workflows.
          </p>
        </div>
        <Button variant="primary" icon="add">
          Create Task
        </Button>
      </div>

      {/* Tasks List */}
      <Card>
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-border-dark">
                <th className="text-left px-5 py-4 text-xs font-semibold text-text-secondary uppercase tracking-wider">
                  Task
                </th>
                <th className="text-left px-5 py-4 text-xs font-semibold text-text-secondary uppercase tracking-wider">
                  Type
                </th>
                <th className="text-left px-5 py-4 text-xs font-semibold text-text-secondary uppercase tracking-wider">
                  Schedule
                </th>
                <th className="text-left px-5 py-4 text-xs font-semibold text-text-secondary uppercase tracking-wider">
                  Last Run
                </th>
                <th className="text-left px-5 py-4 text-xs font-semibold text-text-secondary uppercase tracking-wider">
                  Next Run
                </th>
                <th className="text-left px-5 py-4 text-xs font-semibold text-text-secondary uppercase tracking-wider">
                  Status
                </th>
                <th className="text-left px-5 py-4 text-xs font-semibold text-text-secondary uppercase tracking-wider">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody>
              {tasks.map((task) => (
                <tr key={task.id} className="border-b border-border-dark/50 hover:bg-surface-dark-light/50">
                  <td className="px-5 py-4">
                    <span className="text-white font-medium">{task.name}</span>
                  </td>
                  <td className="px-5 py-4">
                    <Badge variant="info">{task.type}</Badge>
                  </td>
                  <td className="px-5 py-4 text-text-secondary font-mono text-xs">
                    {task.schedule || 'Manual'}
                  </td>
                  <td className="px-5 py-4 text-text-secondary text-sm">
                    {task.lastRun ? new Date(task.lastRun).toLocaleString() : '-'}
                  </td>
                  <td className="px-5 py-4 text-text-secondary text-sm">
                    {task.nextRun ? new Date(task.nextRun).toLocaleString() : '-'}
                  </td>
                  <td className="px-5 py-4">
                    <Badge variant={task.status === 'active' ? 'success' : 'warning'}>
                      {task.status}
                    </Badge>
                  </td>
                  <td className="px-5 py-4">
                    <div className="flex gap-2">
                      <Button variant="ghost" size="sm" icon="play_arrow" />
                      <Button variant="ghost" size="sm" icon="pause" />
                      <Button variant="ghost" size="sm" icon="edit" />
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Card>
    </div>
  );
};
