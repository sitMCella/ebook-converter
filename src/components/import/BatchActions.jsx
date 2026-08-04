import { BookPlus } from 'lucide-react';
import { useImportContext } from '../../contexts/ImportContext';
import { useImport } from '../../hooks/useImport';
import { Button } from '../ui/Button';

export function BatchActions() {
  const { state, dispatch } = useImportContext();
  const { importStagedFiles, isProcessing } = useImport();

  const selectedCount = state.selectedPaths.size;
  const readyPaths = Array.from(state.selectedPaths).filter((path) => {
    const file = state.stagedFiles.get(path);
    return file?.status === 'ready';
  });

  const handleRemove = () => {
    const paths = Array.from(state.selectedPaths);
    dispatch({ type: 'UNSTAGE_FILES', paths });
  };

  const handleImport = () => {
    if (readyPaths.length === 0) return;
    importStagedFiles(readyPaths);
  };

  return (
    <div className="flex justify-end gap-3 mt-4">
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
  );
}
