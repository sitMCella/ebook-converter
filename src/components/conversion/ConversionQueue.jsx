import { useConversionContext } from '../../contexts/ConversionContext';
import { useImportContext } from '../../contexts/ImportContext';
import { ConversionQueueRow } from './ConversionQueueRow';

export function ConversionQueue() {
  const { state: conversionState } = useConversionContext();
  const { state: importState } = useImportContext();

  const { activeFile, queue } = conversionState;

  if (!activeFile && queue.length === 0) return null;

  const activeFileData = activeFile ? importState.files.get(activeFile) : null;

  return (
    <div className="border border-[var(--border)] rounded-[var(--radius)] overflow-hidden">
      {activeFileData && (
        <ConversionQueueRow file={activeFileData} isActive />
      )}
      {queue.map((path) => {
        const file = importState.files.get(path);
        if (!file) return null;
        return <ConversionQueueRow key={path} file={file} isActive={false} />;
      })}
    </div>
  );
}
