import { StatusBadge } from '../import/StatusBadge';
import { formatFileSize } from '../../lib/format';

export function DocumentListItem({ file, selected, onSelect }) {
  const fileSize = file.metadata?.fileSize ?? file.size;

  return (
    <button
      type="button"
      role="option"
      aria-selected={selected}
      onClick={onSelect}
      className={`w-full text-left px-4 py-2.5 border-b border-[var(--border)] cursor-pointer transition-colors ${
        selected
          ? 'bg-[var(--bg-accent)] font-medium'
          : 'hover:bg-[var(--surface-2)]'
      }`}
    >
      <div className="flex items-center justify-between gap-2">
        <span className="text-[13px] truncate text-[var(--text-primary)]">
          {file.name}
        </span>
        {file.status === 'error' && <StatusBadge status="error" />}
      </div>
      {fileSize > 0 && (
        <span className="text-[12px] text-[var(--text-muted)] mt-0.5 block">
          {formatFileSize(fileSize)}
        </span>
      )}
    </button>
  );
}
