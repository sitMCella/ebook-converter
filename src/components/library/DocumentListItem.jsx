import { StatusBadge } from '../import/StatusBadge';
import { formatFileSize } from '../../lib/format';

export function DocumentListItem({ file, selected, onSelect }) {
  const fileSize = file.metadata?.fileSize ?? file.size;
  const title = file.metadata?.title || file.name;
  const pageCount = file.metadata?.pageCount;

  const secondaryParts = [];
  if (pageCount) secondaryParts.push(`${pageCount} pages`);
  if (fileSize > 0) secondaryParts.push(formatFileSize(fileSize));

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
        <span className="text-[13px] truncate text-[var(--text-primary)]" title={title}>
          {title}
        </span>
        {file.status === 'error' && <StatusBadge status="error" />}
      </div>
      {secondaryParts.length > 0 && (
        <span className="text-[12px] text-[var(--text-muted)] mt-0.5 block">
          {secondaryParts.join(' · ')}
        </span>
      )}
    </button>
  );
}
