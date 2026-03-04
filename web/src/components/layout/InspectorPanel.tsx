import React from 'react';

interface Property {
  label: string;
  value: string;
}

interface Activity {
  title: string;
  description?: string;
  timestamp: string;
  status?: 'completed' | 'active' | 'pending';
}

interface InspectorPanelProps {
  title?: string;
  onClose?: () => void;
  selectedObject?: {
    id: string;
    name: string;
    status: 'active' | 'inactive';
    icon: string;
    iconColor: string;
  };
  properties?: Property[];
  activities?: Activity[];
  footerAction?: {
    label: string;
    onClick: () => void;
  };
}

export const InspectorPanel: React.FC<InspectorPanelProps> = ({
  title = 'Inspector',
  onClose,
  selectedObject,
  properties = [],
  activities = [],
  footerAction
}) => {
  return (
    <aside className="hidden xl:flex w-80 flex-col border-l border-slate-200 dark:border-slate-800 bg-white dark:bg-[#111722] p-0 shrink-0 h-full">
      <div className="p-4 border-b border-slate-200 dark:border-slate-800 flex justify-between items-center">
        <h3 className="text-slate-900 dark:text-white font-bold text-sm">{title}</h3>
        {onClose && (
          <button className="text-slate-400 hover:text-primary transition-colors" onClick={onClose}>
            <span className="material-symbols-outlined text-[20px]">close</span>
          </button>
        )}
      </div>

      <div className="p-4 flex-1 overflow-y-auto">
        {/* Selected Object */}
        {selectedObject && (
          <div className="mb-6">
            <p className="text-xs font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider mb-3">
              Selected Object
            </p>
            <div className="flex items-center gap-3 mb-4">
              <div className={`size-10 rounded-lg flex items-center justify-center ${selectedObject.iconColor}`}>
                <span className="material-symbols-outlined">{selectedObject.icon}</span>
              </div>
              <div>
                <p className="text-slate-900 dark:text-white text-sm font-semibold">{selectedObject.name}</p>
                <p className="text-emerald-500 text-xs flex items-center gap-1">
                  <span className="block size-1.5 rounded-full bg-emerald-500"></span>
                  {selectedObject.status === 'active' ? 'Active' : 'Inactive'}
                </p>
              </div>
            </div>

            {properties.length > 0 && (
              <div className="space-y-3">
                {properties.map((prop, index) => (
                  <div
                    key={index}
                    className="flex justify-between text-sm py-2 border-b border-slate-100 dark:border-slate-800/50"
                  >
                    <span className="text-slate-500 dark:text-slate-400">{prop.label}</span>
                    <span className="text-slate-900 dark:text-slate-200 font-mono text-xs">{prop.value}</span>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {/* Recent Activity */}
        {activities.length > 0 && (
          <div>
            <p className="text-xs font-bold text-slate-400 dark:text-slate-500 uppercase tracking-wider mb-3">
              Recent Activity
            </p>
            <div className="space-y-4">
              {activities.map((activity, index) => (
                <div key={index} className="flex gap-3 relative">
                  <div className="flex flex-col items-center">
                    <div
                      className={`size-2 rounded-full ${
                        activity.status === 'active'
                          ? 'bg-primary'
                          : activity.status === 'completed'
                          ? 'bg-slate-300 dark:bg-slate-600'
                          : 'bg-slate-300 dark:bg-slate-600'
                      }`}
                    ></div>
                    {index < activities.length - 1 && (
                      <div className="w-px h-full bg-slate-200 dark:bg-slate-800 my-1"></div>
                    )}
                  </div>
                  <div className="pb-2">
                    <p className="text-slate-800 dark:text-slate-200 text-xs font-medium">{activity.title}</p>
                    {activity.description && (
                      <p className="text-slate-400 dark:text-slate-500 text-[10px]">{activity.description}</p>
                    )}
                    <p className="text-slate-400 dark:text-slate-600 text-[10px] mt-1">{activity.timestamp}</p>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>

      {footerAction && (
        <div className="p-4 border-t border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-[#1a2130]">
          <button
            className="w-full py-2 bg-primary hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors shadow-lg shadow-primary/20"
            onClick={footerAction.onClick}
          >
            {footerAction.label}
          </button>
        </div>
      )}
    </aside>
  );
};
