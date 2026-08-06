import { formatFileSize } from '../../lib/format';

function MetadataRow({ label, value }) {
  if (value == null || value === '' || value === 0) return null;

  return (
    <div className="flex flex-col gap-0.5">
      <span className="text-[11px] text-[var(--text-muted)] uppercase tracking-[0.3px]">
        {label}
      </span>
      <span className="text-[13px] text-[var(--text-primary)]">{value}</span>
    </div>
  );
}

function countOverrides(overrides) {
  let count = 0;
  for (const group of Object.values(overrides)) {
    if (typeof group === 'object' && group !== null) {
      count += Object.keys(group).length;
    }
  }
  return count;
}

function getSettingsLabel(file) {
  if (!file.overrides) return 'Default';
  const count = countOverrides(file.overrides);
  if (count === 0) return 'Default';
  return `${count} override${count !== 1 ? 's' : ''}`;
}

export function EpubMetadata({ file }) {
  const result = file.conversionResult;
  const epubSize = result?.fileSize ?? 0;
  const images = result?.images ?? 0;

  const rows = [
    { label: 'Source', value: file.name },
    { label: 'EPUB size', value: epubSize > 0 ? formatFileSize(epubSize) : null },
    { label: 'Images', value: images > 0 ? `${images} extracted` : null },
    { label: 'Cover', value: result?.hasCover ? 'Included' : null },
    { label: 'Settings used', value: getSettingsLabel(file) },
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
