import { useState, useEffect } from 'react';
import { Book, Loader } from 'lucide-react';
import { readEpubPreview, isTauri } from '../../lib/tauri';

export function EpubPreview({ file }) {
  const [coverImage, setCoverImage] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const outputPath = file.outputPath;

  useEffect(() => {
    if (!outputPath || !isTauri) return;

    let cancelled = false;
    setLoading(true);
    setError(null);

    readEpubPreview(outputPath)
      .then((data) => {
        if (!cancelled) setCoverImage(data.coverImage);
      })
      .catch((err) => {
        if (!cancelled) setError(err?.message || err?.toString() || 'Failed to load preview');
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => { cancelled = true; };
  }, [outputPath]);

  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center py-10 rounded-[12px] bg-[var(--surface-2)] border border-[var(--border)]">
        <Loader size={24} className="text-[var(--text-muted)] animate-spin" />
        <p className="text-[13px] text-[var(--text-muted)] mt-3">Loading preview...</p>
      </div>
    );
  }

  if (coverImage) {
    return (
      <div className="flex justify-center rounded-[12px] bg-[var(--surface-2)] border border-[var(--border)] p-4">
        <img
          src={coverImage}
          alt="Cover"
          className="max-h-[300px] rounded shadow-sm"
        />
      </div>
    );
  }

  return (
    <div className="flex flex-col items-center justify-center py-10 rounded-[12px] bg-[var(--surface-2)] border border-[var(--border)]">
      <Book size={48} className="text-[var(--text-muted)] mb-3" />
      <p className="text-[13px] text-[var(--text-muted)]">
        {error || 'No cover image available'}
      </p>
    </div>
  );
}
