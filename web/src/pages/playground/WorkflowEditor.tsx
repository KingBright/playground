import React, { useState } from 'react';
import { Card, Button, TextArea } from '../../components/ui';
import { FlowNode, StepTimeline } from '../../components/flow';
import { TerminalConsole } from '../../components/visualization';

const steps = [
  { id: '1', title: 'Initiate Process', description: 'Initialize workflow', status: 'success' as const, timestamp: '10:00:01 AM', icon: 'play_circle' },
  { id: '2', title: 'Load Data Models', description: 'Fetch agent configurations', status: 'success' as const, timestamp: '10:00:05 AM', icon: 'database' },
  { id: '3', title: 'Execute Logic Block', description: 'Running agent simulation', status: 'active' as const, timestamp: '10:01:20 AM', icon: 'memory' },
  { id: '4', title: 'Finalize Output', description: 'Pending completion', status: 'pending' as const, icon: 'check_circle' }
];

const terminalLines = [
  { level: 'info' as const, timestamp: '10:00:01', message: 'Initializing worker pool...' },
  { level: 'info' as const, timestamp: '10:00:02', message: 'Loaded 4 core modules.' },
  { level: 'warn' as const, timestamp: '10:00:05', message: "Cache miss for key 'user_prefs_v2'. Retrying..." },
  { level: 'info' as const, timestamp: '10:00:06', message: 'Data fetch completed (145ms).' },
  { level: 'info' as const, timestamp: '10:01:20', message: 'Processing logic block #442.' },
  { level: 'success' as const, timestamp: '10:01:21', message: 'Output generated successfully.' }
];

export const WorkflowEditor: React.FC = () => {
  const [code, setCode] = useState(`// Workflow Script Example
step("Initialize", () => {
  env.setState({ status: "running" });
});

step("Process Data", () => {
  const data = agent.invoke("DataProcessor", input);
  env.update("results", data);
});

step("Finalize", () => {
  env.setState({ status: "completed" });
});`);

  const [activeTab, setActiveTab] = useState<'editor' | 'visual'>('editor');

  return (
    <div className="flex flex-col gap-6 h-full">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-white text-2xl font-bold">Workflow Editor</h1>
          <p className="text-text-secondary text-sm">Design and debug agent workflows with step-by-step execution tracking.</p>
        </div>
        <div className="flex gap-2">
          <Button variant="secondary" icon="play_arrow">Run</Button>
          <Button variant="secondary" icon="pause">Pause</Button>
          <Button variant="primary" icon="save">Save</Button>
        </div>
      </div>

      {/* Main Content */}
      <div className="grid grid-cols-12 gap-6 flex-1 min-h-0">
        {/* Left: Editor */}
        <div className="col-span-12 lg:col-span-7 flex flex-col gap-4">
          <Card className="flex-1 flex flex-col overflow-hidden">
            <div className="flex border-b border-border-dark">
              {(['editor', 'visual'] as const).map((tab) => (
                <button
                  key={tab}
                  onClick={() => setActiveTab(tab)}
                  className={`px-4 py-3 text-sm font-medium transition-colors ${
                    activeTab === tab
                      ? 'text-white border-b-2 border-primary'
                      : 'text-text-secondary hover:text-white'
                  }`}
                >
                  {tab === 'editor' ? 'Code Editor' : 'Visual Flow'}
                </button>
              ))}
            </div>

            {activeTab === 'editor' ? (
              <div className="flex-1 p-4 bg-[#0d1117]">
                <TextArea
                  className="w-full h-full font-mono text-sm bg-transparent border-none resize-none focus:ring-0"
                  value={code}
                  onChange={(e) => setCode(e.target.value)}
                />
              </div>
            ) : (
              <div className="flex-1 p-6 overflow-y-auto">
                <div className="space-y-6">
                  <FlowNode
                    id="start"
                    title="Initialize"
                    description="Set up environment and load configurations"
                    status="completed"
                    onConfigure={() => {}}
                    onViewLogs={() => {}}
                  />
                  <FlowNode
                    id="process"
                    title="Process Data"
                    description="Execute agent logic and process inputs"
                    status="running"
                    onConfigure={() => {}}
                    onViewLogs={() => {}}
                  />
                  <FlowNode
                    id="output"
                    title="Generate Output"
                    description="Finalize results and output"
                    status="pending"
                    onConfigure={() => {}}
                    onViewLogs={() => {}}
                  />
                </div>
              </div>
            )}
          </Card>
        </div>

        {/* Right: Timeline & Console */}
        <div className="col-span-12 lg:col-span-5 flex flex-col gap-4">
          <Card className="p-4">
            <h3 className="text-white font-bold text-sm mb-4 flex items-center gap-2">
              <span className="material-symbols-outlined text-primary">timeline</span>
              Execution Timeline
            </h3>
            <StepTimeline steps={steps} />
          </Card>

          <Card className="flex-1 flex flex-col overflow-hidden">
            <h3 className="text-white font-bold text-sm mb-4 px-4 pt-4 flex items-center gap-2">
              <span className="material-symbols-outlined text-primary">terminal</span>
              Console Output
            </h3>
            <div className="flex-1 px-4 pb-4 overflow-hidden">
              <TerminalConsole lines={terminalLines} showPrompt />
            </div>
          </Card>
        </div>
      </div>
    </div>
  );
};
