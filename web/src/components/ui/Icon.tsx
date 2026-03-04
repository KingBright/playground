// 本地图标组件 - 不依赖外部字体
// 使用 Unicode 符号或文字作为图标替代方案

import React from 'react';

interface IconProps {
  name: string;
  className?: string;
  style?: React.CSSProperties;
}

// 图标映射表 - 使用 Unicode 符号或 emoji
const iconMap: Record<string, string> = {
  // 导航
  'dashboard': '📊',
  'psychology': '🧠',
  'smart_toy': '🤖',
  'apps': '📱',
  'schedule': '⏰',
  'description': '📄',
  'settings': '⚙️',

  // 操作
  'add': '➕',
  'home': '🏠',
  'light_mode': '☀️',
  'dark_mode': '🌙',
  'close': '✕',
  'menu': '☰',
  'more_vert': '⋮',
  'search': '🔍',
  'filter_list': '🔽',
  'refresh': '↻',
  'delete': '🗑️',
  'edit': '✏️',
  'save': '💾',
  'play_arrow': '▶️',
  'pause': '⏸️',
  'stop': '⏹️',
  'skip_next': '⏭️',
  'skip_previous': '⏮️',

  // 状态
  'check_circle': '✅',
  'error': '❌',
  'warning': '⚠️',
  'info': 'ℹ️',
  'help': '❓',

  // 文件
  'folder': '📁',
  'file': '📄',
  'upload': '⬆️',
  'download': '⬇️',

  // 通信
  'send': '📤',
  'mail': '✉️',
  'chat': '💬',
  'notifications': '🔔',

  // 用户
  'person': '👤',
  'group': '👥',
  'login': '🔐',
  'logout': '🚪',

  // 其他
  'code': '📝',
  'link': '🔗',
  'open_in_new': '↗️',
  'arrow_back': '←',
  'arrow_forward': '→',
  'arrow_up': '↑',
  'arrow_down': '↓',
  'expand_more': '▼',
  'expand_less': '▲',
  'chevron_left': '‹',
  'chevron_right': '›',
  'visibility': '👁️',
  'visibility_off': '🚫',
  'sync': '🔄',
  'cached': '💾',
  'timer': '⏱️',
  'speed': '⚡',
  'memory': '💾',
  'storage': '💽',
  'network': '🌐',
  'security': '🔒',
  'bug_report': '🐛',
  'analytics': '📈',
  'trending_up': '📈',
  'trending_down': '📉',
  'call_made': '↗️',
  'call_received': '↘️',
  'account_tree': '🌳',
  'polyline': '📈',
  'radio_button_checked': '🔘',
  'radio_button_unchecked': '○',
  'check_box': '☑️',
  'check_box_outline_blank': '⬜',
  'toggle_on': '🔛',
  'toggle_off': '⭕',

  // 默认
  'default': '•',
};

export const Icon: React.FC<IconProps> = ({ name, className = '', style }) => {
  const icon = iconMap[name] || iconMap['default'];

  return (
    <span
      className={`inline-flex items-center justify-center ${className}`}
      style={{
        fontSize: '1.25em',
        lineHeight: 1,
        ...style
      }}
      role="img"
      aria-label={name}
    >
      {icon}
    </span>
  );
};

export default Icon;
