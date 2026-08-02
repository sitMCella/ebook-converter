import { describe, it, expect } from 'vitest';
import { render, screen, act } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
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

function SeedFiles({ files, children }) {
  const { dispatch } = useImportContext();
  useEffect(() => {
    if (files && files.length > 0) {
      dispatch({ type: 'ADD_FILES', files });
    }
  }, []);
  return children;
}

describe('ImportList', () => {
  it('shows empty state when no files', () => {
    render(
      <Wrapper>
        <ImportList />
      </Wrapper>
    );
    expect(screen.getByText('No files imported yet.')).toBeInTheDocument();
  });

  it('shows "Recent imports" label when files exist', () => {
    render(
      <Wrapper>
        <SeedFiles files={[{ path: '/a.pdf', name: 'a.pdf', status: 'ready', size: 1024, metadata: null }]}>
          <ImportList />
        </SeedFiles>
      </Wrapper>
    );
    expect(screen.getByText('Recent imports')).toBeInTheDocument();
  });

  it('renders file rows for each imported file', () => {
    render(
      <Wrapper>
        <SeedFiles
          files={[
            { path: '/a.pdf', name: 'a.pdf', status: 'ready', size: 1024, metadata: null },
            { path: '/b.pdf', name: 'b.pdf', status: 'ready', size: 2048, metadata: null },
          ]}
        >
          <ImportList />
        </SeedFiles>
      </Wrapper>
    );
    expect(screen.getByText('a.pdf')).toBeInTheDocument();
    expect(screen.getByText('b.pdf')).toBeInTheDocument();
  });
});
