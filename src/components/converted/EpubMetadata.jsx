import { useState } from 'react';
import { ChevronRight, ChevronDown } from 'lucide-react';
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
  const [expanded, setExpanded] = useState(false);
  const result = file.conversionResult;
  const epubSize = result?.fileSize ?? 0;
  const images = result?.images ?? 0;

  const epubName = file.outputPath
    ? file.outputPath.split(/[\\/]/).pop()
    : file.name?.replace(/\.pdf$/i, '.epub') || null;

  const rows = [
    { label: 'File name', value: epubName },
    { label: 'Source', value: file.name },
    { label: 'EPUB size', value: epubSize > 0 ? formatFileSize(epubSize) : null },
    { label: 'Images', value: images > 0 ? `${images} extracted` : null },
    { label: 'Cover', value: result?.hasCover ? 'Included' : null },
    { label: 'Settings used', value: getSettingsLabel(file) },
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

      {!expanded && file.metadata?.title && (
        <p className="pl-6 text-[13px] text-[var(--text-secondary)]">
          {file.metadata.title}{file.metadata.pageCount ? ` · ${file.metadata.pageCount} pages` : ''}
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
