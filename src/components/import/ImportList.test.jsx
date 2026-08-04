import { describe, it, expect, vi } from 'vitest';
import { render, screen, act } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';

vi.mock('../../lib/tauri', async (importOriginal) => ({
  ...(await importOriginal()),
  listBooks: vi.fn().mockResolvedValue([]),
}));

import { ImportProvider, useImportContext } from '../../contexts/ImportContext';
import { ImportList } from './ImportList';
import { useEffect } from 'react';

function Wrapper({ children }) {
  return (
    <ImportProvider>
      <MemoryRouter>{children}</MemoryRouter>
    </ImportProvider>
  );
}

function SeedStagedFiles({ files, children }) {
  const { dispatch } = useImportContext();
  useEffect(() => {
    if (files && files.length > 0) {
      dispatch({ type: 'STAGE_FILES', files });
    }
  }, []);
  return children;
}

describe('ImportList', () => {
  it('shows empty state when no files are staged', () => {
    render(
      <Wrapper>
        <ImportList />
      </Wrapper>
    );
    expect(screen.getByText('No files staged yet.')).toBeInTheDocument();
  });

  it('shows "Ready to import" label when staged files exist', () => {
    render(
      <Wrapper>
        <SeedStagedFiles files={[{ path: '/a.pdf', name: 'a.pdf', status: 'ready', size: 1024, metadata: null }]}>
          <ImportList />
        </SeedStagedFiles>
      </Wrapper>
    );
    expect(screen.getByText('Ready to import')).toBeInTheDocument();
  });

  it('renders file rows for each staged file', () => {
    render(
      <Wrapper>
        <SeedStagedFiles
          files={[
            { path: '/a.pdf', name: 'a.pdf', status: 'ready', size: 1024, metadata: null },
            { path: '/b.pdf', name: 'b.pdf', status: 'ready', size: 2048, metadata: null },
          ]}
        >
          <ImportList />
        </SeedStagedFiles>
      </Wrapper>
    );
    expect(screen.getByText('a.pdf')).toBeInTheDocument();
    expect(screen.getByText('b.pdf')).toBeInTheDocument();
  });

  it('does not show library files in the staging list', () => {
    function SeedLibraryFiles({ children }) {
      const { dispatch } = useImportContext();
      useEffect(() => {
        dispatch({
          type: 'ADD_FILES',
          files: [{ path: '/lib.pdf', name: 'lib.pdf', status: 'ready', size: 1024, metadata: null }],
        });
      }, []);
      return children;
    }

    render(
      <Wrapper>
        <SeedLibraryFiles>
          <ImportList />
        </SeedLibraryFiles>
      </Wrapper>
    );
    expect(screen.getByText('No files staged yet.')).toBeInTheDocument();
    expect(screen.queryByText('lib.pdf')).not.toBeInTheDocument();
  });
});
