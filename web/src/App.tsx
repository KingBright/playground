import React, { useState } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { SideNavBar } from './components/layout';
import { Icon } from './components/ui/Icon';
import type { NavItem } from './types';

// Pages
import { Dashboard } from './pages/mission-control/Dashboard';
import { KnowledgeManager } from './pages/brain/KnowledgeManager';
import { WorkflowEditor } from './pages/playground/WorkflowEditor';
import { AgentRegistry } from './pages/registry/AgentRegistry';
import { TaskScheduler } from './pages/scheduler/TaskScheduler';
// import { SystemLogs } from './pages/logs/SystemLogs';
import { Settings } from './pages/settings/Settings';

const navItems: NavItem[] = [
  { id: 'dashboard', label: 'Mission Control', icon: 'dashboard', path: '/dashboard' },
  { id: 'brain', label: 'External Brain', icon: 'psychology', path: '/brain' },
  { id: 'playground', label: 'Agent Playground', icon: 'smart_toy', path: '/playground' },
  { id: 'registry', label: 'Agent Registry', icon: 'apps', path: '/registry' },
  { id: 'scheduler', label: 'Task Scheduler', icon: 'schedule', path: '/scheduler' },
  { id: 'logs', label: 'System Logs', icon: 'description', path: '/logs' },
  { id: 'settings', label: 'Settings', icon: 'settings', path: '/settings' },
];

const App: React.FC = () => {
  const [activeItemId, setActiveItemId] = useState('dashboard');
  const [darkMode, setDarkMode] = useState(true);

  const handleNavItemClick = (item: NavItem) => {
    setActiveItemId(item.id);
  };

  return (
    <BrowserRouter>
      <div className={`${darkMode ? 'dark' : ''} h-screen overflow-hidden`}>
        <div className="flex h-full w-full bg-background-light dark:bg-background-dark">
          <SideNavBar
            items={navItems}
            activeItemId={activeItemId}
            onItemClick={handleNavItemClick}
            systemStatus={{ healthy: true, uptime: 98.4 }}
            user={{ name: 'Alex Chen', role: 'Architect' }}
            footerAction={{
              label: 'New Simulation',
              icon: 'add',
              onClick: () => console.log('New simulation')
            }}
          />

          <main className="flex-1 flex flex-col h-full overflow-hidden">
            {/* Header */}
            <header className="h-16 border-b border-border-dark flex items-center justify-between px-6 bg-background-dark/80 backdrop-blur-md sticky top-0 z-10">
              <div className="flex items-center text-text-secondary">
                <Icon name="home" className="mr-2" />
                <span className="mx-2 text-xs">/</span>
                <span className="text-sm font-medium text-white">
                  {navItems.find(item => item.id === activeItemId)?.label || 'Dashboard'}
                </span>
              </div>
              <div className="flex items-center gap-4">
                <button
                  onClick={() => setDarkMode(!darkMode)}
                  className="p-2 text-text-secondary hover:text-white rounded hover:bg-surface-dark transition-colors"
                >
                  <Icon name={darkMode ? 'light_mode' : 'dark_mode'} />
                </button>
                <div className="h-8 w-8 rounded-full bg-primary flex items-center justify-center text-white font-bold text-xs cursor-pointer">
                  AI
                </div>
              </div>
            </header>

            {/* Page Content */}
            <div className="flex-1 overflow-y-auto p-6">
              <div className="max-w-7xl mx-auto h-full">
                <Routes>
                  <Route path="/dashboard" element={<Dashboard />} />
                  <Route path="/brain" element={<KnowledgeManager />} />
                  <Route path="/playground" element={<WorkflowEditor />} />
                  <Route path="/registry" element={<AgentRegistry />} />
                  <Route path="/scheduler" element={<TaskScheduler />} />
                  {/* <Route path="/logs" element={<SystemLogs />} /> */}
                  <Route path="/settings" element={<Settings />} />
                  <Route path="/" element={<Navigate to="/dashboard" replace />} />
                </Routes>
              </div>
            </div>
          </main>
        </div>
      </div>
    </BrowserRouter>
  );
};

export default App;
