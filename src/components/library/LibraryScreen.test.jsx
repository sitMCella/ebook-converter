import { describe, it, expect, vi } from 'vitest';
import { render, screen, within, act } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';

vi.mock('../../lib/tauri', async (importOriginal) => ({
  ...(await importOriginal()),
  listBooks: vi.fn().mockResolvedValue([]),
  getPdfCover: vi.fn().mockResolvedValue({ coverImage: null }),
}));

vi.mock('../../lib/settings', async () => {
  const actual = await vi.importActual('../../lib/settings');
  return {
    ...actual,
    loadSettings: vi.fn(() => Promise.resolve({ ...actual.DEFAULT_SETTINGS })),
    saveSettings: vi.fn(() => Promise.resolve()),
  };
});

import { ImportProvider, useImportContext } from '../../contexts/ImportContext';
import { ConversionProvider } from '../../contexts/ConversionContext';
import { SettingsProvider } from '../../contexts/SettingsContext';
import { LibraryScreen } from './LibraryScreen';
import { useEffect } from 'react';

const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return { ...actual, useNavigate: () => mockNavigate };
});

const testFiles = [
  {
    path: '/docs/design-patterns.pdf',
    name: 'Design patterns.pdf',
    size: 13003776,
    status: 'ready',
    metadata: {
      title: 'Design Patterns',
      author: 'Gamma, Helm, Johnson, Vlissides',
      pageCount: 384,
      pdfVersion: '1.7',
      createdDate: '1994-10-21',
      modifiedDate: '2004-03-15',
      producer: 'Adobe Acrobat 6.0',
      fileSize: 13003776,
    },
  },
  {
    path: '/docs/clean-architecture.pdf',
    name: 'Clean architecture.pdf',
    size: 9123840,
    status: 'ready',
    metadata: {
      title: 'Clean Architecture',
      author: 'Robert C. Martin',
      pageCount: 432,
      pdfVersion: '1.6',
      createdDate: '2017-09-12',
      producer: 'LaTeX',
      fileSize: 9123840,
    },
  },
  {
    path: '/docs/pragmatic-programmer.pdf',
    name: 'Pragmatic programmer.pdf',
    size: 15938355,
    status: 'converted',
    metadata: {
      title: 'The Pragmatic Programmer',
      author: 'David Thomas, Andrew Hunt',
      pageCount: 352,
      pdfVersion: '2.0',
      createdDate: '2019-09-20',
      fileSize: 15938355,
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

async function renderLibrary({ files = testFiles, initialPath = '/library' } = {}) {
  const result = render(
    <SettingsProvider>
      <ImportProvider>
        <ConversionProvider>
          <MemoryRouter initialEntries={[initialPath]}>
            <SeedFiles files={files}>
              <LibraryScreen />
            </SeedFiles>
          </MemoryRouter>
        </ConversionProvider>
      </ImportProvider>
    </SettingsProvider>,
  );
  await act(async () => {});
  return result;
}

describe('LibraryScreen', () => {
  it('shows empty state when no files are imported', async () => {
    render(
      <SettingsProvider>
        <ImportProvider>
          <ConversionProvider>
            <MemoryRouter>
              <LibraryScreen />
            </MemoryRouter>
          </ConversionProvider>
        </ImportProvider>
      </SettingsProvider>,
    );
    await act(async () => {});
    expect(screen.getByText(/your library is empty/i)).toBeInTheDocument();
    expect(screen.getByText('Go to Import')).toBeInTheDocument();
  });

  it('navigates to /import when "Go to Import" is clicked', async () => {
    const user = userEvent.setup();
    render(
      <SettingsProvider>
        <ImportProvider>
          <ConversionProvider>
            <MemoryRouter>
              <LibraryScreen />
            </MemoryRouter>
          </ConversionProvider>
        </ImportProvider>
      </SettingsProvider>,
    );
    await act(async () => {});
    await user.click(screen.getByText('Go to Import'));
    expect(mockNavigate).toHaveBeenCalledWith('/import');
  });

  it('renders document list with all imported files', async () => {
    await renderLibrary();
    const listbox = screen.getByRole('listbox');
    expect(within(listbox).getByText('Design Patterns')).toBeInTheDocument();
    expect(within(listbox).getByText('Clean Architecture')).toBeInTheDocument();
    expect(within(listbox).getByText('The Pragmatic Programmer')).toBeInTheDocument();
  });

  it('renders header with title and search input', async () => {
    await renderLibrary();
    expect(screen.getByText('Library')).toBeInTheDocument();
    expect(screen.getByPlaceholderText('Search documents...')).toBeInTheDocument();
  });

  it('auto-selects the first document and shows metadata summary', async () => {
    await renderLibrary();
    expect(screen.getByText(/Design Patterns · 384 pages/)).toBeInTheDocument();
  });

  it('expands metadata section to show all rows', async () => {
    const user = userEvent.setup();
    await renderLibrary();

    await user.click(screen.getByRole('button', { name: /metadata/i }));

    expect(screen.getAllByText('Design Patterns').length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText('Gamma, Helm, Johnson, Vlissides')).toBeInTheDocument();
    expect(screen.getByText('384')).toBeInTheDocument();
  });

  it('switches detail panel when a different document is clicked', async () => {
    const user = userEvent.setup();
    await renderLibrary();

    const listbox = screen.getByRole('listbox');
    const secondItem = within(listbox).getByText('Clean Architecture');
    await user.click(secondItem);

    expect(screen.getByText(/Clean Architecture · 432 pages/)).toBeInTheDocument();
  });

  it('filters document list by search query', async () => {
    const user = userEvent.setup();
    await renderLibrary();

    const searchInput = screen.getByPlaceholderText('Search documents...');
    await user.type(searchInput, 'clean');

    expect(screen.queryByText('Design Patterns')).not.toBeInTheDocument();
    expect(screen.getByText('Clean Architecture')).toBeInTheDocument();
    expect(screen.queryByText('The Pragmatic Programmer')).not.toBeInTheDocument();
  });

  it('shows "No documents match" when search has no results', async () => {
    const user = userEvent.setup();
    await renderLibrary();

    const searchInput = screen.getByPlaceholderText('Search documents...');
    await user.type(searchInput, 'nonexistent');

    expect(screen.getByText(/no documents match/i)).toBeInTheDocument();
  });

  it('shows error badge only for error documents', async () => {
    await renderLibrary({
      files: [
        { path: '/a.pdf', name: 'a.pdf', size: 1024, status: 'ready', metadata: { fileSize: 1024 } },
        { path: '/b.pdf', name: 'b.pdf', size: 1024, status: 'error', errorMessage: 'Corrupted', metadata: { fileSize: 1024 } },
      ],
    });
    const badges = screen.getAllByText('Error');
    expect(badges).toHaveLength(1);
    expect(screen.queryByText('Ready')).not.toBeInTheDocument();
  });

  it('shows page preview placeholder when no cover image is available', async () => {
    await renderLibrary();
    expect(screen.getByText(/no cover image available/i)).toBeInTheDocument();
  });

  it('hides metadata rows when values are absent', async () => {
    const user = userEvent.setup();
    await renderLibrary({
      files: [
        {
          path: '/docs/no-meta.pdf',
          name: 'no-meta.pdf',
          size: 1024,
          status: 'ready',
          metadata: { pageCount: 10, pdfVersion: '1.4', fileSize: 1024 },
        },
      ],
    });

    await user.click(screen.getByRole('button', { name: /metadata/i }));

    expect(screen.queryByText('Title')).not.toBeInTheDocument();
    expect(screen.queryByText('Authors')).not.toBeInTheDocument();
    expect(screen.getByText('10')).toBeInTheDocument();
  });

  it('shows "Convert to EPUB" button for ready documents', async () => {
    await renderLibrary();
    expect(screen.getByText('Convert to EPUB')).toBeInTheDocument();
  });

  it('shows "Reconvert to EPUB" for converted documents', async () => {
    const user = userEvent.setup();
    await renderLibrary();

    const listbox = screen.getByRole('listbox');
    await user.click(within(listbox).getByText('The Pragmatic Programmer'));

    expect(screen.getByText('Reconvert to EPUB')).toBeInTheDocument();
  });

  it('shows "View EPUB" button for converted documents', async () => {
    const user = userEvent.setup();
    await renderLibrary();

    const listbox = screen.getByRole('listbox');
    await user.click(within(listbox).getByText('The Pragmatic Programmer'));

    expect(screen.getByText('View EPUB')).toBeInTheDocument();
  });

  it('does not show "View EPUB" button for ready documents', async () => {
    await renderLibrary();
    expect(screen.queryByText('View EPUB')).not.toBeInTheDocument();
  });

  it('navigates to /converted when "View EPUB" is clicked', async () => {
    const user = userEvent.setup();
    await renderLibrary();

    const listbox = screen.getByRole('listbox');
    await user.click(within(listbox).getByText('The Pragmatic Programmer'));
    await user.click(screen.getByText('View EPUB'));

    expect(mockNavigate).toHaveBeenCalledWith('/converted', {
      state: { selectedPath: '/docs/pragmatic-programmer.pdf' },
    });
  });
});
