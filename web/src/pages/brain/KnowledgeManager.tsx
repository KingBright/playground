import React, { useEffect, useState } from 'react';
import { api } from '../../api';
import { Card, Input } from '../../components/ui';
import type { KnowledgeSlice } from '../../types';

const defaultSlices: KnowledgeSlice[] = [
  {
    id: '1',
    name: '#ProjectAlpha_V2',
    nodeCount: 12400,
    status: 'active',
    lastUpdated: '2m ago',
    tags: ['project', 'v2']
  },
  {
    id: '2',
    name: '@Marketing_Q3',
    nodeCount: 8500,
    status: 'active',
    lastUpdated: '15m ago',
    tags: ['marketing', 'q3']
  },
  {
    id: '3',
    name: '#Research_2024',
    nodeCount: 23100,
    status: 'inactive',
    lastUpdated: '1h ago',
    tags: ['research']
  }
];

export const KnowledgeManager: React.FC = () => {
  const [searchQuery, setSearchQuery] = useState('');
  const [activeFilter, setActiveFilter] = useState('Entities');
  const [slices, setSlices] = useState<KnowledgeSlice[]>(defaultSlices);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const data = await api.brain.getKnowledgeSlices();
        setSlices(data.slices);
      } catch (error) {
        console.error('Failed to fetch knowledge slices:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, []);


  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-primary animate-pulse">Loading knowledge graph...</div>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-6 h-full">
      {/* Header */}
      <div className="flex flex-wrap justify-between items-end gap-4">
        <div className="flex flex-col gap-2">
          <h1 className="text-white text-3xl font-black leading-tight tracking-tight">Graph Memory Overview</h1>
          <p className="text-text-secondary text-base font-normal">
            Manage data ingestion, entity extraction, and knowledge relationships.
          </p>
        </div>
        <div className="flex gap-3">
          <button className="flex items-center justify-center h-10 px-4 bg-surface-dark hover:bg-surface-dark-light text-white rounded-lg text-sm font-medium transition-colors border border-border-dark">
            <span className="material-symbols-outlined mr-2 text-sm">upload_file</span>
            Import Schema
          </button>
          <button className="flex items-center justify-center h-10 px-4 bg-primary hover:bg-blue-600 text-white rounded-lg text-sm font-bold shadow-lg shadow-blue-900/20 transition-colors">
            <span className="material-symbols-outlined mr-2 text-sm">add_circle</span>
            Add Data Source
          </button>
        </div>
      </div>

      {/* Three Column Layout */}
      <div className="grid grid-cols-12 gap-6 flex-1 min-h-0">
        {/* Left Column: Knowledge Slices & Search */}
        <div className="col-span-12 lg:col-span-3 flex flex-col gap-4">
          {/* Search Block */}
          <Card className="p-4">
            <label className="flex flex-col w-full mb-3">
              <span className="text-xs font-semibold text-text-secondary mb-2 uppercase tracking-wide">Knowledge Search</span>
              <div className="flex w-full items-center rounded-lg bg-background-dark border border-border-dark focus-within:border-primary transition-colors">
                <span className="material-symbols-outlined text-text-secondary pl-3">search</span>
                <Input
                  className="w-full bg-transparent border-none text-white placeholder-text-muted focus:ring-0 text-sm py-2.5"
                  placeholder="#ProjectAlpha, @Entities..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                />
              </div>
            </label>
            <div className="flex flex-wrap gap-2 mb-2">
              {['Entities', 'Relations', 'Raw'].map((filter) => (
                <button
                  key={filter}
                  onClick={() => setActiveFilter(filter)}
                  className={`px-2.5 py-1 rounded text-xs border transition-colors ${activeFilter === filter
                    ? 'bg-surface-dark-light text-white border-primary'
                    : 'bg-surface-dark text-text-secondary border-transparent hover:border-border-dark'
                    }`}
                >
                  {filter}
                </button>
              ))}
            </div>
          </Card>

          {/* Slices List */}
          <Card className="flex flex-col flex-1 overflow-hidden">
            <div className="p-4 border-b border-border-dark flex justify-between items-center bg-surface-dark-light">
              <h3 className="text-white font-bold text-sm">Active Slices</h3>
              <button className="text-text-secondary hover:text-white">
                <span className="material-symbols-outlined text-sm">filter_list</span>
              </button>
            </div>
            <div className="flex-1 overflow-y-auto p-2 space-y-2">
              {slices.map((slice) => (
                <div
                  key={slice.id}
                  className="p-3 rounded-lg bg-surface-dark-light/40 border border-border-dark hover:border-primary/50 cursor-pointer group"
                >
                  <div className="flex justify-between items-start mb-1">
                    <div className="flex items-center gap-2">
                      <span className={`material-symbols-outlined ${slice.name.startsWith('#') ? 'text-accent-neon' : 'text-accent-purple'} text-sm`}>
                        token
                      </span>
                      <span className="text-white text-sm font-medium">{slice.name}</span>
                    </div>
                    <span className={`size-2 rounded-full ${slice.status === 'active' ? 'bg-green-500' : 'bg-slate-500'}`}></span>
                  </div>
                  <div className="flex justify-between text-xs text-text-secondary mt-2">
                    <span>{slice.nodeCount.toLocaleString()} Nodes</span>
                    <span className="text-text-muted">Updated {slice.lastUpdated}</span>
                  </div>
                </div>
              ))}
            </div>
          </Card>
        </div>

        {/* Center Column: Graph Visualization */}
        <div className="col-span-12 lg:col-span-6 flex flex-col">
          <Card className="flex-1 flex flex-col overflow-hidden">
            <div className="p-4 border-b border-border-dark flex justify-between items-center">
              <h3 className="text-white font-bold text-sm flex items-center gap-2">
                <span className="material-symbols-outlined text-primary">hub</span>
                Knowledge Graph
              </h3>
              <div className="flex gap-2">
                <button className="p-1.5 text-text-secondary hover:text-white rounded hover:bg-surface-dark-light transition-colors">
                  <span className="material-symbols-outlined text-sm">zoom_in</span>
                </button>
                <button className="p-1.5 text-text-secondary hover:text-white rounded hover:bg-surface-dark-light transition-colors">
                  <span className="material-symbols-outlined text-sm">zoom_out</span>
                </button>
                <button className="p-1.5 text-text-secondary hover:text-white rounded hover:bg-surface-dark-light transition-colors">
                  <span className="material-symbols-outlined text-sm">fullscreen</span>
                </button>
              </div>
            </div>
            <div className="flex-1 bg-background-dark relative overflow-hidden flex items-center justify-center">
              {/* Graph Visualization Placeholder */}
              <div className="text-center">
                <div className="w-64 h-64 rounded-full border-2 border-dashed border-border-dark flex items-center justify-center mb-4 mx-auto">
                  <div className="w-48 h-48 rounded-full border-2 border-dashed border-primary/30 flex items-center justify-center">
                    <div className="w-32 h-32 rounded-full bg-primary/10 flex items-center justify-center">
                      <span className="material-symbols-outlined text-4xl text-primary">hub</span>
                    </div>
                  </div>
                </div>
                <p className="text-text-secondary text-sm">Interactive Graph Visualization</p>
                <p className="text-text-muted text-xs mt-1">12,400 nodes • 48,200 edges</p>
              </div>
            </div>
          </Card>
        </div>

        {/* Right Column: Details Panel */}
        <div className="col-span-12 lg:col-span-3 flex flex-col gap-4">
          <Card className="p-4">
            <h3 className="text-white font-bold text-sm mb-4">Entity Details</h3>
            <div className="space-y-3">
              <div className="flex justify-between text-sm">
                <span className="text-text-secondary">Type</span>
                <span className="text-white">Organization</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-text-secondary">Confidence</span>
                <span className="text-white">0.98</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-text-secondary">Relations</span>
                <span className="text-white">24</span>
              </div>
            </div>
          </Card>

          <Card className="flex-1 p-4">
            <h3 className="text-white font-bold text-sm mb-4">Recent Updates</h3>
            <div className="space-y-3">
              {[
                { time: '2m ago', action: 'New node added', entity: '#ProjectAlpha_V2' },
                { time: '15m ago', action: 'Relation updated', entity: '@Marketing_Q3' },
                { time: '1h ago', action: 'Entity merged', entity: '#Research_2024' }
              ].map((update, idx) => (
                <div key={idx} className="flex gap-3 text-sm">
                  <span className="text-text-muted text-xs whitespace-nowrap">{update.time}</span>
                  <div>
                    <p className="text-white">{update.action}</p>
                    <p className="text-text-secondary text-xs">{update.entity}</p>
                  </div>
                </div>
              ))}
            </div>
          </Card>
        </div>
      </div>
    </div>
  );
};
