import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { EpubMetadata } from './EpubMetadata';

const baseFile = {
  path: '/docs/design-patterns.pdf',
  name: 'Design patterns.pdf',
  conversionResult: {
    outputPath: '/output/Design patterns.epub',
    images: 47,
    fileSize: 3250585,
  },
};

async function expand() {
  const user = userEvent.setup();
  await user.click(screen.getByRole('button', { name: /metadata/i }));
}

describe('EpubMetadata', () => {
  it('renders metadata button collapsed by default', () => {
    render(<EpubMetadata file={baseFile} />);
    expect(screen.getByRole('button', { name: /metadata/i })).toBeInTheDocument();
  });

  it('shows summary with source name and size when collapsed', () => {
    render(<EpubMetadata file={baseFile} />);
    expect(screen.getByText(/Design patterns\.pdf · 3\.1 MB · 47 images/)).toBeInTheDocument();
  });

  it('shows source file name when expanded', async () => {
    render(<EpubMetadata file={baseFile} />);
    await expand();
    expect(screen.getByText('Source')).toBeInTheDocument();
    expect(screen.getByText('Design patterns.pdf')).toBeInTheDocument();
  });

  it('shows formatted EPUB size when expanded', async () => {
    render(<EpubMetadata file={baseFile} />);
    await expand();
    expect(screen.getByText('EPUB size')).toBeInTheDocument();
    expect(screen.getByText('3.1 MB')).toBeInTheDocument();
  });

  it('shows images with "extracted" suffix when expanded', async () => {
    render(<EpubMetadata file={baseFile} />);
    await expand();
    expect(screen.getByText('Images')).toBeInTheDocument();
    expect(screen.getByText('47 extracted')).toBeInTheDocument();
  });

  it('shows "Default" when no overrides', async () => {
    render(<EpubMetadata file={baseFile} />);
    await expand();
    expect(screen.getByText('Settings used')).toBeInTheDocument();
    expect(screen.getByText('Default')).toBeInTheDocument();
  });

  it('hides EPUB size row when fileSize is zero', async () => {
    const file = {
      ...baseFile,
      conversionResult: { ...baseFile.conversionResult, fileSize: 0 },
    };
    render(<EpubMetadata file={file} />);
    await expand();
    expect(screen.queryByText('EPUB size')).not.toBeInTheDocument();
  });

  it('hides Images row when images is zero', async () => {
    const file = {
      ...baseFile,
      conversionResult: { ...baseFile.conversionResult, images: 0 },
    };
    render(<EpubMetadata file={file} />);
    await expand();
    expect(screen.queryByText('Images')).not.toBeInTheDocument();
  });

  it('shows singular "1 override" for a single override', async () => {
    const file = {
      ...baseFile,
      overrides: { structure: { headingLevelThreshold: 2 } },
    };
    render(<EpubMetadata file={file} />);
    await expand();
    expect(screen.getByText('1 override')).toBeInTheDocument();
  });

  it('shows plural "2 overrides" for multiple overrides', async () => {
    const file = {
      ...baseFile,
      overrides: {
        structure: { headingLevelThreshold: 2 },
        images: { imageQuality: 'high' },
      },
    };
    render(<EpubMetadata file={file} />);
    await expand();
    expect(screen.getByText('2 overrides')).toBeInTheDocument();
  });

  it('shows "Default" when overrides object has empty groups', async () => {
    const file = { ...baseFile, overrides: { structure: {} } };
    render(<EpubMetadata file={file} />);
    await expand();
    expect(screen.getByText('Default')).toBeInTheDocument();
  });

  it('still shows Settings used row when conversionResult is missing', async () => {
    const file = { path: '/a.pdf', name: '', conversionResult: undefined };
    render(<EpubMetadata file={file} />);
    await expand();
    expect(screen.getByText('Settings used')).toBeInTheDocument();
    expect(screen.getByText('Default')).toBeInTheDocument();
    expect(screen.queryByText('Source')).not.toBeInTheDocument();
    expect(screen.queryByText('EPUB size')).not.toBeInTheDocument();
    expect(screen.queryByText('Images')).not.toBeInTheDocument();
  });

  it('shows Cover row when hasCover is true', async () => {
    const file = {
      ...baseFile,
      conversionResult: { ...baseFile.conversionResult, hasCover: true },
    };
    render(<EpubMetadata file={file} />);
    await expand();
    expect(screen.getByText('Cover')).toBeInTheDocument();
    expect(screen.getByText('Included')).toBeInTheDocument();
  });

  it('hides Cover row when hasCover is false', async () => {
    const file = {
      ...baseFile,
      conversionResult: { ...baseFile.conversionResult, hasCover: false },
    };
    render(<EpubMetadata file={file} />);
    await expand();
    expect(screen.queryByText('Cover')).not.toBeInTheDocument();
  });

  it('hides Cover row when hasCover is undefined', async () => {
    render(<EpubMetadata file={baseFile} />);
    await expand();
    expect(screen.queryByText('Cover')).not.toBeInTheDocument();
  });
});
