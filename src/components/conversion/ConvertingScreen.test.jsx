import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import { ImportProvider, useImportContext } from '../../contexts/ImportContext';
import { ConversionProvider, useConversionContext } from '../../contexts/ConversionContext';
import { ConvertingScreen } from './ConvertingScreen';
import { useEffect } from 'react';

vi.mock('../../lib/tauri', () => ({
  convertPdfToEpub: vi.fn(),
  cancelConversion: vi.fn().mockResolvedValue(undefined),
  onConversionProgress: vi.fn().mockResolvedValue(() => {}),
  listBooks: vi.fn().mockResolvedValue([]),
}));

vi.mock('../../lib/settings', () => ({
  loadSettings: vi.fn().mockResolvedValue({}),
  settingsToConversionOptions: vi.fn().mockReturnValue({}),
}));

const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return { ...actual, useNavigate: () => mockNavigate };
});

function SeedConversion({ files, paths, complete, children }) {
  const { dispatch: importDispatch } = useImportContext();
  const { dispatch: conversionDispatch } = useConversionContext();
  useEffect(() => {
    if (files) {
      importDispatch({ type: 'ADD_FILES', files });
    }
    if (paths) {
      conversionDispatch({ type: 'ENQUEUE_FILES', paths });
    }
    if (complete) {
      conversionDispatch({ type: 'COMPLETE_ACTIVE', path: paths[0] });
      conversionDispatch({ type: 'START_NEXT' });
    }
  }, []);
  return children;
}

function Wrapper({ children }) {
  return (
    <ImportProvider>
      <ConversionProvider>
        <MemoryRouter>{children}</MemoryRouter>
      </ConversionProvider>
    </ImportProvider>
  );
}

beforeEach(() => {
  Element.prototype.scrollIntoView = () => {};
});

describe('ConvertingScreen', () => {
  it('shows "Converting" heading when active', () => {
    render(
      <Wrapper>
        <SeedConversion
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'converting' }]}
          paths={['/a.pdf']}
        >
          <ConvertingScreen />
        </SeedConversion>
      </Wrapper>,
    );
    expect(screen.getByRole('heading', { name: 'Converting' })).toBeInTheDocument();
  });

  it('shows "Cancel all" button when files are queued', () => {
    render(
      <Wrapper>
        <SeedConversion
          files={[
            { path: '/a.pdf', name: 'a.pdf', status: 'converting' },
            { path: '/b.pdf', name: 'b.pdf', status: 'converting' },
          ]}
          paths={['/a.pdf', '/b.pdf']}
        >
          <ConvertingScreen />
        </SeedConversion>
      </Wrapper>,
    );
    expect(screen.getByText('Cancel all')).toBeInTheDocument();
  });

  it('shows confirmation dialog when "Cancel all" is clicked', async () => {
    const user = userEvent.setup();
    render(
      <Wrapper>
        <SeedConversion
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'converting' }]}
          paths={['/a.pdf']}
        >
          <ConvertingScreen />
        </SeedConversion>
      </Wrapper>,
    );
    await user.click(screen.getByText('Cancel all'));
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(screen.getByText(/Cancel 1 remaining conversion/)).toBeInTheDocument();
  });

  it('shows "Conversion complete" and "View converted" when done', () => {
    render(
      <Wrapper>
        <SeedConversion
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'converted' }]}
          paths={['/a.pdf']}
          complete
        >
          <ConvertingScreen />
        </SeedConversion>
      </Wrapper>,
    );
    expect(screen.getByText('Conversion complete')).toBeInTheDocument();
    expect(screen.getByText('View converted')).toBeInTheDocument();
  });

  it('navigates to /converted when "View converted" is clicked', async () => {
    const user = userEvent.setup();
    render(
      <Wrapper>
        <SeedConversion
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'converted' }]}
          paths={['/a.pdf']}
          complete
        >
          <ConvertingScreen />
        </SeedConversion>
      </Wrapper>,
    );
    await user.click(screen.getByText('View converted'));
    expect(mockNavigate).toHaveBeenCalledWith('/converted');
  });

  it('hides "Cancel all" when conversion is complete', () => {
    render(
      <Wrapper>
        <SeedConversion
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'converted' }]}
          paths={['/a.pdf']}
          complete
        >
          <ConvertingScreen />
        </SeedConversion>
      </Wrapper>,
    );
    expect(screen.queryByText('Cancel all')).not.toBeInTheDocument();
  });
});
