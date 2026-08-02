import { useState } from 'react';
import { FileText } from 'lucide-react';
import { StatusBadge } from '../import/StatusBadge';
import { ProgressBar } from './ProgressBar';

export function ConversionQueueRow({ file, isActive }) {
  const [showError, setShowError] = useState(false);

  return (
    <>
      <div
        className={`flex items-center gap-3 px-3.5 py-2.5 border-b border-[var(--border)] ${
          file.status === 'error' ? 'cursor-pointer' : ''
        }`}
        onClick={() => {
          if (file.status === 'error') setShowError((s) => !s);
        }}
      >
        <FileText size={18} className="text-[var(--text-accent)] shrink-0" />
        <span className="flex-1 min-w-0 text-[13px] font-medium truncate text-[var(--text-primary)]">
          {file.name}
        </span>
        {isActive && file.conversionProgress != null && (
          <span className="text-[12px] text-[var(--text-muted)] shrink-0">
            {file.conversionProgress}%
          </span>
        )}
        {!isActive && file.status === 'converting' && (
          <span className="text-[12px] text-[var(--text-muted)] shrink-0">Queued</span>
        )}
        <StatusBadge status={isActive ? file.status : 'converting'} />
      </div>
      {isActive && file.status === 'converting' && (
        <div className="px-3.5 pb-2">
          <ProgressBar percent={file.conversionProgress || 0} label={`Converting ${file.name}`} />
        </div>
      )}
      {showError && file.errorMessage && (
        <div className="px-3.5 py-2 bg-[var(--bg-danger)] text-[var(--text-danger)] text-[12px] border-b border-[var(--border)]">
          {file.errorMessage}
        </div>
      )}
    </>
  );
}
