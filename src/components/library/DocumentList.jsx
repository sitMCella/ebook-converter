import { DocumentListItem } from './DocumentListItem';

export function DocumentList({ files, selectedPath, onSelect }) {
  return (
    <div
      className="w-[260px] min-w-[260px] border-r border-[var(--border)] overflow-y-auto"
      role="listbox"
      aria-label="Document list"
    >
      {files.map((file) => (
        <DocumentListItem
          key={file.path}
          file={file}
          selected={file.path === selectedPath}
          onSelect={() => onSelect(file.path)}
        />
      ))}
    </div>
  );
}
