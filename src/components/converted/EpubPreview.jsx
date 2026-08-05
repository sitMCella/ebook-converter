import { useState, useEffect, useRef } from 'react';
import { Book, ChevronLeft, ChevronRight, Loader } from 'lucide-react';
import { readEpubPreview, isTauri } from '../../lib/tauri';

export function EpubPreview({ file }) {
  const [preview, setPreview] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [currentChapter, setCurrentChapter] = useState(0);
  const contentRef = useRef(null);

  const outputPath = file.outputPath;

  useEffect(() => {
    if (!outputPath || !isTauri) return;

    let cancelled = false;
    setLoading(true);
    setError(null);

    readEpubPreview(outputPath)
      .then((data) => {
        if (!cancelled) {
          setPreview(data);
          setCurrentChapter(0);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(err?.message || err?.toString() || 'Failed to load preview');
        }
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => { cancelled = true; };
  }, [outputPath]);

  useEffect(() => {
    if (contentRef.current) {
      contentRef.current.scrollTop = 0;
    }
  }, [currentChapter]);

  if (!isTauri) {
    return <Placeholder chapters={file.conversionResult?.chapters ?? 0} />;
  }

  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center py-10 rounded-[12px] bg-[var(--surface-2)] border border-[var(--border)]">
        <Loader size={24} className="text-[var(--text-muted)] animate-spin" />
        <p className="text-[13px] text-[var(--text-muted)] mt-3">Loading preview...</p>
      </div>
    );
  }

  if (error) {
    return <Placeholder chapters={file.conversionResult?.chapters ?? 0} message={error} />;
  }

  if (!preview || preview.chapters.length === 0) {
    return <Placeholder chapters={file.conversionResult?.chapters ?? 0} />;
  }

  const chapter = preview.chapters[currentChapter];
  const totalChapters = preview.chapters.length;

  return (
    <div className="rounded-[12px] border border-[var(--border)] overflow-hidden bg-[var(--surface-0)]">
      {preview.coverImage && currentChapter === 0 && (
        <div className="flex justify-center bg-[var(--surface-2)] p-4 border-b border-[var(--border)]">
          <img
            src={preview.coverImage}
            alt="Cover"
            className="max-h-[200px] rounded shadow-sm"
          />
        </div>
      )}

      <div className="flex items-center justify-between px-4 py-2 bg-[var(--surface-1)] border-b border-[var(--border)]">
        <button
          onClick={() => setCurrentChapter((c) => Math.max(0, c - 1))}
          disabled={currentChapter === 0}
          className="p-1 rounded hover:bg-[var(--surface-2)] disabled:opacity-30 disabled:cursor-not-allowed text-[var(--text-secondary)]"
          aria-label="Previous chapter"
        >
          <ChevronLeft size={16} />
        </button>

        <span className="text-[12px] text-[var(--text-secondary)] font-medium truncate mx-2">
          {chapter.title}
          <span className="text-[var(--text-muted)] ml-1">
            ({currentChapter + 1}/{totalChapters})
          </span>
        </span>

        <button
          onClick={() => setCurrentChapter((c) => Math.min(totalChapters - 1, c + 1))}
          disabled={currentChapter === totalChapters - 1}
          className="p-1 rounded hover:bg-[var(--surface-2)] disabled:opacity-30 disabled:cursor-not-allowed text-[var(--text-secondary)]"
          aria-label="Next chapter"
        >
          <ChevronRight size={16} />
        </button>
      </div>

      <div
        ref={contentRef}
        className="epub-preview-content px-5 py-4 overflow-y-auto"
        style={{ maxHeight: '400px' }}
        dangerouslySetInnerHTML={{ __html: chapter.html }}
      />
    </div>
  );
}

function Placeholder({ chapters, message }) {
  return (
    <div className="flex flex-col items-center justify-center py-10 rounded-[12px] bg-[var(--surface-2)] border border-[var(--border)]">
      <Book size={48} className="text-[var(--text-muted)] mb-3" />
      <p className="text-[13px] text-[var(--text-muted)]">
        {message || 'EPUB preview not yet available'}
      </p>
      {chapters > 0 && (
        <p className="text-[12px] text-[var(--text-muted)] mt-1">
          {chapters} {chapters === 1 ? 'chapter' : 'chapters'}
        </p>
      )}
    </div>
  );
}
