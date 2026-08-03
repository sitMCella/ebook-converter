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
  const meta = file.metadata;
  const fileSize = meta?.fileSize ?? file.size;

  const rows = [
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

  if (visibleRows.length === 0) {
    return (
      <div>
        <h4 className="text-[14px] font-medium mb-3 pb-2 border-b border-[var(--border)]">
          Metadata
        </h4>
        <p className="text-[12px] text-[var(--text-muted)]">
          No metadata available.
        </p>
      </div>
    );
  }

  return (
    <div>
      <h4 className="text-[14px] font-medium mb-3 pb-2 border-b border-[var(--border)]">
        Metadata
      </h4>
      <div className="flex flex-col gap-3">
        {visibleRows.map((row) => (
          <MetadataRow key={row.label} label={row.label} value={row.value} />
        ))}
      </div>
    </div>
  );
}
