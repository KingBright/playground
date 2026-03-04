import React from 'react';

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  helperText?: string;
  error?: string;
  icon?: string;
  iconRight?: string;
}

export const Input: React.FC<InputProps> = ({
  label,
  helperText,
  error,
  icon,
  iconRight,
  className = '',
  ...props
}) => {
  return (
    <div className="w-full">
      {label && (
        <label className="block text-xs font-semibold text-slate-500 dark:text-text-secondary mb-2 uppercase tracking-wide">
          {label}
        </label>
      )}
      <div
        className={`
          flex items-center gap-2 rounded-lg
          bg-slate-50 dark:bg-background-dark
          border ${error ? 'border-red-500' : 'border-slate-300 dark:border-border-dark'}
          px-3 py-2.5
          focus-within:ring-2 focus-within:ring-primary focus-within:border-transparent
          transition-all
          ${className}
        `}
      >
        {icon && (
          <span className="material-symbols-outlined text-slate-400 text-[20px]">{icon}</span>
        )}
        <input
          className="w-full bg-transparent border-none outline-none text-slate-900 dark:text-white placeholder-slate-400 dark:placeholder-text-muted text-sm"
          {...props}
        />
        {iconRight && (
          <span className="material-symbols-outlined text-slate-400 text-[20px] cursor-pointer">
            {iconRight}
          </span>
        )}
      </div>
      {helperText && !error && (
        <p className="mt-1 text-xs text-slate-500 dark:text-text-muted">{helperText}</p>
      )}
      {error && (
        <p className="mt-1 text-xs text-red-500">{error}</p>
      )}
    </div>
  );
};

interface TextAreaProps extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  label?: string;
  helperText?: string;
  error?: string;
}

export const TextArea: React.FC<TextAreaProps> = ({
  label,
  helperText,
  error,
  className = '',
  ...props
}) => {
  return (
    <div className="w-full">
      {label && (
        <label className="block text-xs font-semibold text-slate-500 dark:text-text-secondary mb-2 uppercase tracking-wide">
          {label}
        </label>
      )}
      <textarea
        className={`
          w-full rounded-lg
          bg-slate-50 dark:bg-background-dark
          border ${error ? 'border-red-500' : 'border-slate-300 dark:border-border-dark'}
          px-3 py-2.5
          focus:ring-2 focus:ring-primary focus:border-transparent
          transition-all
          text-slate-900 dark:text-white placeholder-slate-400 dark:placeholder-text-muted text-sm
          resize-none
          ${className}
        `}
        {...props}
      />
      {helperText && !error && (
        <p className="mt-1 text-xs text-slate-500 dark:text-text-muted">{helperText}</p>
      )}
      {error && (
        <p className="mt-1 text-xs text-red-500">{error}</p>
      )}
    </div>
  );
};
