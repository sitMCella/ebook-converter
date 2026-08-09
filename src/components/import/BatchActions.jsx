import { BookPlus } from 'lucide-react';
import { useImportContext } from '../../contexts/ImportContext';
import { useImport } from '../../hooks/useImport';
import { Button } from '../ui/Button';
import { ProgressBar } from '../conversion/ProgressBar';

export function BatchActions({ onImportComplete }) {
  const { state, dispatch } = useImportContext();
  const { importStagedFiles, isProcessing, importProgress } = useImport();

  const selectedCount = state.selectedPaths.size;
  const readyPaths = Array.from(state.selectedPaths).filter((path) => {
    const file = state.stagedFiles.get(path);
    return file?.status === 'ready';
  });

  const handleRemove = () => {
    const paths = Array.from(state.selectedPaths);
    dispatch({ type: 'UNSTAGE_FILES', paths });
  };

  const handleImport = async () => {
    if (readyPaths.length === 0) return;
    const count = await importStagedFiles(readyPaths);
    if (count > 0) onImportComplete?.(count);
  };

  const percent = importProgress
    ? Math.round((importProgress.completed / importProgress.total) * 100)
    : 0;

  return (
    <div className="mt-4 space-y-3">
      {importProgress && (
        <div className="flex items-center gap-3">
          <div className="flex-1">
            <ProgressBar
              percent={percent}
              label={`Importing files: ${importProgress.completed} of ${importProgress.total}`}
            />
          </div>
          <span className="text-[13px] text-[var(--text-secondary)] whitespace-nowrap">
            {importProgress.completed} / {importProgress.total}
          </span>
        </div>
      )}
      <div className="flex justify-end gap-3">
        <Button
          variant="secondary"
          disabled={selectedCount === 0}
          onClick={handleRemove}
        >
          Remove selected
        </Button>
        <Button
          disabled={readyPaths.length === 0 || isProcessing}
          onClick={handleImport}
        >
          <BookPlus size={16} />
          Import to library
        </Button>
      </div>
    </div>
  );
}
