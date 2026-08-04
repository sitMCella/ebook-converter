import { useNavigate } from 'react-router-dom';
import { CheckCircle } from 'lucide-react';
import { useConversionContext } from '../../contexts/ConversionContext';
import { useImportContext } from '../../contexts/ImportContext';
import { StatusBadge } from '../import/StatusBadge';

export function CompletedList() {
  const navigate = useNavigate();
  const { state: conversionState } = useConversionContext();
  const { state: importState } = useImportContext();

  if (conversionState.completedFiles.length === 0) return null;

  return (
    <div>
      <p className="text-[12px] text-[var(--text-muted)] mb-2 px-1">Completed</p>
      <div className="border border-[var(--border)] rounded-[var(--radius)] overflow-hidden">
        {conversionState.completedFiles.map((path) => {
          const file = importState.files.get(path);
          if (!file) return null;

          return (
            <div
              key={path}
              className="flex items-center gap-3 px-3.5 py-2.5 border-b border-[var(--border)] last:border-b-0 cursor-pointer hover:bg-[var(--surface-1)] transition-colors"
              onClick={() => navigate('/converted', { state: { selectedPath: path } })}
            >
              <CheckCircle
                size={18}
                className={
                  file.status === 'error'
                    ? 'text-[var(--text-danger)] shrink-0'
                    : 'text-[var(--text-success)] shrink-0'
                }
              />
              <span className="flex-1 min-w-0 text-[13px] font-medium truncate text-[var(--text-primary)]">
                {file.name}
              </span>
              {file.status === 'error' && <StatusBadge status="error" />}
            </div>
          );
        })}
      </div>
    </div>
  );
}
