import { useEffect, useCallback } from 'react';
import { FolderOpen } from 'lucide-react';
import { openPdfFiles } from '../../lib/tauri';
import { useImport } from '../../hooks/useImport';
import { useDragDrop } from '../../hooks/useDragDrop';
import { DropZone } from './DropZone';
import { ImportList } from './ImportList';
import { BatchActions } from './BatchActions';
import { Button } from '../ui/Button';

export function ImportScreen() {
  const { stageFiles } = useImport();

  const handleBrowse = useCallback(async () => {
    const paths = await openPdfFiles();
    if (paths) stageFiles(paths);
  }, [stageFiles]);

  useDragDrop(stageFiles);

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

      <DropZone onFilesSelected={stageFiles} />

      <div className="mt-6">
        <ImportList />
      </div>

      <BatchActions />
    </div>
  );
}
