import React, { useEffect, useState } from 'react';
import { api } from '../../api';
import { Card, Button, Input, Badge } from '../../components/ui';
import type { Agent } from '../../types';

const defaultAgents: Agent[] = [
  {
    id: '1',
    name: 'Searcher',
    type: 'universal',
    description: 'Universal search agent for web and internal knowledge base',
    capabilities: ['web_search', 'knowledge_query', 'summarize'],
    status: 'active',
    version: '2.1.0',
    icon: 'search'
  },
  {
    id: '2',
    name: 'FactChecker',
    type: 'universal',
    description: 'Verifies facts and cross-references multiple sources',
    capabilities: ['fact_check', 'source_verify', 'confidence_score'],
    status: 'active',
    version: '1.5.2',
    icon: 'fact_check'
  },
  {
    id: '3',
    name: 'CodeAssistant',
    type: 'universal',
    description: 'Specialized in code generation and review',
    capabilities: ['code_gen', 'code_review', 'debug'],
    status: 'active',
    version: '3.0.1',
    icon: 'code'
  }
];

export const AgentRegistry: React.FC = () => {
  const [searchQuery, setSearchQuery] = useState('');
  const [agents, setAgents] = useState<Agent[]>(defaultAgents);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const data = await api.synergy.getAgents();
        if (data && data.agents) {
          setAgents(data.agents);
        }
      } catch (error) {
        console.error('Failed to fetch agents:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, []);


  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-primary animate-pulse">Loading agent registry...</div>
      </div>
    );
  }

  const filteredAgents = agents.filter(agent =>
    agent.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    agent.description.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="flex flex-col gap-6 h-full">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-white text-2xl font-bold">Agent Registry</h1>
          <p className="text-text-secondary text-sm">
            Browse and manage universal agents available across the platform.
          </p>
        </div>
        <Button variant="primary" icon="add">
          Register Agent
        </Button>
      </div>

      {/* Search and Filters */}
      <Card className="p-4">
        <div className="flex gap-4">
          <div className="flex-1">
            <Input
              icon="search"
              placeholder="Search agents..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>
          <Button variant="secondary" icon="filter_list">
            Filters
          </Button>
        </div>
      </Card>

      {/* Agents Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {filteredAgents.map((agent) => (
          <Card key={agent.id} hover className="p-5">
            <div className="flex items-start gap-4">
              <div className="w-12 h-12 rounded-lg bg-primary/20 flex items-center justify-center text-primary">
                <span className="material-symbols-outlined text-2xl">{agent.icon}</span>
              </div>
              <div className="flex-1">
                <div className="flex items-center justify-between">
                  <h3 className="text-white font-bold">{agent.name}</h3>
                  <Badge variant={agent.status === 'active' ? 'success' : 'default'}>
                    {agent.status}
                  </Badge>
                </div>
                <p className="text-text-secondary text-sm mt-1">{agent.description}</p>
                <p className="text-text-muted text-xs mt-2">v{agent.version}</p>

                <div className="flex flex-wrap gap-1 mt-3">
                  {agent.capabilities.slice(0, 3).map((cap, idx) => (
                    <span
                      key={idx}
                      className="px-2 py-0.5 bg-surface-dark-light rounded text-xs text-text-secondary"
                    >
                      {cap}
                    </span>
                  ))}
                </div>

                <div className="flex gap-2 mt-4">
                  <Button variant="primary" size="sm">
                    Mount
                  </Button>
                  <Button variant="secondary" size="sm">
                    Configure
                  </Button>
                </div>
              </div>
            </div>
          </Card>
        ))}
      </div>
    </div>
  );
};
