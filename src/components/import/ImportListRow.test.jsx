import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import { ImportListRow } from './ImportListRow';

const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return { ...actual, useNavigate: () => mockNavigate };
});

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
      <MemoryRouter>
        <ImportListRow file={file} selected={selected} onToggleSelect={onToggle} />
      </MemoryRouter>
    );
    return { ...result, onToggle };
  }

  it('renders file name', () => {
    renderRow();
    expect(screen.getByText('document.pdf')).toBeInTheDocument();
  });

  it('renders file size from metadata', () => {
    renderRow({ metadata: { fileSize: 1024 } });
    expect(screen.getByText('1 KB')).toBeInTheDocument();
  });

  it('renders status badge', () => {
    renderRow();
    expect(screen.getByText('Ready')).toBeInTheDocument();
  });

  it('renders error badge for error status', () => {
    renderRow({ status: 'error', errorMessage: 'Corrupted' });
    expect(screen.getByText('Error')).toBeInTheDocument();
  });

  it('shows error message when error row is clicked', async () => {
    const user = userEvent.setup();
    renderRow({ status: 'error', errorMessage: 'File corrupted' });
    const row = screen.getByText('Error').closest('div[class*="flex items-center"]');
    await user.click(row);
    expect(screen.getByText('File corrupted')).toBeInTheDocument();
  });

  it('toggles error message visibility on repeated clicks', async () => {
    const user = userEvent.setup();
    renderRow({ status: 'error', errorMessage: 'File corrupted' });
    const row = screen.getByText('Error').closest('div[class*="flex items-center"]');
    await user.click(row);
    expect(screen.getByText('File corrupted')).toBeInTheDocument();
    await user.click(row);
    expect(screen.queryByText('File corrupted')).not.toBeInTheDocument();
  });

  it('navigates to /library when file name is clicked', async () => {
    const user = userEvent.setup();
    renderRow();
    await user.click(screen.getByText('document.pdf'));
    expect(mockNavigate).toHaveBeenCalledWith('/library');
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
