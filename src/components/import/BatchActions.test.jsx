import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, act } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useEffect } from 'react';
import { MemoryRouter } from 'react-router-dom';
import { ImportProvider, useImportContext } from '../../contexts/ImportContext';
import { BatchActions } from './BatchActions';

vi.mock('../../lib/tauri', async () => {
  const actual = await vi.importActual('../../lib/tauri');
  return {
    ...actual,
    validatePdf: vi.fn().mockResolvedValue({ status: 'valid' }),
    getPdfMetadata: vi.fn().mockResolvedValue({ title: null, pageCount: 0, pdfVersion: '1.7' }),
    getFileSize: vi.fn().mockResolvedValue(0),
    importPdf: vi.fn().mockResolvedValue({ bookId: 'uuid-test', storedPdfPath: '/stored/test.pdf' }),
    saveBookMetadata: vi.fn().mockResolvedValue(undefined),
    listBooks: vi.fn().mockResolvedValue([]),
  };
});

import { importPdf } from '../../lib/tauri';

function Wrapper({ children }) {
  return (
    <ImportProvider>
      <MemoryRouter>{children}</MemoryRouter>
    </ImportProvider>
  );
}

function SeedStagedState({ files = [], selectedPaths = [], children }) {
  const { dispatch } = useImportContext();
  useEffect(() => {
    if (files.length > 0) {
      dispatch({ type: 'STAGE_FILES', files });
    }
    for (const path of selectedPaths) {
      dispatch({ type: 'TOGGLE_SELECTION', path });
    }
  }, []);
  return children;
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe('BatchActions', () => {
  it('disables "Remove selected" when no rows are selected', async () => {
    render(
      <Wrapper>
        <BatchActions />
      </Wrapper>
    );
    await act(async () => {});
    expect(screen.getByText('Remove selected').closest('button')).toBeDisabled();
  });

  it('disables "Import to library" when no ready rows are selected', async () => {
    render(
      <Wrapper>
        <BatchActions />
      </Wrapper>
    );
    await act(async () => {});
    const importBtn = screen.getByText('Import to library').closest('button');
    expect(importBtn).toBeDisabled();
  });

  it('enables "Remove selected" when rows are selected', async () => {
    render(
      <Wrapper>
        <SeedStagedState
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }]}
          selectedPaths={['/a.pdf']}
        >
          <BatchActions />
        </SeedStagedState>
      </Wrapper>
    );
    await act(async () => {});
    expect(screen.getByText('Remove selected').closest('button')).not.toBeDisabled();
  });

  it('enables "Import to library" when ready rows are selected', async () => {
    render(
      <Wrapper>
        <SeedStagedState
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }]}
          selectedPaths={['/a.pdf']}
        >
          <BatchActions />
        </SeedStagedState>
      </Wrapper>
    );
    await act(async () => {});
    const importBtn = screen.getByText('Import to library').closest('button');
    expect(importBtn).not.toBeDisabled();
  });

  it('keeps "Import to library" disabled when only error files are selected', async () => {
    render(
      <Wrapper>
        <SeedStagedState
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'error' }]}
          selectedPaths={['/a.pdf']}
        >
          <BatchActions />
        </SeedStagedState>
      </Wrapper>
    );
    await act(async () => {});
    const importBtn = screen.getByText('Import to library').closest('button');
    expect(importBtn).toBeDisabled();
  });

  it('removes selected files immediately without confirmation', async () => {
    const user = userEvent.setup();
    function AssertAfterRemove() {
      const { state } = useImportContext();
      return <span data-testid="staged-count">{state.stagedFiles.size}</span>;
    }

    render(
      <Wrapper>
        <SeedStagedState
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }]}
          selectedPaths={['/a.pdf']}
        >
          <BatchActions />
          <AssertAfterRemove />
        </SeedStagedState>
      </Wrapper>
    );
    await act(async () => {});
    await user.click(screen.getByText('Remove selected'));
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(screen.getByTestId('staged-count')).toHaveTextContent('0');
  });

  it('calls importPdf when "Import to library" is clicked', async () => {
    const user = userEvent.setup();
    render(
      <Wrapper>
        <SeedStagedState
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'ready', metadata: { title: null, pageCount: 0, pdfVersion: '1.7' } }]}
          selectedPaths={['/a.pdf']}
        >
          <BatchActions />
        </SeedStagedState>
      </Wrapper>
    );
    await act(async () => {});
    await user.click(screen.getByText('Import to library').closest('button'));
    await act(async () => {});
    expect(importPdf).toHaveBeenCalledWith('/a.pdf');
  });
});
