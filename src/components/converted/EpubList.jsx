import { EpubListItem } from './EpubListItem';

export function EpubList({ files, selectedPath, onSelect }) {
  return (
    <div
      className="w-[260px] min-w-[260px] border-r border-[var(--border)] overflow-y-auto"
      role="listbox"
      aria-label="Converted EPUB list"
    >
      {files.map((file) => (
        <EpubListItem
          key={file.path}
          file={file}
          selected={file.path === selectedPath}
          onSelect={() => onSelect(file.path)}
        />
      ))}
    </div>
  );
}
