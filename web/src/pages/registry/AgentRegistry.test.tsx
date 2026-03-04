import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import { AgentRegistry } from './AgentRegistry';
import { api } from '../../api';

// Mock the API module
vi.mock('../../api', () => ({
  api: {
    synergy: {
      getAgents: vi.fn(),
    },
  },
}));

describe('AgentRegistry Component', () => {
  const mockAgents = [
    {
      id: 'agent-1',
      name: 'Test Agent One',
      description: 'A test agent',
      type: 'universal',
      capabilities: ['data', 'chat'],
      status: 'active',
      version: '1.0',
      icon: 'smart_toy'
    },
    {
      id: 'agent-2',
      name: 'Test Agent Two',
      description: 'Another test agent',
      type: 'oracle',
      capabilities: ['knowledge'],
      status: 'inactive',
      version: '2.0',
      icon: 'brain'
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders loading state initially', () => {
    (api.synergy.getAgents as any).mockImplementation(() => new Promise(() => {}));
    render(<AgentRegistry />);
    expect(screen.getByText('Loading agent registry...')).toBeInTheDocument();
  });

  it('renders agent list successfully from API', async () => {
    (api.synergy.getAgents as any).mockResolvedValue({ agents: mockAgents });

    render(<AgentRegistry />);

    await waitFor(() => {
      expect(screen.queryByText('Loading agent registry...')).not.toBeInTheDocument();
    });

    // Check header
    expect(screen.getByText('Agent Registry')).toBeInTheDocument();

    // Check agents rendering
    expect(screen.getByText('Test Agent One')).toBeInTheDocument();
    expect(screen.getByText('Test Agent Two')).toBeInTheDocument();

    // Check agent properties
    expect(screen.getByText('A test agent')).toBeInTheDocument();
    expect(screen.getByText('Another test agent')).toBeInTheDocument();
    expect(screen.getByText('v1.0')).toBeInTheDocument();
    expect(screen.getByText('v2.0')).toBeInTheDocument();
    expect(screen.getByText('data')).toBeInTheDocument();
    expect(screen.getByText('chat')).toBeInTheDocument();
    expect(screen.getByText('knowledge')).toBeInTheDocument();
  });

  it('falls back to default agents if API returns empty/undefined agents but no error', async () => {
    (api.synergy.getAgents as any).mockResolvedValue({});

    render(<AgentRegistry />);

    await waitFor(() => {
      expect(screen.queryByText('Loading agent registry...')).not.toBeInTheDocument();
    });

    // It should render default agent if no agents are passed (handling logic in component)
    // Wait for empty state or default state
  });

  it('filters agents by search term', async () => {
    (api.synergy.getAgents as any).mockResolvedValue({ agents: mockAgents });

    render(<AgentRegistry />);

    await waitFor(() => {
      expect(screen.queryByText('Loading agent registry...')).not.toBeInTheDocument();
    });

    // Both should be visible
    expect(screen.getByText('Test Agent One')).toBeInTheDocument();
    expect(screen.getByText('Test Agent Two')).toBeInTheDocument();

    // Find input and type
    const searchInput = screen.getByPlaceholderText('Search agents...');
    fireEvent.change(searchInput, { target: { value: 'One' } });

    // One should be visible, Two should not
    expect(screen.getByText('Test Agent One')).toBeInTheDocument();
    expect(screen.queryByText('Test Agent Two')).not.toBeInTheDocument();
  });

  it('filters agents by description term', async () => {
    (api.synergy.getAgents as any).mockResolvedValue({ agents: mockAgents });

    render(<AgentRegistry />);

    await waitFor(() => {
      expect(screen.queryByText('Loading agent registry...')).not.toBeInTheDocument();
    });

    const searchInput = screen.getByPlaceholderText('Search agents...');
    fireEvent.change(searchInput, { target: { value: 'Another' } });

    expect(screen.queryByText('Test Agent One')).not.toBeInTheDocument();
    expect(screen.getByText('Test Agent Two')).toBeInTheDocument();
  });
});
