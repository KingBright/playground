import React from 'react';
import { Badge } from '../ui/Badge';
import { Button } from '../ui/Button';

interface FlowNodeProps {
  id: string;
  title: string;
  description: string;
  status: 'running' | 'completed' | 'error' | 'pending';
  imageUrl?: string;
  onConfigure?: () => void;
  onViewLogs?: () => void;
  className?: string;
}

export const FlowNode: React.FC<FlowNodeProps> = ({
  id,
  title,
  description,
  status,
  imageUrl,
  onConfigure,
  onViewLogs,
  className = '',
}) => {
  const getStatusStyles = () => {
    switch (status) {
      case 'running':
        return { badge: 'info' as const, stripe: 'bg-emerald-500' };
      case 'completed':
        return { badge: 'success' as const, stripe: 'bg-blue-500' };
      case 'error':
        return { badge: 'error' as const, stripe: 'bg-red-500' };
      default:
        return { badge: 'default' as const, stripe: 'bg-slate-500' };
    }
  };

  const { badge, stripe } = getStatusStyles();

  return (
    <div className={`relative group ${className}`}>
      {/* Input Port */}
      <div className="hidden md:block absolute left-0 top-1/2 -translate-x-1/2 -translate-y-1/2 w-4 h-4 bg-primary rounded-full border-2 border-background-dark z-10" />

      {/* Main Card */}
      <div className="flex flex-col md:flex-row items-stretch justify-between gap-6 rounded-xl bg-surface-dark p-6 shadow-lg border border-border-dark relative overflow-hidden">
        {/* Status Indicator Stripe */}
        <div className={`absolute top-0 left-0 w-1 h-full ${stripe}`} />

        <div className="flex flex-[2] flex-col justify-between gap-6 z-10">
          <div className="flex flex-col gap-2">
            <div className="flex items-center gap-2">
              <Badge variant={badge}>{status}</Badge>
              <span className="text-text-secondary text-xs">Node ID: #{id}</span>
            </div>
            <h3 className="text-white text-2xl font-bold leading-tight">{title}</h3>
            <p className="text-text-secondary text-sm leading-normal max-w-md">{description}</p>
          </div>
          <div className="flex gap-3 mt-2">
            <Button variant="primary" size="sm" icon="settings" onClick={onConfigure}>
              Configure
            </Button>
            <Button variant="secondary" size="sm" icon="terminal" onClick={onViewLogs}>
              View Logs
            </Button>
          </div>
        </div>

        {/* Visual/Media Slot */}
        {imageUrl && (
          <div
            className="w-full md:w-64 bg-background-dark rounded-lg flex items-center justify-center relative overflow-hidden border border-border-dark bg-cover bg-center min-h-[150px]"
            style={{ backgroundImage: `url(${imageUrl})` }}
          >
            <div className="absolute inset-0 bg-primary/10" />
          </div>
        )}
      </div>

      {/* Output Port */}
      <div className="hidden md:block absolute right-0 top-1/2 translate-x-1/2 -translate-y-1/2 w-4 h-4 bg-primary rounded-full border-2 border-background-dark z-10" />
    </div>
  );
};
