import { useState, useEffect, useMemo } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { Search, FolderOpen, Upload } from 'lucide-react';
import { useImportContext } from '../../contexts/ImportContext';
import { openFolder, getBooksDir, isTauri } from '../../lib/tauri';
import { EpubList } from './EpubList';
import { EpubDetailPanel } from './EpubDetailPanel';
import { Button } from '../ui/Button';

function getEpubName(file) {
  if (file.outputPath) {
    return file.outputPath.split(/[\\/]/).pop();
  }
  return file.name.replace(/\.pdf$/i, '.epub');
}

export function ConvertedScreen() {
  const { state } = useImportContext();
  const location = useLocation();
  const navigate = useNavigate();
  const [selectedPath, setSelectedPath] = useState(
    location.state?.selectedPath || null,
  );
  const [searchQuery, setSearchQuery] = useState('');

  const convertedFiles = useMemo(
    () => Array.from(state.files.values()).filter((f) => f.status === 'converted'),
    [state.files],
  );

  const filteredFiles = useMemo(
    () =>
      convertedFiles.filter((f) =>
        getEpubName(f).toLowerCase().includes(searchQuery.toLowerCase()),
      ),
    [convertedFiles, searchQuery],
  );

  const isInFilteredList = filteredFiles.some((f) => f.path === selectedPath);
  const selectedFile =
    selectedPath && isInFilteredList ? state.files.get(selectedPath) : null;

  useEffect(() => {
    if (!selectedFile && filteredFiles.length > 0) {
      setSelectedPath(filteredFiles[0].path);
    }
  }, [selectedFile, filteredFiles]);

  const handleOpenFolder = async () => {
    const booksDir = await getBooksDir();
    if (booksDir) {
      await openFolder(booksDir);
    }
  };

  if (convertedFiles.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-64 gap-4">
        <Upload size={32} className="text-[var(--text-muted)]" />
        <p className="text-[13px] text-[var(--text-muted)]">
          No converted files yet. Import and convert a PDF to see it here.
        </p>
        <Button onClick={() => navigate('/import')}>Go to Import</Button>
      </div>
    );
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-5">
        <h3 className="text-[18px] font-medium">Converted EPUBs</h3>
        <div className="flex items-center gap-3">
          {isTauri && (
            <Button variant="secondary" onClick={handleOpenFolder}>
              <FolderOpen size={16} />
              Open folder
            </Button>
          )}
          <div className="relative">
            <Search
              size={14}
              className="absolute left-2.5 top-1/2 -translate-y-1/2 text-[var(--text-muted)]"
            />
            <input
              type="text"
              placeholder="Search converted..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              aria-label="Search converted EPUBs"
              className="w-[180px] pl-8 pr-3 py-1.5 text-[13px] rounded-[var(--radius)] border border-[var(--border)] bg-[var(--surface-0)] text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:ring-2 focus:ring-[var(--fill-accent)]"
            />
          </div>
        </div>
      </div>

      <div className="flex rounded-[12px] border border-[var(--border)] bg-[var(--surface-0)] overflow-hidden" style={{ minHeight: 'calc(100vh - 140px)' }}>
        <EpubList
          files={filteredFiles}
          selectedPath={selectedPath}
          onSelect={setSelectedPath}
        />
        <div className="flex-1 overflow-y-auto">
          {selectedFile ? (
            <EpubDetailPanel file={selectedFile} />
          ) : (
            <div className="flex items-center justify-center h-full text-[13px] text-[var(--text-muted)]">
              {filteredFiles.length === 0
                ? 'No converted files match your search.'
                : 'Select a converted file to view details.'}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
