import { useState, useEffect, useMemo } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { Search, Upload } from 'lucide-react';
import { useImportContext } from '../../contexts/ImportContext';
import { DocumentList } from './DocumentList';
import { DetailPanel } from './DetailPanel';
import { Button } from '../ui/Button';

export function LibraryScreen() {
  const { state } = useImportContext();
  const location = useLocation();
  const navigate = useNavigate();
  const [selectedPath, setSelectedPath] = useState(
    location.state?.selectedPath || null,
  );
  const [searchQuery, setSearchQuery] = useState('');

  const files = useMemo(
    () => Array.from(state.files.values()),
    [state.files],
  );

  const filteredFiles = useMemo(
    () =>
      files.filter((f) =>
        f.name.toLowerCase().includes(searchQuery.toLowerCase()),
      ),
    [files, searchQuery],
  );

  const isInFilteredList = filteredFiles.some((f) => f.path === selectedPath);
  const selectedFile =
    selectedPath && isInFilteredList ? state.files.get(selectedPath) : null;

  useEffect(() => {
    if (!selectedFile && filteredFiles.length > 0) {
      setSelectedPath(filteredFiles[0].path);
    }
  }, [selectedFile, filteredFiles]);

  if (files.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-64 gap-4">
        <Upload size={32} className="text-[var(--text-muted)]" />
        <p className="text-[13px] text-[var(--text-muted)]">
          Your library is empty. Import some PDFs to get started.
        </p>
        <Button onClick={() => navigate('/import')}>Go to Import</Button>
      </div>
    );
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-5">
        <h3 className="text-[18px] font-medium">Library</h3>
        <div className="relative">
          <Search
            size={14}
            className="absolute left-2.5 top-1/2 -translate-y-1/2 text-[var(--text-muted)]"
          />
          <input
            type="text"
            placeholder="Search documents..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            aria-label="Search documents"
            className="w-[180px] pl-8 pr-3 py-1.5 text-[13px] rounded-[var(--radius)] border border-[var(--border)] bg-[var(--surface-0)] text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:ring-2 focus:ring-[var(--fill-accent)]"
          />
        </div>
      </div>

      <div className="flex rounded-[12px] border border-[var(--border)] bg-[var(--surface-0)] overflow-hidden" style={{ minHeight: 'calc(100vh - 140px)' }}>
        <DocumentList
          files={filteredFiles}
          selectedPath={selectedPath}
          onSelect={setSelectedPath}
        />
        <div className="flex-1 overflow-y-auto">
          {selectedFile ? (
            <DetailPanel file={selectedFile} />
          ) : (
            <div className="flex items-center justify-center h-full text-[13px] text-[var(--text-muted)]">
              {filteredFiles.length === 0
                ? 'No documents match your search.'
                : 'Select a document to view details.'}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
