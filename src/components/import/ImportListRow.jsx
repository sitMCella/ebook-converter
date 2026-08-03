import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { FileText } from 'lucide-react';
import { Checkbox } from '../ui/Checkbox';
import { StatusBadge } from './StatusBadge';
import { formatFileSize } from '../../lib/format';

export function ImportListRow({ file, selected, onToggleSelect }) {
  const navigate = useNavigate();
  const [showError, setShowError] = useState(false);

  const fileSize = file.metadata?.fileSize ?? file.size;

  return (
    <>
      <div
        className={`flex items-center gap-3 px-3.5 py-2.5 border-b border-[var(--border)] transition-colors ${
          file.status === 'error' ? 'cursor-pointer' : ''
        }`}
        onClick={() => {
          if (file.status === 'error') setShowError((s) => !s);
        }}
      >
        <Checkbox
          checked={selected}
          onChange={onToggleSelect}
          label={`Select ${file.name}`}
        />
        <FileText size={18} className="text-[var(--text-accent)] shrink-0" />
        <button
          type="button"
          className="flex-1 min-w-0 text-left text-[13px] font-medium truncate hover:underline bg-transparent border-none cursor-pointer p-0 text-[var(--text-primary)]"
          onClick={(e) => {
            e.stopPropagation();
            navigate('/library', { state: { selectedPath: file.path } });
          }}
        >
          {file.name}
        </button>
        {fileSize > 0 && (
          <span className="text-[12px] text-[var(--text-muted)] shrink-0">
            {formatFileSize(fileSize)}
          </span>
        )}
        <StatusBadge status={file.status} />
      </div>
      {showError && file.errorMessage && (
        <div className="px-3.5 py-2 bg-[var(--bg-danger)] text-[var(--text-danger)] text-[12px] border-b border-[var(--border)]">
          {file.errorMessage}
        </div>
      )}
    </>
  );
}
