import React from 'react';

interface TerminalLine {
  timestamp?: string;
  level: 'info' | 'warn' | 'error' | 'success';
  message: string;
}

interface TerminalConsoleProps {
  title?: string;
  lines: TerminalLine[];
  showPrompt?: boolean;
  className?: string;
}

export const TerminalConsole: React.FC<TerminalConsoleProps> = ({
  title = 'Terminal',
  lines,
  showPrompt = true,
  className = ''
}) => {
  const getLevelStyles = (level: string) => {
    switch (level) {
      case 'info':
        return 'text-blue-300';
      case 'warn':
        return 'text-yellow-300';
      case 'error':
        return 'text-red-400';
      case 'success':
        return 'text-green-400';
      default:
        return 'text-gray-400';
    }
  };

  return (
    <div className={`bg-[#0d1117] rounded-xl border border-border-dark flex flex-col overflow-hidden shadow-lg ${className}`}>
      {/* Terminal Header */}
      <div className="bg-surface-dark border-b border-border-dark px-4 py-2 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full bg-red-500/80"></div>
          <div className="w-3 h-3 rounded-full bg-yellow-500/80"></div>
          <div className="w-3 h-3 rounded-full bg-green-500/80"></div>
        </div>
        <div className="text-xs text-text-secondary font-mono">{title}</div>
      </div>

      {/* Terminal Body */}
      <div className="p-4 font-mono text-sm overflow-y-auto flex-1 text-gray-400 min-h-[200px]">
        {lines.map((line, index) => (
          <div key={index} className="mb-1 opacity-80">
            {line.timestamp && <span className="text-gray-600">[{line.timestamp}] </span>}
            {line.level && <span className={getLevelStyles(line.level)}>{line.level.toUpperCase()} </span>}
            <span>{line.message}</span>
          </div>
        ))}

        {showPrompt && (
          <div className="mt-4 flex items-center gap-2">
            <span className="text-green-500">➜</span>
            <span className="text-blue-400">~</span>
            <span className="animate-pulse w-2 h-4 bg-gray-400 block"></span>
          </div>
        )}
      </div>
    </div>
  );
};
