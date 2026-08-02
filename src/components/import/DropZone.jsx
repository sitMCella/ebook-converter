import { useState, useCallback } from 'react';
import { CloudUpload } from 'lucide-react';
import { openPdfFiles } from '../../lib/tauri';

export function DropZone({ onFilesSelected }) {
  const [isDragOver, setIsDragOver] = useState(false);

  const handleDragEnter = useCallback((e) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(true);
  }, []);

  const handleDragOver = useCallback((e) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDragLeave = useCallback((e) => {
    e.preventDefault();
    e.stopPropagation();
    if (!e.currentTarget.contains(e.relatedTarget)) {
      setIsDragOver(false);
    }
  }, []);

  const handleDrop = useCallback(
    (e) => {
      e.preventDefault();
      e.stopPropagation();
      setIsDragOver(false);
    },
    []
  );

  const handleKeyDown = useCallback(
    async (e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        const paths = await openPdfFiles();
        if (paths) onFilesSelected(paths);
      }
    },
    [onFilesSelected]
  );

  return (
    <div
      role="button"
      tabIndex={0}
      aria-label="Drop zone for PDF files"
      onDragEnter={handleDragEnter}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
      onKeyDown={handleKeyDown}
      className={`w-full py-10 px-5 rounded-[12px] border-[1.5px] border-dashed transition-all flex flex-col items-center justify-center gap-3 outline-none focus-visible:ring-2 focus-visible:ring-[var(--fill-accent)] ${
        isDragOver
          ? 'border-[var(--fill-accent)] bg-[var(--bg-accent)] scale-[1.01]'
          : 'border-[var(--border-strong)] bg-transparent'
      }`}
    >
      <CloudUpload
        size={32}
        className={isDragOver ? 'text-[var(--text-accent)]' : 'text-[var(--text-muted)]'}
      />
      <p className="text-[13px] text-[var(--text-primary)] font-medium">
        Drop PDF files here
      </p>
      <p className="text-[12px] text-[var(--text-muted)]">
        or click &quot;Browse files&quot; to select from your computer
      </p>
    </div>
  );
}
