import React from 'react';

interface CallStep {
  id: string;
  title: string;
  description: string;
  icon: string;
  color: string;
}

interface OracleCallChainProps {
  steps: CallStep[];
  latency?: string;
  tokens?: string;
}

export const OracleCallChain: React.FC<OracleCallChainProps> = ({
  steps,
  latency = '420ms',
  tokens = '1,204'
}) => {
  return (
    <div className="bg-surface-dark rounded-xl p-6 border border-border-dark flex-1 overflow-x-auto">
      {/* Horizontal Chain Visualizer */}
      <div className="flex items-start min-w-[600px] gap-2">
        {steps.map((step, index) => (
          <div key={step.id} className="flex-1 flex flex-col gap-3 relative group">
            <div className="flex items-center gap-2 mb-2">
              <div className={`w-8 h-8 rounded-full bg-${step.color}-500/20 flex items-center justify-center text-${step.color}-400 border border-${step.color}-500/30`}>
                <span className="material-symbols-outlined text-[18px]">{step.icon}</span>
              </div>
              <span className={`text-xs font-semibold text-${step.color}-400 uppercase`}>{step.id}</span>
              {index < steps.length - 1 && (
                <>
                  <div className="h-[2px] bg-border-dark flex-1 ml-2 relative top-[1px]" />
                  <span className="material-symbols-outlined text-border-dark -ml-2 text-[16px]">arrow_right</span>
                </>
              )}
            </div>
            <div className="bg-background-dark p-3 rounded border border-border-dark">
              <p className="text-white text-sm font-medium">{step.title}</p>
              <p className="text-text-secondary text-xs mt-1">{step.description}</p>
            </div>
          </div>
        ))}
      </div>

      {/* Footer Stats */}
      <div className="mt-8 pt-6 border-t border-border-dark flex justify-between items-center text-sm text-text-secondary">
        <span>Total Latency: <span className="text-white">{latency}</span></span>
        <span>Tokens: <span className="text-white">{tokens}</span></span>
      </div>
    </div>
  );
};
