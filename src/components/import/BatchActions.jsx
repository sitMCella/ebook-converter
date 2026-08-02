import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { ArrowRightLeft } from 'lucide-react';
import { useImportContext } from '../../contexts/ImportContext';
import { useConversion } from '../../hooks/useConversion';
import { deleteBook } from '../../lib/tauri';
import { Button } from '../ui/Button';
import { ConfirmDialog } from '../ui/ConfirmDialog';

export function BatchActions() {
  const { state, dispatch } = useImportContext();
  const { startConversion } = useConversion();
  const navigate = useNavigate();
  const [showConfirm, setShowConfirm] = useState(false);

  const selectedCount = state.selectedPaths.size;
  const readyPaths = Array.from(state.selectedPaths).filter((path) => {
    const file = state.files.get(path);
    return file?.status === 'ready';
  });

  const handleRemove = async () => {
    const paths = Array.from(state.selectedPaths);
    for (const path of paths) {
      const file = state.files.get(path);
      if (file?.bookId) {
        try {
          await deleteBook(file.bookId);
        } catch (e) {
          console.warn('Failed to delete book storage:', e);
        }
      }
    }
    dispatch({ type: 'REMOVE_FILES', paths });
    setShowConfirm(false);
  };

  const handleConvert = () => {
    if (readyPaths.length === 0) return;
    startConversion(readyPaths);
    navigate('/converting');
  };

  return (
    <>
      <div className="flex justify-end gap-3 mt-4">
        <Button
          variant="secondary"
          disabled={selectedCount === 0}
          onClick={() => setShowConfirm(true)}
        >
          Remove selected
        </Button>
        <Button
          disabled={readyPaths.length === 0}
          onClick={handleConvert}
        >
          <ArrowRightLeft size={16} />
          Convert selected
        </Button>
      </div>

      <ConfirmDialog
        open={showConfirm}
        title="Remove files"
        message={`Remove ${selectedCount} file(s) from the import list? This will also delete the stored copies.`}
        onConfirm={handleRemove}
        onCancel={() => setShowConfirm(false)}
      />
    </>
  );
}
