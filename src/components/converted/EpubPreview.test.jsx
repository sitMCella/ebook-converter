import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { EpubPreview } from './EpubPreview';

vi.mock('../../lib/tauri', () => ({
  isTauri: false,
  readEpubPreview: vi.fn(),
}));

describe('EpubPreview', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('shows placeholder when not in Tauri', () => {
    const file = { conversionResult: { chapters: 10 } };
    render(<EpubPreview file={file} />);
    expect(screen.getByText('No cover image available')).toBeInTheDocument();
  });

  it('shows plural chapter count in placeholder', () => {
    const file = { conversionResult: { chapters: 23 } };
    render(<EpubPreview file={file} />);
    expect(screen.getByText('23 chapters')).toBeInTheDocument();
  });

  it('shows singular chapter count for one chapter', () => {
    const file = { conversionResult: { chapters: 1 } };
    render(<EpubPreview file={file} />);
    expect(screen.getByText('1 chapter')).toBeInTheDocument();
  });

  it('hides chapter count when chapters is zero', () => {
    const file = { conversionResult: { chapters: 0 } };
    render(<EpubPreview file={file} />);
    expect(screen.queryByText(/chapter/i)).not.toBeInTheDocument();
  });

  it('hides chapter count when conversionResult is missing', () => {
    const file = {};
    render(<EpubPreview file={file} />);
    expect(screen.queryByText(/chapter/i)).not.toBeInTheDocument();
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
    tauri.readEpubPreview.mockReturnValue(new Promise(() => {}));

    const file = { outputPath: '/path/to/book.epub', conversionResult: { chapters: 5 } };
    render(<EpubPreview file={file} />);
    expect(screen.getByText('Loading preview...')).toBeInTheDocument();

    tauri.isTauri = false;
  });

  it('renders cover image after loading', async () => {
    const tauri = await import('../../lib/tauri');
    tauri.isTauri = true;
    tauri.readEpubPreview.mockResolvedValue({
      coverImage: 'data:image/jpeg;base64,abc123',
    });

    const file = { outputPath: '/path/to/book.epub', conversionResult: { chapters: 2 } };
    render(<EpubPreview file={file} />);

    await waitFor(() => {
      expect(screen.getByAltText('Cover')).toBeInTheDocument();
    });

    const img = screen.getByAltText('Cover');
    expect(img).toHaveAttribute('src', 'data:image/jpeg;base64,abc123');

    tauri.isTauri = false;
  });

  it('shows placeholder when no cover image', async () => {
    const tauri = await import('../../lib/tauri');
    tauri.isTauri = true;
    tauri.readEpubPreview.mockResolvedValue({
      coverImage: null,
    });

    const file = { outputPath: '/path/to/book.epub', conversionResult: { chapters: 3 } };
    render(<EpubPreview file={file} />);

    await waitFor(() => {
      expect(screen.getByText('No cover image available')).toBeInTheDocument();
    });

    expect(screen.getByText('3 chapters')).toBeInTheDocument();

    tauri.isTauri = false;
  });

  it('shows error message on failure', async () => {
    const tauri = await import('../../lib/tauri');
    tauri.isTauri = true;
    tauri.readEpubPreview.mockRejectedValue(new Error('Failed to read'));

    const file = { outputPath: '/path/to/book.epub', conversionResult: { chapters: 3 } };
    render(<EpubPreview file={file} />);

    await waitFor(() => {
      expect(screen.getByText('Failed to read')).toBeInTheDocument();
    });

    tauri.isTauri = false;
  });
});
