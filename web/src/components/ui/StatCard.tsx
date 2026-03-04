import React from 'react';
import { Card } from './Card';
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
    <Card hover className="p-5">
      <div className="flex justify-between items-start mb-4">
        <div className={`p-2 rounded-lg ${iconColor}`}>
          <span className="material-symbols-outlined text-white">{icon}</span>
        </div>
        {change !== undefined && (
          <span className={`flex items-center text-xs font-bold px-2 py-1 rounded ${getTrendStyles()}`}>
            {trend === 'up' && '+'}{change}%
            {changeLabel && <span className="ml-1">{changeLabel}</span>}
          </span>
        )}
      </div>
      <div className="flex flex-col">
        <span className="text-text-secondary text-sm font-medium">{label}</span>
        <span className="text-white text-2xl font-bold mt-1">{value}</span>
      </div>
    </Card>
  );
};
