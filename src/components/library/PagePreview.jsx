import { useState, useEffect } from 'react';
import { FileText, Loader } from 'lucide-react';
import { getPdfCover, isTauri } from '../../lib/tauri';

export function PagePreview({ file }) {
  const [coverImage, setCoverImage] = useState(null);
  const [loading, setLoading] = useState(false);

  const pdfPath = file.storedPdfPath;

  useEffect(() => {
    if (!pdfPath || !isTauri) return;

    let cancelled = false;
    setLoading(true);
    setCoverImage(null);

    getPdfCover(pdfPath)
      .then((data) => {
        if (!cancelled) setCoverImage(data.coverImage);
      })
      .catch(() => {})
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => { cancelled = true; };
  }, [pdfPath]);

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
      <div className="flex flex-col items-center rounded-[12px] bg-[var(--surface-2)] border border-[var(--border)] p-4">
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
      <FileText size={48} className="text-[var(--text-muted)] mb-3" />
      <p className="text-[13px] text-[var(--text-muted)]">
        No cover image available
      </p>
    </div>
  );
}
