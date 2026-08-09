import { useState } from 'react';
import { ChevronRight, ChevronDown } from 'lucide-react';
import { formatFileSize } from '../../lib/format';

function MetadataRow({ label, value }) {
  if (value == null || value === '') return null;

  return (
    <div className="flex flex-col gap-0.5">
      <span className="text-[11px] text-[var(--text-muted)] uppercase tracking-[0.3px]">
        {label}
      </span>
      <span className="text-[13px] text-[var(--text-primary)]">{value}</span>
    </div>
  );
}

export function MetadataSection({ file }) {
  const [expanded, setExpanded] = useState(false);
  const meta = file.metadata;
  const fileSize = meta?.fileSize ?? file.size;

  const rows = [
    { label: 'File name', value: file.name },
    { label: 'Title', value: meta?.title },
    { label: 'Authors', value: meta?.author },
    { label: 'Pages', value: meta?.pageCount || null },
    { label: 'File size', value: fileSize > 0 ? formatFileSize(fileSize) : null },
    { label: 'Format', value: meta?.pdfVersion ? `PDF ${meta.pdfVersion}` : null },
    { label: 'Created', value: meta?.createdDate },
    { label: 'Modified', value: meta?.modifiedDate },
    { label: 'Producer', value: meta?.producer },
  ];

  const visibleRows = rows.filter((r) => r.value != null && r.value !== '');

  return (
    <div>
      <button
        type="button"
        onClick={() => setExpanded(!expanded)}
        className="flex items-center gap-1.5 w-full text-left text-[14px] font-medium py-2 cursor-pointer bg-transparent border-none text-[var(--text-primary)]"
      >
        {expanded ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
        Metadata
      </button>

      {!expanded && meta?.title && (
        <p className="pl-6 text-[13px] text-[var(--text-secondary)]">
          {meta.title}{meta.pageCount ? ` · ${meta.pageCount} pages` : ''}
        </p>
      )}

      {expanded && (
        <div className="pl-6 pb-2 flex flex-col gap-3">
          {visibleRows.length === 0 ? (
            <p className="text-[12px] text-[var(--text-muted)]">
              No metadata available.
            </p>
          ) : (
            visibleRows.map((row) => (
              <MetadataRow key={row.label} label={row.label} value={row.value} />
            ))
          )}
        </div>
      )}
    </div>
  );
}
