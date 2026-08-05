import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
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
    expect(screen.getByText('EPUB preview not yet available')).toBeInTheDocument();
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
    expect(screen.getByText('EPUB preview not yet available')).toBeInTheDocument();
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

  it('renders chapter content after loading', async () => {
    const tauri = await import('../../lib/tauri');
    tauri.isTauri = true;
    tauri.readEpubPreview.mockResolvedValue({
      chapters: [
        { title: 'Chapter 1', html: '<p>Hello world</p>' },
        { title: 'Chapter 2', html: '<p>Second chapter</p>' },
      ],
      coverImage: null,
    });

    const file = { outputPath: '/path/to/book.epub', conversionResult: { chapters: 2 } };
    render(<EpubPreview file={file} />);

    await waitFor(() => {
      expect(screen.getByText('Hello world')).toBeInTheDocument();
    });

    expect(screen.getByText('Chapter 1')).toBeInTheDocument();
    expect(screen.getByText('(1/2)')).toBeInTheDocument();

    tauri.isTauri = false;
  });

  it('navigates between chapters', async () => {
    const user = userEvent.setup();
    const tauri = await import('../../lib/tauri');
    tauri.isTauri = true;
    tauri.readEpubPreview.mockResolvedValue({
      chapters: [
        { title: 'First', html: '<p>Page one</p>' },
        { title: 'Second', html: '<p>Page two</p>' },
      ],
      coverImage: null,
    });

    const file = { outputPath: '/path/to/book.epub', conversionResult: { chapters: 2 } };
    render(<EpubPreview file={file} />);

    await waitFor(() => {
      expect(screen.getByText('Page one')).toBeInTheDocument();
    });

    const nextButton = screen.getByLabelText('Next chapter');
    await user.click(nextButton);

    expect(screen.getByText('Page two')).toBeInTheDocument();
    expect(screen.getByText('Second')).toBeInTheDocument();
    expect(screen.getByText('(2/2)')).toBeInTheDocument();

    tauri.isTauri = false;
  });

  it('shows cover image when available', async () => {
    const tauri = await import('../../lib/tauri');
    tauri.isTauri = true;
    tauri.readEpubPreview.mockResolvedValue({
      chapters: [{ title: 'Ch1', html: '<p>Content</p>' }],
      coverImage: 'data:image/jpeg;base64,abc123',
    });

    const file = { outputPath: '/path/to/book.epub', conversionResult: { chapters: 1 } };
    render(<EpubPreview file={file} />);

    await waitFor(() => {
      expect(screen.getByAltText('Cover')).toBeInTheDocument();
    });

    const img = screen.getByAltText('Cover');
    expect(img).toHaveAttribute('src', 'data:image/jpeg;base64,abc123');

    tauri.isTauri = false;
  });

  it('shows placeholder on error', async () => {
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

  it('disables previous button on first chapter', async () => {
    const tauri = await import('../../lib/tauri');
    tauri.isTauri = true;
    tauri.readEpubPreview.mockResolvedValue({
      chapters: [
        { title: 'Ch1', html: '<p>First</p>' },
        { title: 'Ch2', html: '<p>Second</p>' },
      ],
      coverImage: null,
    });

    const file = { outputPath: '/path/to/book.epub', conversionResult: { chapters: 2 } };
    render(<EpubPreview file={file} />);

    await waitFor(() => {
      expect(screen.getByText('First')).toBeInTheDocument();
    });

    expect(screen.getByLabelText('Previous chapter')).toBeDisabled();
    expect(screen.getByLabelText('Next chapter')).not.toBeDisabled();

    tauri.isTauri = false;
  });

  it('disables next button on last chapter', async () => {
    const user = userEvent.setup();
    const tauri = await import('../../lib/tauri');
    tauri.isTauri = true;
    tauri.readEpubPreview.mockResolvedValue({
      chapters: [
        { title: 'Ch1', html: '<p>First</p>' },
        { title: 'Ch2', html: '<p>Second</p>' },
      ],
      coverImage: null,
    });

    const file = { outputPath: '/path/to/book.epub', conversionResult: { chapters: 2 } };
    render(<EpubPreview file={file} />);

    await waitFor(() => {
      expect(screen.getByText('First')).toBeInTheDocument();
    });

    await user.click(screen.getByLabelText('Next chapter'));

    expect(screen.getByLabelText('Next chapter')).toBeDisabled();
    expect(screen.getByLabelText('Previous chapter')).not.toBeDisabled();

    tauri.isTauri = false;
  });
});
