import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { EpubPreview } from './EpubPreview';

vi.mock('../../lib/tauri', () => ({
  isTauri: false,
  getPdfCover: vi.fn(),
}));

describe('EpubPreview', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('shows placeholder when not in Tauri', () => {
    const file = { conversionResult: {} };
    render(<EpubPreview file={file} />);
    expect(screen.getByText('No cover image available')).toBeInTheDocument();
  });

  it('shows placeholder when storedPdfPath is missing', () => {
    const file = {};
    render(<EpubPreview file={file} />);
    expect(screen.getByText('No cover image available')).toBeInTheDocument();
  });
});

describe('EpubPreview in Tauri', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('shows loading state while fetching', async () => {
    const tauri = await import('../../lib/tauri');
    tauri.isTauri = true;
    tauri.getPdfCover.mockReturnValue(new Promise(() => {}));

    const file = { storedPdfPath: '/path/to/book.pdf', conversionResult: {} };
    render(<EpubPreview file={file} />);
    expect(screen.getByText('Loading preview...')).toBeInTheDocument();

    tauri.isTauri = false;
  });

  it('renders cover image after loading', async () => {
    const tauri = await import('../../lib/tauri');
    tauri.isTauri = true;
    tauri.getPdfCover.mockResolvedValue({
      coverImage: 'data:image/png;base64,abc123',
    });

    const file = { storedPdfPath: '/path/to/book.pdf', conversionResult: {} };
    render(<EpubPreview file={file} />);

    await waitFor(() => {
      expect(screen.getByAltText('Cover')).toBeInTheDocument();
    });

    const img = screen.getByAltText('Cover');
    expect(img).toHaveAttribute('src', 'data:image/png;base64,abc123');

    tauri.isTauri = false;
  });

  it('shows placeholder when no cover image', async () => {
    const tauri = await import('../../lib/tauri');
    tauri.isTauri = true;
    tauri.getPdfCover.mockResolvedValue({
      coverImage: null,
    });

    const file = { storedPdfPath: '/path/to/book.pdf', conversionResult: {} };
    render(<EpubPreview file={file} />);

    await waitFor(() => {
      expect(screen.getByText('No cover image available')).toBeInTheDocument();
    });

    tauri.isTauri = false;
  });
});
