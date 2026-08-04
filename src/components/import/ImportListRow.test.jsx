import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ImportListRow } from './ImportListRow';

describe('ImportListRow', () => {
  const baseFile = {
    path: '/test/document.pdf',
    name: 'document.pdf',
    size: 2048,
    status: 'ready',
    metadata: { fileSize: 2048 },
  };

  function renderRow(fileOverrides = {}, selected = false) {
    const onToggle = vi.fn();
    const file = { ...baseFile, ...fileOverrides };
    const result = render(
      <ImportListRow file={file} selected={selected} onToggleSelect={onToggle} />
    );
    return { ...result, onToggle };
  }

  it('renders file name as plain text', () => {
    renderRow();
    expect(screen.getByText('document.pdf')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'document.pdf' })).not.toBeInTheDocument();
  });

  it('renders file size from metadata', () => {
    renderRow({ metadata: { fileSize: 1024 } });
    expect(screen.getByText('1 KB')).toBeInTheDocument();
  });

  it('does not render status badge for non-error status', () => {
    renderRow();
    expect(screen.queryByText('Ready')).not.toBeInTheDocument();
  });

  it('renders error badge for error status', () => {
    renderRow({ status: 'error', errorMessage: 'Corrupted' });
    expect(screen.getByText('Error')).toBeInTheDocument();
  });

  it('shows error message when error row is clicked', async () => {
    const user = userEvent.setup();
    renderRow({ status: 'error', errorMessage: 'File corrupted' });
    const row = screen.getByText('document.pdf').closest('div[class*="flex items-center"]');
    await user.click(row);
    expect(screen.getByText('File corrupted')).toBeInTheDocument();
  });

  it('toggles error message visibility on repeated clicks', async () => {
    const user = userEvent.setup();
    renderRow({ status: 'error', errorMessage: 'File corrupted' });
    const row = screen.getByText('document.pdf').closest('div[class*="flex items-center"]');
    await user.click(row);
    expect(screen.getByText('File corrupted')).toBeInTheDocument();
    await user.click(row);
    expect(screen.queryByText('File corrupted')).not.toBeInTheDocument();
  });

  it('renders checkbox with selection state', () => {
    renderRow({}, true);
    expect(screen.getByRole('checkbox')).toHaveAttribute('aria-checked', 'true');
  });

  it('does not show file size when size is 0', () => {
    renderRow({ size: 0, metadata: null });
    expect(screen.queryByText('0 B')).not.toBeInTheDocument();
  });

  it('falls back to file.size when metadata is null', () => {
    renderRow({ size: 4096, metadata: null });
    expect(screen.getByText('4 KB')).toBeInTheDocument();
  });
});
