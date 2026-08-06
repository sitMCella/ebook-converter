import { useNavigate } from 'react-router-dom';
import { ExternalLink, RefreshCw } from 'lucide-react';
import { openFileWithSystem, isTauri } from '../../lib/tauri';
import { EpubPreview } from './EpubPreview';
import { EpubMetadata } from './EpubMetadata';
import { Button } from '../ui/Button';

export function EpubDetailPanel({ file }) {
  const navigate = useNavigate();

  const handleOpenInReader = async () => {
    if (file.outputPath) {
      await openFileWithSystem(file.outputPath);
    }
  };

  const handleReconvert = () => {
    navigate('/library', { state: { selectedPath: file.path } });
  };

  return (
    <div className="p-5 flex flex-col gap-6">
      <EpubPreview file={file} />
      <EpubMetadata file={file} />

      <div className="flex flex-col gap-2">
        {isTauri && (
          <Button variant="primary" onClick={handleOpenInReader}>
            <ExternalLink size={16} />
            Open in reader
          </Button>
        )}
        <Button variant="secondary" onClick={handleReconvert}>
          <RefreshCw size={16} />
          Reconvert
        </Button>
      </div>
    </div>
  );
}
