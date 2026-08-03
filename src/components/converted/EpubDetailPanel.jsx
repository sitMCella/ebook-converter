import { useNavigate } from 'react-router-dom';
import { ExternalLink, Download, RefreshCw } from 'lucide-react';
import { openFileWithSystem, saveFile, isTauri } from '../../lib/tauri';
import { EpubPreview } from './EpubPreview';
import { EpubMetadata } from './EpubMetadata';
import { TableOfContents } from './TableOfContents';
import { Button } from '../ui/Button';

function getEpubName(file) {
  if (file.outputPath) {
    return file.outputPath.split(/[\\/]/).pop();
  }
  return file.name.replace(/\.pdf$/i, '.epub');
}

export function EpubDetailPanel({ file }) {
  const navigate = useNavigate();

  const handleOpenInReader = async () => {
    if (file.outputPath) {
      await openFileWithSystem(file.outputPath);
    }
  };

  const handleSaveAs = async () => {
    if (file.outputPath && isTauri) {
      try {
        const { readFile } = await import('@tauri-apps/plugin-fs');
        const data = await readFile(file.outputPath);
        await saveFile(data, getEpubName(file), [
          { name: 'EPUB Files', extensions: ['epub'] },
        ]);
      } catch {
        // save cancelled or failed
      }
    }
  };

  const handleReconvert = () => {
    navigate('/library', { state: { selectedPath: file.path } });
  };

  return (
    <div className="p-5 flex flex-col gap-6">
      <EpubPreview file={file} />
      <EpubMetadata file={file} />
      <TableOfContents />

      <div className="flex flex-col gap-2">
        {isTauri && (
          <Button variant="primary" onClick={handleOpenInReader}>
            <ExternalLink size={16} />
            Open in reader
          </Button>
        )}
        {isTauri && (
          <Button variant="secondary" onClick={handleSaveAs}>
            <Download size={16} />
            Save as...
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
