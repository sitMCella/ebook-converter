import { Book } from 'lucide-react';

export function EpubPreview({ file }) {
  const chapters = file.conversionResult?.chapters ?? 0;

  return (
    <div className="flex flex-col items-center justify-center py-10 rounded-[12px] bg-[var(--surface-2)] border border-[var(--border)]">
      <Book size={48} className="text-[var(--text-muted)] mb-3" />
      <p className="text-[13px] text-[var(--text-muted)]">
        EPUB preview not yet available
      </p>
      {chapters > 0 && (
        <p className="text-[12px] text-[var(--text-muted)] mt-1">
          {chapters} {chapters === 1 ? 'chapter' : 'chapters'}
        </p>
      )}
    </div>
  );
}
