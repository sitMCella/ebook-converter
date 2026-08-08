import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { DocumentListItem } from './DocumentListItem';

const baseFile = {
  path: '/docs/design-patterns.pdf',
  name: 'Design patterns.pdf',
  size: 13003776,
  status: 'ready',
  metadata: {
    title: 'Design Patterns',
    author: 'Gamma, Helm, Johnson, Vlissides',
    pageCount: 384,
    fileSize: 13003776,
  },
};

describe('DocumentListItem', () => {
  it('displays the metadata title instead of the filename', () => {
    render(<DocumentListItem file={baseFile} selected={false} onSelect={() => {}} />);
    expect(screen.getByText('Design Patterns')).toBeInTheDocument();
    expect(screen.queryByText('Design patterns.pdf')).not.toBeInTheDocument();
  });

  it('falls back to filename when metadata title is absent', () => {
    const file = { ...baseFile, metadata: { ...baseFile.metadata, title: null } };
    render(<DocumentListItem file={file} selected={false} onSelect={() => {}} />);
    expect(screen.getByText('Design patterns.pdf')).toBeInTheDocument();
  });

  it('falls back to filename when metadata is missing entirely', () => {
    const file = { ...baseFile, metadata: undefined };
    render(<DocumentListItem file={file} selected={false} onSelect={() => {}} />);
    expect(screen.getByText('Design patterns.pdf')).toBeInTheDocument();
  });

  it('shows page count and file size in secondary line', () => {
    render(<DocumentListItem file={baseFile} selected={false} onSelect={() => {}} />);
    expect(screen.getByText('384 pages · 12.4 MB')).toBeInTheDocument();
  });

  it('shows only page count when file size is zero', () => {
    const file = { ...baseFile, size: 0, metadata: { ...baseFile.metadata, fileSize: 0 } };
    render(<DocumentListItem file={file} selected={false} onSelect={() => {}} />);
    expect(screen.getByText('384 pages')).toBeInTheDocument();
  });

  it('shows only file size when page count is absent', () => {
    const file = { ...baseFile, metadata: { ...baseFile.metadata, pageCount: undefined } };
    render(<DocumentListItem file={file} selected={false} onSelect={() => {}} />);
    expect(screen.getByText('12.4 MB')).toBeInTheDocument();
    expect(screen.queryByText(/pages/)).not.toBeInTheDocument();
  });

  it('hides secondary line when both page count and file size are absent', () => {
    const file = { ...baseFile, size: 0, metadata: { fileSize: 0 } };
    render(<DocumentListItem file={file} selected={false} onSelect={() => {}} />);
    const option = screen.getByRole('option');
    expect(option.querySelectorAll('span')).toHaveLength(1);
  });

  it('sets title attribute for tooltip on long names', () => {
    render(<DocumentListItem file={baseFile} selected={false} onSelect={() => {}} />);
    expect(screen.getByTitle('Design Patterns')).toBeInTheDocument();
  });

  it('sets aria-selected true when selected', () => {
    render(<DocumentListItem file={baseFile} selected={true} onSelect={() => {}} />);
    expect(screen.getByRole('option')).toHaveAttribute('aria-selected', 'true');
  });

  it('sets aria-selected false when not selected', () => {
    render(<DocumentListItem file={baseFile} selected={false} onSelect={() => {}} />);
    expect(screen.getByRole('option')).toHaveAttribute('aria-selected', 'false');
  });

  it('calls onSelect when clicked', async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();
    render(<DocumentListItem file={baseFile} selected={false} onSelect={onSelect} />);

    await user.click(screen.getByRole('option'));
    expect(onSelect).toHaveBeenCalledOnce();
  });

  it('uses file.size as fallback when metadata.fileSize is missing', () => {
    const file = { ...baseFile, size: 5242880, metadata: { ...baseFile.metadata, fileSize: undefined } };
    render(<DocumentListItem file={file} selected={false} onSelect={() => {}} />);
    expect(screen.getByText('384 pages · 5.0 MB')).toBeInTheDocument();
  });
});
