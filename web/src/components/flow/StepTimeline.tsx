import React from 'react';

interface TimelineStep {
  id: string;
  title: string;
  description?: string;
  status: 'success' | 'active' | 'pending';
  timestamp?: string;
  icon?: string;
}

interface StepTimelineProps {
  steps: TimelineStep[];
}

export const StepTimeline: React.FC<StepTimelineProps> = ({ steps }) => {
  const getIconStyles = (status: string) => {
    switch (status) {
      case 'success':
        return 'text-primary';
      case 'active':
        return 'text-emerald-400';
      default:
        return 'text-text-secondary';
    }
  };

  const getTextStyles = (status: string) => {
    switch (status) {
      case 'success':
      case 'active':
        return 'text-white';
      default:
        return 'text-text-secondary';
    }
  };

  return (
    <div className="bg-surface-dark rounded-xl p-6 border border-border-dark">
      <div className="grid grid-cols-[40px_1fr] gap-x-3">
        {steps.map((step, index) => (
          <React.Fragment key={step.id}>
            {/* Icon Column */}
            <div className="flex flex-col items-center gap-1 pt-1">
              <span className={`material-symbols-outlined text-[24px] ${getIconStyles(step.status)}`}>
                {step.icon || (step.status === 'success' ? 'check_circle' : step.status === 'active' ? 'play_circle' : 'circle')}
              </span>
              {index < steps.length - 1 && (
                <div className="w-[2px] bg-border-dark h-full min-h-[32px]" />
              )}
            </div>

            {/* Content Column */}
            <div className={`flex flex-1 flex-col ${index < steps.length - 1 ? 'pb-6' : ''}`}>
              <div className="flex justify-between items-start">
                <p className={`text-base font-medium ${getTextStyles(step.status)}`}>{step.title}</p>
                {step.status === 'active' && (
                  <span className="text-xs bg-emerald-500/20 text-emerald-400 px-2 py-0.5 rounded">Active</span>
                )}
                {step.status === 'success' && (
                  <span className="text-xs bg-primary/20 text-primary px-2 py-0.5 rounded">Success</span>
                )}
              </div>
              {step.description && (
                <p className="text-text-secondary text-sm mt-1">{step.description}</p>
              )}
              {step.timestamp && (
                <p className="text-text-secondary/50 text-xs mt-1">{step.timestamp}</p>
              )}
            </div>
          </React.Fragment>
        ))}
      </div>
    </div>
  );
};
