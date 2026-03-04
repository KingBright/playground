import React, { useState } from 'react';
import { Card, Button, Input } from '../../components/ui';
import { Icon } from '../../components/ui/Icon';

interface SettingsSection {
  id: string;
  title: string;
  icon: string;
  description: string;
}

const settingsSections: SettingsSection[] = [
  { id: 'general', title: 'General', icon: 'settings', description: 'Basic application settings' },
  { id: 'api', title: 'API Configuration', icon: 'api', description: 'Backend API endpoints and keys' },
  { id: 'storage', title: 'Storage', icon: 'storage', description: 'Database and storage backends' },
  { id: 'llm', title: 'LLM Providers', icon: 'psychology', description: 'Language model configuration' },
  { id: 'agents', title: 'Agent Defaults', icon: 'smart_toy', description: 'Default agent settings' },
  { id: 'notifications', title: 'Notifications', icon: 'notifications', description: 'Alert and notification preferences' },
];

export const Settings: React.FC = () => {
  const [activeSection, setActiveSection] = useState('general');
  const [settings, setSettings] = useState({
    // General
    darkMode: true,
    language: 'en',
    timezone: 'UTC',
    autoSave: true,
    // API
    apiBaseUrl: '/api',
    apiTimeout: 30000,
    retryAttempts: 3,
    // Storage
    vectorDbProvider: 'in-memory',
    graphDbProvider: 'in-memory',
    cacheProvider: 'in-memory',
    rawStoragePath: '/tmp/agent-playground',
    // LLM
    llmProvider: 'openai',
    llmModel: 'gpt-4',
    llmTemperature: 0.7,
    llmMaxTokens: 4096,
    // Agents
    defaultAgentTimeout: 60000,
    maxAgentsPerSession: 10,
    enableOracle: true,
    // Notifications
    enableNotifications: true,
    notifyOnError: true,
    notifyOnWarning: true,
    notifyOnComplete: false,
  });

  const handleToggle = (key: keyof typeof settings) => {
    setSettings(prev => ({ ...prev, [key]: !prev[key] }));
  };

  const handleChange = (key: keyof typeof settings, value: string | number) => {
    setSettings(prev => ({ ...prev, [key]: value }));
  };

  const saveSettings = () => {
    // In real implementation, this would save to backend
    console.log('Saving settings:', settings);
    alert('Settings saved successfully!');
  };

  const renderGeneralSettings = () => (
    <div className="space-y-6">
      <div className="flex items-center justify-between py-3 border-b border-border-dark">
        <div>
          <div className="text-white font-medium">Dark Mode</div>
          <div className="text-text-secondary text-sm">Use dark theme for the interface</div>
        </div>
        <button
          onClick={() => handleToggle('darkMode')}
          className={`w-12 h-6 rounded-full transition-colors ${
            settings.darkMode ? 'bg-primary' : 'bg-gray-600'
          }`}
        >
          <div className={`w-5 h-5 rounded-full bg-white shadow transform transition-transform ${
            settings.darkMode ? 'translate-x-6' : 'translate-x-0.5'
          }`} />
        </button>
      </div>

      <div className="flex items-center justify-between py-3 border-b border-border-dark">
        <div>
          <div className="text-white font-medium">Language</div>
          <div className="text-text-secondary text-sm">Interface language</div>
        </div>
        <select
          value={settings.language}
          onChange={(e) => handleChange('language', e.target.value)}
          className="bg-surface-dark-light border border-border-dark rounded px-3 py-1.5 text-white text-sm"
        >
          <option value="en">English</option>
          <option value="zh">中文</option>
          <option value="ja">日本語</option>
        </select>
      </div>

      <div className="flex items-center justify-between py-3 border-b border-border-dark">
        <div>
          <div className="text-white font-medium">Timezone</div>
          <div className="text-text-secondary text-sm">Display timezone for timestamps</div>
        </div>
        <select
          value={settings.timezone}
          onChange={(e) => handleChange('timezone', e.target.value)}
          className="bg-surface-dark-light border border-border-dark rounded px-3 py-1.5 text-white text-sm"
        >
          <option value="UTC">UTC</option>
          <option value="Asia/Shanghai">Asia/Shanghai</option>
          <option value="America/New_York">America/New_York</option>
          <option value="Europe/London">Europe/London</option>
        </select>
      </div>

      <div className="flex items-center justify-between py-3 border-b border-border-dark">
        <div>
          <div className="text-white font-medium">Auto Save</div>
          <div className="text-text-secondary text-sm">Automatically save changes</div>
        </div>
        <button
          onClick={() => handleToggle('autoSave')}
          className={`w-12 h-6 rounded-full transition-colors ${
            settings.autoSave ? 'bg-primary' : 'bg-gray-600'
          }`}
        >
          <div className={`w-5 h-5 rounded-full bg-white shadow transform transition-transform ${
            settings.autoSave ? 'translate-x-6' : 'translate-x-0.5'
          }`} />
        </button>
      </div>
    </div>
  );

  const renderApiSettings = () => (
    <div className="space-y-6">
      <div className="py-3 border-b border-border-dark">
        <div className="text-white font-medium mb-2">API Base URL</div>
        <Input
          value={settings.apiBaseUrl}
          onChange={(e) => handleChange('apiBaseUrl', e.target.value)}
          placeholder="/api"
        />
        <div className="text-text-secondary text-xs mt-1">Base URL for all API requests</div>
      </div>

      <div className="py-3 border-b border-border-dark">
        <div className="text-white font-medium mb-2">Request Timeout (ms)</div>
        <Input
          type="number"
          value={settings.apiTimeout}
          onChange={(e) => handleChange('apiTimeout', parseInt(e.target.value))}
          placeholder="30000"
        />
        <div className="text-text-secondary text-xs mt-1">Maximum time to wait for API responses</div>
      </div>

      <div className="py-3 border-b border-border-dark">
        <div className="text-white font-medium mb-2">Retry Attempts</div>
        <Input
          type="number"
          value={settings.retryAttempts}
          onChange={(e) => handleChange('retryAttempts', parseInt(e.target.value))}
          placeholder="3"
        />
        <div className="text-text-secondary text-xs mt-1">Number of retry attempts for failed requests</div>
      </div>
    </div>
  );

  const renderStorageSettings = () => (
    <div className="space-y-6">
      <div className="flex items-center justify-between py-3 border-b border-border-dark">
        <div>
          <div className="text-white font-medium">Vector Database</div>
          <div className="text-text-secondary text-sm">Backend for vector storage</div>
        </div>
        <select
          value={settings.vectorDbProvider}
          onChange={(e) => handleChange('vectorDbProvider', e.target.value)}
          className="bg-surface-dark-light border border-border-dark rounded px-3 py-1.5 text-white text-sm"
        >
          <option value="in-memory">In-Memory</option>
          <option value="qdrant">Qdrant</option>
          <option value="milvus">Milvus</option>
        </select>
      </div>

      <div className="flex items-center justify-between py-3 border-b border-border-dark">
        <div>
          <div className="text-white font-medium">Graph Database</div>
          <div className="text-text-secondary text-sm">Backend for graph storage</div>
        </div>
        <select
          value={settings.graphDbProvider}
          onChange={(e) => handleChange('graphDbProvider', e.target.value)}
          className="bg-surface-dark-light border border-border-dark rounded px-3 py-1.5 text-white text-sm"
        >
          <option value="in-memory">In-Memory</option>
          <option value="neo4j">Neo4j</option>
        </select>
      </div>

      <div className="flex items-center justify-between py-3 border-b border-border-dark">
        <div>
          <div className="text-white font-medium">Cache Provider</div>
          <div className="text-text-secondary text-sm">Backend for hot memory cache</div>
        </div>
        <select
          value={settings.cacheProvider}
          onChange={(e) => handleChange('cacheProvider', e.target.value)}
          className="bg-surface-dark-light border border-border-dark rounded px-3 py-1.5 text-white text-sm"
        >
          <option value="in-memory">In-Memory</option>
          <option value="redis">Redis</option>
        </select>
      </div>

      <div className="py-3 border-b border-border-dark">
        <div className="text-white font-medium mb-2">Raw Storage Path</div>
        <Input
          value={settings.rawStoragePath}
          onChange={(e) => handleChange('rawStoragePath', e.target.value)}
          placeholder="/tmp/agent-playground"
        />
        <div className="text-text-secondary text-xs mt-1">Path for raw data archive storage</div>
      </div>
    </div>
  );

  const renderLLMSettings = () => (
    <div className="space-y-6">
      <div className="flex items-center justify-between py-3 border-b border-border-dark">
        <div>
          <div className="text-white font-medium">LLM Provider</div>
          <div className="text-text-secondary text-sm">AI model provider</div>
        </div>
        <select
          value={settings.llmProvider}
          onChange={(e) => handleChange('llmProvider', e.target.value)}
          className="bg-surface-dark-light border border-border-dark rounded px-3 py-1.5 text-white text-sm"
        >
          <option value="openai">OpenAI</option>
          <option value="anthropic">Anthropic</option>
          <option value="ollama">Ollama</option>
          <option value="mock">Mock (Testing)</option>
        </select>
      </div>

      <div className="py-3 border-b border-border-dark">
        <div className="text-white font-medium mb-2">Model</div>
        <Input
          value={settings.llmModel}
          onChange={(e) => handleChange('llmModel', e.target.value)}
          placeholder="gpt-4"
        />
        <div className="text-text-secondary text-xs mt-1">Model identifier to use</div>
      </div>

      <div className="py-3 border-b border-border-dark">
        <div className="text-white font-medium mb-2">Temperature</div>
        <div className="flex items-center gap-4">
          <input
            type="range"
            min="0"
            max="2"
            step="0.1"
            value={settings.llmTemperature}
            onChange={(e) => handleChange('llmTemperature', parseFloat(e.target.value))}
            className="flex-1"
          />
          <span className="text-white text-sm w-12">{settings.llmTemperature}</span>
        </div>
        <div className="text-text-secondary text-xs mt-1">Controls randomness in responses (0-2)</div>
      </div>

      <div className="py-3 border-b border-border-dark">
        <div className="text-white font-medium mb-2">Max Tokens</div>
        <Input
          type="number"
          value={settings.llmMaxTokens}
          onChange={(e) => handleChange('llmMaxTokens', parseInt(e.target.value))}
          placeholder="4096"
        />
        <div className="text-text-secondary text-xs mt-1">Maximum tokens in responses</div>
      </div>
    </div>
  );

  const renderAgentSettings = () => (
    <div className="space-y-6">
      <div className="py-3 border-b border-border-dark">
        <div className="text-white font-medium mb-2">Default Agent Timeout (ms)</div>
        <Input
          type="number"
          value={settings.defaultAgentTimeout}
          onChange={(e) => handleChange('defaultAgentTimeout', parseInt(e.target.value))}
          placeholder="60000"
        />
        <div className="text-text-secondary text-xs mt-1">Default timeout for agent operations</div>
      </div>

      <div className="py-3 border-b border-border-dark">
        <div className="text-white font-medium mb-2">Max Agents Per Session</div>
        <Input
          type="number"
          value={settings.maxAgentsPerSession}
          onChange={(e) => handleChange('maxAgentsPerSession', parseInt(e.target.value))}
          placeholder="10"
        />
        <div className="text-text-secondary text-xs mt-1">Maximum number of agents in a single session</div>
      </div>

      <div className="flex items-center justify-between py-3 border-b border-border-dark">
        <div>
          <div className="text-white font-medium">Enable Oracle Protocol</div>
          <div className="text-text-secondary text-sm">Allow agents to request help from Universal Agents</div>
        </div>
        <button
          onClick={() => handleToggle('enableOracle')}
          className={`w-12 h-6 rounded-full transition-colors ${
            settings.enableOracle ? 'bg-primary' : 'bg-gray-600'
          }`}
        >
          <div className={`w-5 h-5 rounded-full bg-white shadow transform transition-transform ${
            settings.enableOracle ? 'translate-x-6' : 'translate-x-0.5'
          }`} />
        </button>
      </div>
    </div>
  );

  const renderNotificationSettings = () => (
    <div className="space-y-6">
      <div className="flex items-center justify-between py-3 border-b border-border-dark">
        <div>
          <div className="text-white font-medium">Enable Notifications</div>
          <div className="text-text-secondary text-sm">Show system notifications</div>
        </div>
        <button
          onClick={() => handleToggle('enableNotifications')}
          className={`w-12 h-6 rounded-full transition-colors ${
            settings.enableNotifications ? 'bg-primary' : 'bg-gray-600'
          }`}
        >
          <div className={`w-5 h-5 rounded-full bg-white shadow transform transition-transform ${
            settings.enableNotifications ? 'translate-x-6' : 'translate-x-0.5'
          }`} />
        </button>
      </div>

      <div className="flex items-center justify-between py-3 border-b border-border-dark">
        <div>
          <div className="text-white font-medium">Notify on Errors</div>
          <div className="text-text-secondary text-sm">Receive notifications for errors</div>
        </div>
        <button
          onClick={() => handleToggle('notifyOnError')}
          className={`w-12 h-6 rounded-full transition-colors ${
            settings.notifyOnError ? 'bg-primary' : 'bg-gray-600'
          }`}
        >
          <div className={`w-5 h-5 rounded-full bg-white shadow transform transition-transform ${
            settings.notifyOnError ? 'translate-x-6' : 'translate-x-0.5'
          }`} />
        </button>
      </div>

      <div className="flex items-center justify-between py-3 border-b border-border-dark">
        <div>
          <div className="text-white font-medium">Notify on Warnings</div>
          <div className="text-text-secondary text-sm">Receive notifications for warnings</div>
        </div>
        <button
          onClick={() => handleToggle('notifyOnWarning')}
          className={`w-12 h-6 rounded-full transition-colors ${
            settings.notifyOnWarning ? 'bg-primary' : 'bg-gray-600'
          }`}
        >
          <div className={`w-5 h-5 rounded-full bg-white shadow transform transition-transform ${
            settings.notifyOnWarning ? 'translate-x-6' : 'translate-x-0.5'
          }`} />
        </button>
      </div>

      <div className="flex items-center justify-between py-3 border-b border-border-dark">
        <div>
          <div className="text-white font-medium">Notify on Completion</div>
          <div className="text-text-secondary text-sm">Receive notifications when tasks complete</div>
        </div>
        <button
          onClick={() => handleToggle('notifyOnComplete')}
          className={`w-12 h-6 rounded-full transition-colors ${
            settings.notifyOnComplete ? 'bg-primary' : 'bg-gray-600'
          }`}
        >
          <div className={`w-5 h-5 rounded-full bg-white shadow transform transition-transform ${
            settings.notifyOnComplete ? 'translate-x-6' : 'translate-x-0.5'
          }`} />
        </button>
      </div>
    </div>
  );

  const renderSettingsContent = () => {
    switch (activeSection) {
      case 'general': return renderGeneralSettings();
      case 'api': return renderApiSettings();
      case 'storage': return renderStorageSettings();
      case 'llm': return renderLLMSettings();
      case 'agents': return renderAgentSettings();
      case 'notifications': return renderNotificationSettings();
      default: return renderGeneralSettings();
    }
  };

  return (
    <div className="flex flex-col gap-6 h-full">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-white text-2xl font-bold">Settings</h1>
          <p className="text-text-secondary text-sm">
            Configure system settings and preferences.
          </p>
        </div>
        <Button variant="primary" icon="save" onClick={saveSettings}>
          Save Changes
        </Button>
      </div>

      {/* Settings Layout */}
      <div className="flex gap-6 flex-1 overflow-hidden">
        {/* Sidebar */}
        <Card className="w-64 shrink-0">
          <nav className="space-y-1">
            {settingsSections.map((section) => (
              <button
                key={section.id}
                onClick={() => setActiveSection(section.id)}
                className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg text-left transition-colors ${
                  activeSection === section.id
                    ? 'bg-primary/20 text-primary'
                    : 'text-text-secondary hover:bg-white/5 hover:text-white'
                }`}
              >
                <Icon name={section.icon} />
                <div>
                  <div className="font-medium text-sm">{section.title}</div>
                </div>
              </button>
            ))}
          </nav>
        </Card>

        {/* Content */}
        <Card className="flex-1 overflow-y-auto">
          <div className="mb-6">
            <h2 className="text-white text-lg font-semibold">
              {settingsSections.find(s => s.id === activeSection)?.title}
            </h2>
            <p className="text-text-secondary text-sm">
              {settingsSections.find(s => s.id === activeSection)?.description}
            </p>
          </div>
          {renderSettingsContent()}
        </Card>
      </div>
    </div>
  );
};
