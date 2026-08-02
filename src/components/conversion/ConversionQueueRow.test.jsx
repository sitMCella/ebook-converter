import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ConversionQueueRow } from './ConversionQueueRow';

describe('ConversionQueueRow', () => {
  const baseFile = {
    path: '/test.pdf',
    name: 'test.pdf',
    status: 'converting',
    conversionProgress: 42,
  };

  it('renders file name', () => {
    render(<ConversionQueueRow file={baseFile} isActive={false} />);
    expect(screen.getByText('test.pdf')).toBeInTheDocument();
  });

  it('shows percentage when active and has progress', () => {
    render(<ConversionQueueRow file={baseFile} isActive />);
    expect(screen.getByText('42%')).toBeInTheDocument();
  });

  it('does not show percentage for queued files', () => {
    render(<ConversionQueueRow file={baseFile} isActive={false} />);
    expect(screen.queryByText('42%')).not.toBeInTheDocument();
    expect(screen.getByText('Queued')).toBeInTheDocument();
  });

  it('renders progress bar when active and converting', () => {
    render(<ConversionQueueRow file={baseFile} isActive />);
    expect(screen.getByRole('progressbar')).toBeInTheDocument();
  });

  it('does not render progress bar for queued files', () => {
    render(<ConversionQueueRow file={baseFile} isActive={false} />);
    expect(screen.queryByRole('progressbar')).not.toBeInTheDocument();
  });

  it('shows error message on click when status is error', async () => {
    const user = userEvent.setup();
    const errorFile = {
      ...baseFile,
      status: 'error',
      errorMessage: 'File too large',
    };
    render(<ConversionQueueRow file={errorFile} isActive />);
    await user.click(screen.getByText('test.pdf'));
    expect(screen.getByText('File too large')).toBeInTheDocument();
  });

  it('toggles error message on repeated clicks', async () => {
    const user = userEvent.setup();
    const errorFile = {
      ...baseFile,
      status: 'error',
      errorMessage: 'File too large',
    };
    render(<ConversionQueueRow file={errorFile} isActive />);
    await user.click(screen.getByText('test.pdf'));
    expect(screen.getByText('File too large')).toBeInTheDocument();
    await user.click(screen.getByText('test.pdf'));
    expect(screen.queryByText('File too large')).not.toBeInTheDocument();
  });

  it('uses 0 for progress bar when conversionProgress is undefined', () => {
    const file = { ...baseFile, conversionProgress: undefined };
    render(<ConversionQueueRow file={file} isActive />);
    const bar = screen.getByRole('progressbar');
    expect(bar).toHaveAttribute('aria-valuenow', '0');
  });
});
