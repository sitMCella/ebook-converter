import { useImportContext } from '../../contexts/ImportContext';
import { ImportListRow } from './ImportListRow';

export function ImportList() {
  const { state, dispatch } = useImportContext();
  const files = Array.from(state.files.values());

  if (files.length === 0) {
    return (
      <div className="py-8 text-center text-[13px] text-[var(--text-muted)]">
        No files imported yet.
      </div>
    );
  }

  return (
    <div>
      <p className="text-[12px] text-[var(--text-muted)] mb-2">Recent imports</p>
      <div className="max-h-[300px] overflow-y-auto rounded-[var(--radius)] border border-[var(--border)] bg-[var(--surface-0)]">
        {files.map((file) => (
          <ImportListRow
            key={file.path}
            file={file}
            selected={state.selectedPaths.has(file.path)}
            onToggleSelect={() =>
              dispatch({ type: 'TOGGLE_SELECTION', path: file.path })
            }
          />
        ))}
      </div>
    </div>
  );
}
