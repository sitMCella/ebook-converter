import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { EpubPreview } from './EpubPreview';

describe('EpubPreview', () => {
  it('shows placeholder text', () => {
    const file = { conversionResult: { chapters: 10 } };
    render(<EpubPreview file={file} />);
    expect(screen.getByText('EPUB preview not yet available')).toBeInTheDocument();
  });

  it('shows plural chapter count', () => {
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
