import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { useEffect } from 'react';

vi.mock('../../lib/tauri', async (importOriginal) => ({
  ...(await importOriginal()),
  listBooks: vi.fn().mockResolvedValue([]),
}));

import { ImportProvider, useImportContext } from '../../contexts/ImportContext';
import { ConversionProvider, useConversionContext } from '../../contexts/ConversionContext';
import { ConversionQueue } from './ConversionQueue';

function SeedState({ files, paths, children }) {
  const { dispatch: importDispatch } = useImportContext();
  const { dispatch: conversionDispatch } = useConversionContext();
  useEffect(() => {
    if (files) {
      importDispatch({ type: 'ADD_FILES', files });
    }
    if (paths) {
      conversionDispatch({ type: 'ENQUEUE_FILES', paths });
    }
  }, []);
  return children;
}

function Wrapper({ children }) {
  return (
    <ImportProvider>
      <ConversionProvider>{children}</ConversionProvider>
    </ImportProvider>
  );
}

describe('ConversionQueue', () => {
  it('renders nothing when no active file and queue is empty', () => {
    const { container } = render(
      <Wrapper>
        <ConversionQueue />
      </Wrapper>,
    );
    expect(container.firstChild).toBeNull();
  });

  it('shows active file', () => {
    render(
      <Wrapper>
        <SeedState
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'converting' }]}
          paths={['/a.pdf']}
        >
          <ConversionQueue />
        </SeedState>
      </Wrapper>,
    );
    expect(screen.getByText('a.pdf')).toBeInTheDocument();
  });

  it('shows queued files', () => {
    render(
      <Wrapper>
        <SeedState
          files={[
            { path: '/a.pdf', name: 'a.pdf', status: 'converting' },
            { path: '/b.pdf', name: 'b.pdf', status: 'converting' },
            { path: '/c.pdf', name: 'c.pdf', status: 'converting' },
          ]}
          paths={['/a.pdf', '/b.pdf', '/c.pdf']}
        >
          <ConversionQueue />
        </SeedState>
      </Wrapper>,
    );
    expect(screen.getByText('a.pdf')).toBeInTheDocument();
    expect(screen.getByText('b.pdf')).toBeInTheDocument();
    expect(screen.getByText('c.pdf')).toBeInTheDocument();
  });

  it('shows "Queued" label for non-active converting files', () => {
    render(
      <Wrapper>
        <SeedState
          files={[
            { path: '/a.pdf', name: 'a.pdf', status: 'converting' },
            { path: '/b.pdf', name: 'b.pdf', status: 'converting' },
          ]}
          paths={['/a.pdf', '/b.pdf']}
        >
          <ConversionQueue />
        </SeedState>
      </Wrapper>,
    );
    expect(screen.getByText('Queued')).toBeInTheDocument();
  });
});
