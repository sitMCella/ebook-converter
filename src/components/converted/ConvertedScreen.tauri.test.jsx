import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';

vi.mock('../../lib/tauri', async (importOriginal) => ({
  ...(await importOriginal()),
  listBooks: vi.fn().mockResolvedValue([]),
  openFileWithSystem: vi.fn(),
  openFolder: vi.fn(),
  getBookDir: vi.fn().mockResolvedValue('/app-data/books/abc-123'),
  isTauri: true,
}));

import { ImportProvider, useImportContext } from '../../contexts/ImportContext';
import { ConversionProvider } from '../../contexts/ConversionContext';
import { ConvertedScreen } from './ConvertedScreen';
import { openFolder, getBookDir } from '../../lib/tauri';
import { useEffect } from 'react';

const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return { ...actual, useNavigate: () => mockNavigate };
});

const convertedFiles = [
  {
    path: '/docs/design-patterns.pdf',
    name: 'Design patterns.pdf',
    size: 13003776,
    status: 'converted',
    bookId: 'abc-123',
    outputPath: '/app-data/books/abc-123/Design patterns.epub',
    conversionResult: {
      outputPath: '/app-data/books/abc-123/Design patterns.epub',
      images: 47,
      fileSize: 3250585,
    },
    metadata: {
      title: 'Design Patterns',
      author: 'Gamma, Helm, Johnson, Vlissides',
      pageCount: 384,
      pdfVersion: '1.7',
      fileSize: 13003776,
    },
  },
];

function SeedFiles({ files, children }) {
  const { dispatch } = useImportContext();
  useEffect(() => {
    dispatch({ type: 'ADD_FILES', files });
  }, []);
  return children;
}

function renderConverted({ files = convertedFiles } = {}) {
  return render(
    <ImportProvider>
      <ConversionProvider>
        <MemoryRouter initialEntries={['/converted']}>
          <SeedFiles files={files}>
            <ConvertedScreen />
          </SeedFiles>
        </MemoryRouter>
      </ConversionProvider>
    </ImportProvider>,
  );
}

describe('ConvertedScreen (Tauri mode)', () => {
  beforeEach(() => vi.clearAllMocks());

  it('shows "Open folder" button in the detail panel', () => {
    renderConverted();
    expect(screen.getByText('Open folder')).toBeInTheDocument();
  });

  it('calls openFolder with the book directory on click', async () => {
    const user = userEvent.setup();
    renderConverted();

    await user.click(screen.getByText('Open folder'));
    expect(getBookDir).toHaveBeenCalledWith('abc-123');
    expect(openFolder).toHaveBeenCalledWith('/app-data/books/abc-123');
  });

  it('does not call openFolder when getBookDir returns empty string', async () => {
    getBookDir.mockResolvedValueOnce('');
    const user = userEvent.setup();
    renderConverted();

    await user.click(screen.getByText('Open folder'));
    expect(openFolder).not.toHaveBeenCalled();
  });
});
