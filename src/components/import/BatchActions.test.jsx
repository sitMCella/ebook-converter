import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useEffect } from 'react';
import { MemoryRouter } from 'react-router-dom';
import { ImportProvider, useImportContext } from '../../contexts/ImportContext';
import { ConversionProvider } from '../../contexts/ConversionContext';
import { BatchActions } from './BatchActions';

const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return { ...actual, useNavigate: () => mockNavigate };
});

vi.mock('../../lib/tauri', async () => {
  const actual = await vi.importActual('../../lib/tauri');
  return {
    ...actual,
    convertPdfToEpub: vi.fn(),
    cancelConversion: vi.fn(),
    onConversionProgress: vi.fn().mockResolvedValue(() => {}),
    deleteBook: vi.fn().mockResolvedValue(undefined),
    listBooks: vi.fn().mockResolvedValue([]),
  };
});

vi.mock('../../lib/settings', async () => {
  const actual = await vi.importActual('../../lib/settings');
  return {
    ...actual,
    loadSettings: vi.fn(() => Promise.resolve({ ...actual.DEFAULT_SETTINGS })),
    saveSettings: vi.fn(() => Promise.resolve()),
    settingsToConversionOptions: vi.fn().mockReturnValue({}),
  };
});

import { deleteBook } from '../../lib/tauri';
import { SettingsProvider } from '../../contexts/SettingsContext';

function Wrapper({ children }) {
  return (
    <SettingsProvider>
      <ImportProvider>
        <ConversionProvider>
          <MemoryRouter>{children}</MemoryRouter>
        </ConversionProvider>
      </ImportProvider>
    </SettingsProvider>
  );
}

function SeedState({ files = [], selectedPaths = [], children }) {
  const { dispatch } = useImportContext();
  useEffect(() => {
    if (files.length > 0) {
      dispatch({ type: 'ADD_FILES', files });
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
  it('disables "Remove selected" when no rows are selected', () => {
    render(
      <Wrapper>
        <BatchActions />
      </Wrapper>
    );
    expect(screen.getByText('Remove selected').closest('button')).toBeDisabled();
  });

  it('disables "Convert selected" when no convertible rows are selected', () => {
    render(
      <Wrapper>
        <BatchActions />
      </Wrapper>
    );
    const convertBtn = screen.getByText('Convert selected').closest('button');
    expect(convertBtn).toBeDisabled();
  });

  it('enables "Remove selected" when rows are selected', () => {
    render(
      <Wrapper>
        <SeedState
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }]}
          selectedPaths={['/a.pdf']}
        >
          <BatchActions />
        </SeedState>
      </Wrapper>
    );
    expect(screen.getByText('Remove selected').closest('button')).not.toBeDisabled();
  });

  it('enables "Convert selected" when ready rows are selected', () => {
    render(
      <Wrapper>
        <SeedState
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }]}
          selectedPaths={['/a.pdf']}
        >
          <BatchActions />
        </SeedState>
      </Wrapper>
    );
    const convertBtn = screen.getByText('Convert selected').closest('button');
    expect(convertBtn).not.toBeDisabled();
  });

  it('keeps "Convert selected" disabled when only error files are selected', () => {
    render(
      <Wrapper>
        <SeedState
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'error' }]}
          selectedPaths={['/a.pdf']}
        >
          <BatchActions />
        </SeedState>
      </Wrapper>
    );
    const convertBtn = screen.getByText('Convert selected').closest('button');
    expect(convertBtn).toBeDisabled();
  });

  it('shows confirmation dialog when "Remove selected" is clicked', async () => {
    const user = userEvent.setup();
    render(
      <Wrapper>
        <SeedState
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }]}
          selectedPaths={['/a.pdf']}
        >
          <BatchActions />
        </SeedState>
      </Wrapper>
    );
    await user.click(screen.getByText('Remove selected'));
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(screen.getByText(/Remove 1 file\(s\)/)).toBeInTheDocument();
  });

  it('removes files on confirm', async () => {
    const user = userEvent.setup();
    render(
      <Wrapper>
        <SeedState
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }]}
          selectedPaths={['/a.pdf']}
        >
          <BatchActions />
        </SeedState>
      </Wrapper>
    );
    await user.click(screen.getByText('Remove selected'));
    await user.click(screen.getByText('Confirm'));
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('navigates to /converting when "Convert selected" is clicked', async () => {
    const user = userEvent.setup();
    render(
      <Wrapper>
        <SeedState
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }]}
          selectedPaths={['/a.pdf']}
        >
          <BatchActions />
        </SeedState>
      </Wrapper>
    );
    await user.click(screen.getByText('Convert selected').closest('button'));
    expect(mockNavigate).toHaveBeenCalledWith('/converting');
  });

  it('calls deleteBook when removing a file with bookId', async () => {
    const user = userEvent.setup();
    render(
      <Wrapper>
        <SeedState
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'ready', bookId: 'uuid-123' }]}
          selectedPaths={['/a.pdf']}
        >
          <BatchActions />
        </SeedState>
      </Wrapper>
    );
    await user.click(screen.getByText('Remove selected'));
    await user.click(screen.getByText('Confirm'));
    expect(deleteBook).toHaveBeenCalledWith('uuid-123');
  });

  it('does not call deleteBook when file has no bookId', async () => {
    const user = userEvent.setup();
    render(
      <Wrapper>
        <SeedState
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }]}
          selectedPaths={['/a.pdf']}
        >
          <BatchActions />
        </SeedState>
      </Wrapper>
    );
    await user.click(screen.getByText('Remove selected'));
    await user.click(screen.getByText('Confirm'));
    expect(deleteBook).not.toHaveBeenCalled();
  });

  it('still removes file from state when deleteBook fails', async () => {
    deleteBook.mockRejectedValue(new Error('disk error'));
    const spy = vi.spyOn(console, 'warn').mockImplementation(() => {});
    const user = userEvent.setup();
    render(
      <Wrapper>
        <SeedState
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'ready', bookId: 'uuid-123' }]}
          selectedPaths={['/a.pdf']}
        >
          <BatchActions />
        </SeedState>
      </Wrapper>
    );
    await user.click(screen.getByText('Remove selected'));
    await user.click(screen.getByText('Confirm'));
    expect(deleteBook).toHaveBeenCalledWith('uuid-123');
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    spy.mockRestore();
  });

  it('mentions stored copies in confirmation dialog', async () => {
    const user = userEvent.setup();
    render(
      <Wrapper>
        <SeedState
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }]}
          selectedPaths={['/a.pdf']}
        >
          <BatchActions />
        </SeedState>
      </Wrapper>
    );
    await user.click(screen.getByText('Remove selected'));
    expect(screen.getByText(/delete the stored copies/)).toBeInTheDocument();
  });
});
