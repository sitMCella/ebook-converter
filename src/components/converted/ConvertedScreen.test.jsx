import { describe, it, expect, vi } from 'vitest';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';

vi.mock('../../lib/tauri', async (importOriginal) => ({
  ...(await importOriginal()),
  listBooks: vi.fn().mockResolvedValue([]),
  openFileWithSystem: vi.fn(),
  openFolder: vi.fn(),
  isTauri: false,
}));

import { ImportProvider, useImportContext } from '../../contexts/ImportContext';
import { ConversionProvider } from '../../contexts/ConversionContext';
import { ConvertedScreen } from './ConvertedScreen';
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
    outputPath: '/output/Design patterns.epub',
    conversionResult: {
      outputPath: '/output/Design patterns.epub',
      chapters: 23,
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
  {
    path: '/docs/pragmatic-programmer.pdf',
    name: 'Pragmatic programmer.pdf',
    size: 15938355,
    status: 'converted',
    outputPath: '/output/Pragmatic programmer.epub',
    conversionResult: {
      outputPath: '/output/Pragmatic programmer.epub',
      chapters: 53,
      images: 12,
      fileSize: 4194304,
    },
    metadata: {
      title: 'The Pragmatic Programmer',
      author: 'David Thomas, Andrew Hunt',
      pageCount: 352,
      pdfVersion: '2.0',
      fileSize: 15938355,
    },
  },
];

const mixedFiles = [
  ...convertedFiles,
  {
    path: '/docs/clean-architecture.pdf',
    name: 'Clean architecture.pdf',
    size: 9123840,
    status: 'ready',
    metadata: { pageCount: 432, pdfVersion: '1.6', fileSize: 9123840 },
  },
];

function SeedFiles({ files, children }) {
  const { dispatch } = useImportContext();
  useEffect(() => {
    dispatch({ type: 'ADD_FILES', files });
  }, []);
  return children;
}

function renderConverted({ files = convertedFiles, initialPath = '/converted' } = {}) {
  return render(
    <ImportProvider>
      <ConversionProvider>
        <MemoryRouter initialEntries={[initialPath]}>
          <SeedFiles files={files}>
            <ConvertedScreen />
          </SeedFiles>
        </MemoryRouter>
      </ConversionProvider>
    </ImportProvider>,
  );
}

describe('ConvertedScreen', () => {
  it('shows empty state when no files are converted', () => {
    render(
      <ImportProvider>
        <ConversionProvider>
          <MemoryRouter>
            <ConvertedScreen />
          </MemoryRouter>
        </ConversionProvider>
      </ImportProvider>,
    );
    expect(screen.getByText(/no converted files yet/i)).toBeInTheDocument();
    expect(screen.getByText('Go to Import')).toBeInTheDocument();
  });

  it('navigates to /import when "Go to Import" is clicked', async () => {
    const user = userEvent.setup();
    render(
      <ImportProvider>
        <ConversionProvider>
          <MemoryRouter>
            <ConvertedScreen />
          </MemoryRouter>
        </ConversionProvider>
      </ImportProvider>,
    );
    await user.click(screen.getByText('Go to Import'));
    expect(mockNavigate).toHaveBeenCalledWith('/import');
  });

  it('only shows files with status converted', () => {
    renderConverted({ files: mixedFiles });
    const listbox = screen.getByRole('listbox');
    expect(within(listbox).getByText('Design patterns.epub')).toBeInTheDocument();
    expect(within(listbox).getByText('Pragmatic programmer.epub')).toBeInTheDocument();
    expect(within(listbox).queryByText('Clean architecture.pdf')).not.toBeInTheDocument();
  });

  it('renders header with title and search input', () => {
    renderConverted();
    expect(screen.getByText('Converted EPUBs')).toBeInTheDocument();
    expect(screen.getByPlaceholderText('Search converted...')).toBeInTheDocument();
  });

  it('auto-selects the first EPUB and shows its metadata', () => {
    renderConverted();
    expect(screen.getByText('Design patterns.pdf')).toBeInTheDocument();
    expect(screen.getByText('23')).toBeInTheDocument();
    expect(screen.getByText('47 extracted')).toBeInTheDocument();
  });

  it('switches detail panel when a different EPUB is clicked', async () => {
    const user = userEvent.setup();
    renderConverted();

    const listbox = screen.getByRole('listbox');
    await user.click(within(listbox).getByText('Pragmatic programmer.epub'));

    expect(screen.getByText('Pragmatic programmer.pdf')).toBeInTheDocument();
    expect(screen.getByText('53')).toBeInTheDocument();
    expect(screen.getByText('12 extracted')).toBeInTheDocument();
  });

  it('filters EPUB list by search query', async () => {
    const user = userEvent.setup();
    renderConverted();

    const searchInput = screen.getByPlaceholderText('Search converted...');
    await user.type(searchInput, 'pragmatic');

    const listbox = screen.getByRole('listbox');
    expect(within(listbox).queryByText('Design patterns.epub')).not.toBeInTheDocument();
    expect(within(listbox).getByText('Pragmatic programmer.epub')).toBeInTheDocument();
  });

  it('shows "No converted files match" when search has no results', async () => {
    const user = userEvent.setup();
    renderConverted();

    const searchInput = screen.getByPlaceholderText('Search converted...');
    await user.type(searchInput, 'nonexistent');

    expect(screen.getByText(/no converted files match/i)).toBeInTheDocument();
  });

  it('shows EPUB preview placeholder', () => {
    renderConverted();
    expect(screen.getByText(/epub preview not yet available/i)).toBeInTheDocument();
  });

  it('shows chapter count in preview', () => {
    renderConverted();
    expect(screen.getByText('23 chapters')).toBeInTheDocument();
  });

  it('shows metadata section with EPUB details', () => {
    renderConverted();
    expect(screen.getByText('Source')).toBeInTheDocument();
    expect(screen.getByText('EPUB size')).toBeInTheDocument();
    expect(screen.getByText('Chapters')).toBeInTheDocument();
    expect(screen.getByText('Images')).toBeInTheDocument();
    expect(screen.getByText('Settings used')).toBeInTheDocument();
    expect(screen.getByText('Default')).toBeInTheDocument();
  });

  it('shows settings override count when overrides exist', () => {
    const filesWithOverrides = [
      {
        ...convertedFiles[0],
        overrides: {
          structure: { headingLevelThreshold: 2 },
          images: { imageQuality: 'high' },
        },
      },
    ];
    renderConverted({ files: filesWithOverrides });
    expect(screen.getByText('2 overrides')).toBeInTheDocument();
  });

  it('hides metadata rows when values are zero', () => {
    const filesNoImages = [
      {
        ...convertedFiles[0],
        conversionResult: {
          ...convertedFiles[0].conversionResult,
          images: 0,
        },
      },
    ];
    renderConverted({ files: filesNoImages });
    expect(screen.queryByText('Images')).not.toBeInTheDocument();
  });

  it('shows table of contents empty state when no toc data', async () => {
    const user = userEvent.setup();
    renderConverted();

    const tocButton = screen.getByText('Table of contents');
    expect(tocButton).toBeInTheDocument();

    await user.click(tocButton);
    expect(screen.getByText(/no chapters detected/i)).toBeInTheDocument();
  });

  it('shows "Reconvert" button that navigates to library', async () => {
    const user = userEvent.setup();
    renderConverted();

    const reconvertButton = screen.getByText('Reconvert');
    await user.click(reconvertButton);

    expect(mockNavigate).toHaveBeenCalledWith('/library', {
      state: { selectedPath: '/docs/design-patterns.pdf' },
    });
  });

  it('derives epub name from outputPath', () => {
    renderConverted();
    const listbox = screen.getByRole('listbox');
    expect(within(listbox).getByText('Design patterns.epub')).toBeInTheDocument();
  });

  it('hides "Open folder" button in browser mode', () => {
    renderConverted();
    expect(screen.queryByText('Open folder')).not.toBeInTheDocument();
  });

  it('shows EPUB file size in list items', () => {
    renderConverted();
    const listbox = screen.getByRole('listbox');
    expect(within(listbox).getByText('3.1 MB')).toBeInTheDocument();
  });
});
