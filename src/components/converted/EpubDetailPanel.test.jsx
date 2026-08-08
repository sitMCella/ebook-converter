import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';

vi.mock('../../lib/tauri', async (importOriginal) => ({
  ...(await importOriginal()),
  openFileWithSystem: vi.fn(),
  isTauri: false,
}));

import { EpubDetailPanel } from './EpubDetailPanel';

const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return { ...actual, useNavigate: () => mockNavigate };
});

const baseFile = {
  path: '/docs/design-patterns.pdf',
  name: 'Design patterns.pdf',
  outputPath: '/output/Design patterns.epub',
  metadata: {
    title: 'Design Patterns',
    pageCount: 384,
    fileSize: 13003776,
  },
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

describe('EpubDetailPanel', () => {
  it('renders preview and metadata sections', () => {
    renderPanel();
    expect(screen.getByText('No cover image available')).toBeInTheDocument();
    expect(screen.getByText('Metadata')).toBeInTheDocument();
  });

  it('renders Reconvert button in browser mode', () => {
    renderPanel();
    expect(screen.getByText('Reconvert')).toBeInTheDocument();
  });

  it('hides "Open in reader" in browser mode', () => {
    renderPanel();
    expect(screen.queryByText('Open in reader')).not.toBeInTheDocument();
  });

  it('hides "Open folder" in browser mode', () => {
    renderPanel();
    expect(screen.queryByText('Open folder')).not.toBeInTheDocument();
  });

  it('navigates to library with source path on Reconvert click', async () => {
    const user = userEvent.setup();
    renderPanel();

    await user.click(screen.getByText('Reconvert'));
    expect(mockNavigate).toHaveBeenCalledWith('/library', {
      state: { selectedPath: '/docs/design-patterns.pdf' },
    });
  });

  it('shows book title and page count in collapsed metadata summary', () => {
    renderPanel();
    expect(screen.getByText(/Design Patterns · 384 pages/)).toBeInTheDocument();
  });
});
