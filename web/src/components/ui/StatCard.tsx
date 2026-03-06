import React from 'react';
import type { StatCardData } from '../../types';

interface StatCardProps {
  data: StatCardData;
}

export const StatCard: React.FC<StatCardProps> = ({ data }) => {
  const { label, value, change, changeLabel, icon, iconColor, trend } = data;

  const getTrendStyles = () => {
    switch (trend) {
      case 'up':
        return 'text-emerald-400 bg-emerald-500/10';
      case 'down':
        return 'text-red-400 bg-red-500/10';
      default:
        return 'text-text-secondary bg-white/5';
    }
  };

  return (
    <div className="bg-[#161e2e] border border-[#232f48] rounded-xl p-5 hover:border-blue-500/50 transition-colors">
      <div className="flex justify-between items-start mb-4">
        <div className={`w-10 h-10 rounded-lg flex items-center justify-center ${iconColor}`}>
          <span className="material-symbols-outlined text-white text-[20px]">{icon}</span>
        </div>
        {change !== undefined && (
          <span className={`flex items-center text-xs font-bold px-2 py-1 rounded ${getTrendStyles()}`}>
            {trend === 'up' && '+'}{change}%
            {changeLabel && <span className="ml-1 opacity-70 font-medium">{changeLabel}</span>}
          </span>
        )}
        {change === undefined && changeLabel && (
          <span className="flex items-center text-xs font-bold px-2 py-1 rounded bg-[#232f48] text-[#92a4c9]">
            {changeLabel}
          </span>
        )}
      </div>
      <div className="flex flex-col">
        <span className="text-[#92a4c9] text-sm font-medium mb-1">{label}</span>
        <div className="flex items-center gap-3">
          <span className="text-white text-3xl font-bold">{value}</span>
          {label === 'Vector DB Storage' && (
            <div className="flex-1 h-1.5 bg-[#232f48] rounded-full overflow-hidden mt-1 max-w-[120px]">
              <div className="h-full bg-teal-400 rounded-full" style={{ width: '64%' }}></div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
