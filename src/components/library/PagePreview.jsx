import { FileText } from 'lucide-react';

export function PagePreview({ file }) {
  const pageCount = file.metadata?.pageCount ?? 0;

  return (
    <div className="flex flex-col items-center justify-center py-10 rounded-[12px] bg-[var(--surface-2)] border border-[var(--border)]">
      <FileText size={48} className="text-[var(--text-muted)] mb-3" />
      <p className="text-[13px] text-[var(--text-muted)]">
        Page preview not yet available
      </p>
      {pageCount > 0 && (
        <p className="text-[12px] text-[var(--text-muted)] mt-1">
          {pageCount} {pageCount === 1 ? 'page' : 'pages'}
        </p>
      )}
    </div>
  );
}
