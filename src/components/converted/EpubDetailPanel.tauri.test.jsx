import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';

vi.mock('../../lib/tauri', async (importOriginal) => ({
  ...(await importOriginal()),
  openFileWithSystem: vi.fn(),
  openFolder: vi.fn(),
  getBookDir: vi.fn().mockResolvedValue('/app-data/books/book-uuid-1'),
  isTauri: true,
}));

import { EpubDetailPanel } from './EpubDetailPanel';
import { openFileWithSystem, openFolder, getBookDir } from '../../lib/tauri';

const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return { ...actual, useNavigate: () => mockNavigate };
});

const baseFile = {
  path: '/docs/design-patterns.pdf',
  name: 'Design patterns.pdf',
  bookId: 'book-uuid-1',
  outputPath: '/output/Design patterns.epub',
  conversionResult: {
    outputPath: '/output/Design patterns.epub',
    images: 47,
    fileSize: 3250585,
  },
};

function renderPanel(file = baseFile) {
  return render(
    <MemoryRouter>
      <EpubDetailPanel file={file} />
    </MemoryRouter>,
  );
}

describe('EpubDetailPanel (Tauri mode)', () => {
  beforeEach(() => vi.clearAllMocks());
  it('shows "Open in reader" button when isTauri is true', () => {
    renderPanel();
    expect(screen.getByText('Open in reader')).toBeInTheDocument();
  });

  it('calls openFileWithSystem with outputPath on "Open in reader" click', async () => {
    const user = userEvent.setup();
    renderPanel();

    await user.click(screen.getByText('Open in reader'));
    expect(openFileWithSystem).toHaveBeenCalledWith('/output/Design patterns.epub');
  });

  it('does not call openFileWithSystem when outputPath is missing', async () => {
    const user = userEvent.setup();
    renderPanel({ ...baseFile, outputPath: undefined });

    await user.click(screen.getByText('Open in reader'));
    expect(openFileWithSystem).not.toHaveBeenCalled();
  });

  it('shows "Open folder" button when file has bookId', () => {
    renderPanel();
    expect(screen.getByText('Open folder')).toBeInTheDocument();
  });

  it('calls openFolder with the book directory on click', async () => {
    const user = userEvent.setup();
    renderPanel();

    await user.click(screen.getByText('Open folder'));
    expect(getBookDir).toHaveBeenCalledWith('book-uuid-1');
    expect(openFolder).toHaveBeenCalledWith('/app-data/books/book-uuid-1');
  });

  it('does not show "Open folder" when bookId is missing', () => {
    renderPanel({ ...baseFile, bookId: undefined });
    expect(screen.queryByText('Open folder')).not.toBeInTheDocument();
  });
});
