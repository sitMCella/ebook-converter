import { formatFileSize } from '../../lib/format';

function getEpubName(file) {
  if (file.outputPath) {
    return file.outputPath.split(/[\\/]/).pop();
  }
  return file.name.replace(/\.pdf$/i, '.epub');
}

export function EpubListItem({ file, selected, onSelect }) {
  const epubName = getEpubName(file);
  const epubSize = file.conversionResult?.fileSize ?? 0;

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
      <span className="text-[13px] truncate block text-[var(--text-primary)]">
        {epubName}
      </span>
      {epubSize > 0 && (
        <span className="text-[12px] text-[var(--text-muted)] mt-0.5 block">
          {formatFileSize(epubSize)}
        </span>
      )}
    </button>
  );
}
