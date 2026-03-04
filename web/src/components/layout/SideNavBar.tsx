import React from 'react';
import type { NavItem } from '../../types';

interface SideNavBarProps {
  items: NavItem[];
  activeItemId: string;
  onItemClick: (item: NavItem) => void;
  systemStatus?: {
    healthy: boolean;
    uptime: number;
  };
  user?: {
    name: string;
    role: string;
    avatar?: string;
  };
  footerAction?: {
    label: string;
    icon: string;
    onClick: () => void;
  };
}

export const SideNavBar: React.FC<SideNavBarProps> = ({
  items,
  activeItemId,
  onItemClick,
  systemStatus,
  user,
  footerAction
}) => {
  const groupedItems = items.reduce((acc, item) => {
    const group = item.children ? 'group' : 'main';
    if (!acc[group]) acc[group] = [];
    acc[group].push(item);
    return acc;
  }, {} as Record<string, NavItem[]>);

  return (
    <div className="hidden lg:flex w-72 flex-col justify-between border-r border-slate-200 dark:border-slate-800 bg-white dark:bg-[#111722] p-4 shrink-0 h-full">
      <div className="flex flex-col gap-4">
        {/* Logo */}
        <div className="flex items-center gap-3 px-2">
          <div className="bg-primary/20 rounded-full size-10 flex items-center justify-center text-primary">
            <span className="material-symbols-outlined">hub</span>
          </div>
          <div className="flex flex-col">
            <h1 className="text-slate-900 dark:text-white text-base font-bold leading-normal">AI Platform</h1>
            <p className="text-slate-500 dark:text-[#92a4c9] text-xs font-normal leading-normal">v2.4.0-alpha</p>
          </div>
        </div>

        <div className="h-px bg-slate-200 dark:bg-slate-800 w-full my-1"></div>

        {/* Main Navigation */}
        <div className="flex flex-col gap-1">
          {groupedItems.main?.map(item => (
            <div
              key={item.id}
              className={`flex items-center gap-3 px-3 py-2.5 rounded-lg cursor-pointer transition-colors group ${
                activeItemId === item.id
                  ? 'bg-[#232f48] text-white'
                  : 'text-[#92a4c9] hover:bg-[#232f48]/50 hover:text-white'
              }`}
              onClick={() => onItemClick(item)}
            >
              <span className={`material-symbols-outlined text-[20px] ${
                activeItemId === item.id ? 'text-white' : 'group-hover:text-white'
              }`}>
                {item.icon}
              </span>
              <p className="text-sm font-medium leading-normal">{item.label}</p>
            </div>
          ))}
        </div>

        {/* Grouped Navigation */}
        {groupedItems.group && (
          <div className="mt-4">
            <p className="px-3 text-xs font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider mb-2">
              Systems
            </p>
            {groupedItems.group.map(item => (
              <div
                key={item.id}
                className={`flex items-center gap-3 px-3 py-2.5 rounded-lg cursor-pointer transition-colors group ${
                  activeItemId === item.id
                    ? 'bg-[#232f48] text-white'
                    : 'text-[#92a4c9] hover:bg-[#232f48]/50 hover:text-white'
                }`}
                onClick={() => onItemClick(item)}
              >
                <span className={`material-symbols-outlined text-[20px] ${
                  activeItemId === item.id ? 'text-white' : 'group-hover:text-white'
                }`}>
                  {item.icon}
                </span>
                <p className="text-sm font-medium leading-normal">{item.label}</p>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="flex flex-col gap-4">
        {/* System Status Mini-Widget */}
        {systemStatus && (
          <div className="rounded-lg bg-surface-dark border border-border-dark p-3">
            <div className="flex items-center justify-between mb-2">
              <span className="text-xs text-text-secondary font-medium">System Health</span>
              <span className={`flex h-2 w-2 rounded-full ${systemStatus.healthy ? 'bg-green-500' : 'bg-red-500'}`}></span>
            </div>
            <div className="w-full bg-border-dark rounded-full h-1.5 mb-1">
              <div className="bg-primary h-1.5 rounded-full" style={{ width: `${systemStatus.uptime}%` }}></div>
            </div>
            <span className="text-[10px] text-text-secondary">{systemStatus.uptime}% Uptime</span>
          </div>
        )}

        {/* Footer Action */}
        {footerAction && (
          <button
            className="flex w-full cursor-pointer items-center justify-center overflow-hidden rounded-lg h-10 px-4 bg-primary hover:bg-blue-700 transition-colors text-white text-sm font-bold leading-normal tracking-[0.015em] shadow-lg shadow-blue-900/20"
            onClick={footerAction.onClick}
          >
            <span className="mr-2 material-symbols-outlined text-[18px]">{footerAction.icon}</span>
            <span className="truncate">{footerAction.label}</span>
          </button>
        )}

        {/* User Profile */}
        {user && (
          <div className="flex items-center gap-3 px-3 py-2.5 rounded-lg text-[#92a4c9] hover:bg-[#232f48]/50 hover:text-white cursor-pointer transition-colors">
            {user.avatar ? (
              <img src={user.avatar} alt={user.name} className="size-8 rounded-full" />
            ) : (
              <div className="size-8 rounded-full bg-gradient-to-tr from-accent-purple to-primary flex items-center justify-center text-white text-xs font-bold">
                {user.name.charAt(0)}
              </div>
            )}
            <div className="flex flex-col">
              <p className="text-white text-sm font-medium">{user.name}</p>
              <p className="text-[#92a4c9] text-xs">{user.role}</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
