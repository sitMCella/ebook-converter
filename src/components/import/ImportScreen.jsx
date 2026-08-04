import { useState, useEffect, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { FolderOpen, Library, ArrowRight } from 'lucide-react';
import { openPdfFiles } from '../../lib/tauri';
import { useImport } from '../../hooks/useImport';
import { useDragDrop } from '../../hooks/useDragDrop';
import { DropZone } from './DropZone';
import { ImportList } from './ImportList';
import { BatchActions } from './BatchActions';
import { Button } from '../ui/Button';

export function ImportScreen() {
  const { stageFiles } = useImport();
  const navigate = useNavigate();
  const [importedCount, setImportedCount] = useState(0);

  const handleStageFiles = useCallback(
    (paths) => {
      stageFiles(paths);
      setImportedCount(0);
    },
    [stageFiles]
  );

  const handleBrowse = useCallback(async () => {
    const paths = await openPdfFiles();
    if (paths) handleStageFiles(paths);
  }, [handleStageFiles]);

  const handleImportComplete = useCallback((count) => {
    setImportedCount((prev) => prev + count);
  }, []);

  useDragDrop(handleStageFiles);

  useEffect(() => {
    const handleKeyDown = (e) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'o') {
        e.preventDefault();
        handleBrowse();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleBrowse]);

  return (
    <div>
      <div className="flex items-center justify-between mb-5">
        <h3 className="text-[18px] font-medium">Import PDF files</h3>
        <Button onClick={handleBrowse}>
          <FolderOpen size={16} />
          Browse files
        </Button>
      </div>

      <DropZone onFilesSelected={handleStageFiles} />

      <div className="mt-6">
        <ImportList />
      </div>

      {importedCount > 0 && (
        <div className="flex items-center justify-between mt-4 px-4 py-3 rounded-[var(--radius)] border border-[var(--border)] bg-[var(--surface-1)]">
          <div className="flex items-center gap-2 text-[13px] text-[var(--text-secondary)]">
            <Library size={16} className="text-[var(--fill-accent)]" />
            <span>
              {importedCount} {importedCount === 1 ? 'file' : 'files'} imported
              to library
            </span>
          </div>
          <button
            type="button"
            className="inline-flex items-center gap-1 text-[13px] font-medium text-[var(--fill-accent)] hover:opacity-80 transition-opacity cursor-pointer"
            onClick={() => navigate('/library')}
          >
            View in Library
            <ArrowRight size={14} />
          </button>
        </div>
      )}

      <BatchActions onImportComplete={handleImportComplete} />
    </div>
  );
}
