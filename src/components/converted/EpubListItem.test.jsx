import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { EpubListItem } from './EpubListItem';

const baseFile = {
  path: '/docs/design-patterns.pdf',
  name: 'Design patterns.pdf',
  outputPath: '/output/Design patterns.epub',
  conversionResult: { fileSize: 3250585 },
};

describe('EpubListItem', () => {
  it('derives EPUB name from outputPath', () => {
    render(<EpubListItem file={baseFile} selected={false} onSelect={() => {}} />);
    expect(screen.getByText('Design patterns.epub')).toBeInTheDocument();
  });

  it('falls back to replacing .pdf extension when outputPath is missing', () => {
    const file = { ...baseFile, outputPath: undefined };
    render(<EpubListItem file={file} selected={false} onSelect={() => {}} />);
    expect(screen.getByText('Design patterns.epub')).toBeInTheDocument();
  });

  it('shows formatted file size from conversionResult', () => {
    render(<EpubListItem file={baseFile} selected={false} onSelect={() => {}} />);
    expect(screen.getByText('3.1 MB')).toBeInTheDocument();
  });

  it('hides size when conversionResult fileSize is zero', () => {
    const file = { ...baseFile, conversionResult: { fileSize: 0 } };
    render(<EpubListItem file={file} selected={false} onSelect={() => {}} />);
    expect(screen.queryByText('0 B')).not.toBeInTheDocument();
  });

  it('hides size when conversionResult is missing', () => {
    const file = { ...baseFile, conversionResult: undefined };
    render(<EpubListItem file={file} selected={false} onSelect={() => {}} />);
    const option = screen.getByRole('option');
    expect(option.querySelectorAll('span')).toHaveLength(1);
  });

  it('sets aria-selected true when selected', () => {
    render(<EpubListItem file={baseFile} selected={true} onSelect={() => {}} />);
    expect(screen.getByRole('option')).toHaveAttribute('aria-selected', 'true');
  });

  it('sets aria-selected false when not selected', () => {
    render(<EpubListItem file={baseFile} selected={false} onSelect={() => {}} />);
    expect(screen.getByRole('option')).toHaveAttribute('aria-selected', 'false');
  });

  it('calls onSelect when clicked', async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();
    render(<EpubListItem file={baseFile} selected={false} onSelect={onSelect} />);

    await user.click(screen.getByRole('option'));
    expect(onSelect).toHaveBeenCalledOnce();
  });
});
