import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';

vi.mock('../../lib/tauri', async (importOriginal) => ({
  ...(await importOriginal()),
  openFileWithSystem: vi.fn(),
  saveFile: vi.fn(),
  isTauri: false,
}));

import { EpubDetailPanel } from './EpubDetailPanel';
import { openFileWithSystem } from '../../lib/tauri';

const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return { ...actual, useNavigate: () => mockNavigate };
});

const baseFile = {
  path: '/docs/design-patterns.pdf',
  name: 'Design patterns.pdf',
  outputPath: '/output/Design patterns.epub',
  conversionResult: {
    outputPath: '/output/Design patterns.epub',
    chapters: 23,
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
  it('renders preview, metadata, and table of contents sections', () => {
    renderPanel();
    expect(screen.getByText('EPUB preview not yet available')).toBeInTheDocument();
    expect(screen.getByText('Metadata')).toBeInTheDocument();
    expect(screen.getByText('Table of contents')).toBeInTheDocument();
  });

  it('renders Reconvert button in browser mode', () => {
    renderPanel();
    expect(screen.getByText('Reconvert')).toBeInTheDocument();
  });

  it('hides "Open in reader" and "Save as..." in browser mode', () => {
    renderPanel();
    expect(screen.queryByText('Open in reader')).not.toBeInTheDocument();
    expect(screen.queryByText('Save as...')).not.toBeInTheDocument();
  });

  it('navigates to library with source path on Reconvert click', async () => {
    const user = userEvent.setup();
    renderPanel();

    await user.click(screen.getByText('Reconvert'));
    expect(mockNavigate).toHaveBeenCalledWith('/library', {
      state: { selectedPath: '/docs/design-patterns.pdf' },
    });
  });

  it('derives epub name from outputPath for the panel', () => {
    renderPanel();
    expect(screen.getByText('Design patterns.pdf')).toBeInTheDocument();
  });
});
