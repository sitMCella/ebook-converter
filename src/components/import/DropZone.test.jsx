import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { DropZone } from './DropZone';

vi.mock('../../lib/tauri', () => ({
  openPdfFiles: vi.fn(),
}));

import { openPdfFiles } from '../../lib/tauri';

describe('DropZone', () => {
  it('renders drop zone with instruction text', () => {
    render(<DropZone onFilesSelected={() => {}} />);
    expect(screen.getByText('Drop PDF files here')).toBeInTheDocument();
    expect(
      screen.getByText(/click "Browse files" to select/)
    ).toBeInTheDocument();
  });

  it('has accessible role and label', () => {
    render(<DropZone onFilesSelected={() => {}} />);
    const zone = screen.getByRole('button', { name: 'Drop zone for PDF files' });
    expect(zone).toBeInTheDocument();
    expect(zone).toHaveAttribute('tabIndex', '0');
  });

  it('applies drag-over styles on dragenter', () => {
    render(<DropZone onFilesSelected={() => {}} />);
    const zone = screen.getByRole('button', { name: 'Drop zone for PDF files' });
    fireEvent.dragEnter(zone, { dataTransfer: { types: ['Files'] } });
    expect(zone.className).toContain('scale-[1.01]');
  });

  it('removes drag-over styles on dragleave', () => {
    render(<DropZone onFilesSelected={() => {}} />);
    const zone = screen.getByRole('button', { name: 'Drop zone for PDF files' });
    fireEvent.dragEnter(zone, { dataTransfer: { types: ['Files'] } });
    fireEvent.dragLeave(zone, { relatedTarget: document.body });
    expect(zone.className).not.toContain('scale-[1.01]');
  });

  it('removes drag-over styles on drop', () => {
    render(<DropZone onFilesSelected={() => {}} />);
    const zone = screen.getByRole('button', { name: 'Drop zone for PDF files' });
    fireEvent.dragEnter(zone);
    fireEvent.drop(zone);
    expect(zone.className).not.toContain('scale-[1.01]');
  });

  it('triggers file dialog on Enter key', async () => {
    const onFilesSelected = vi.fn();
    openPdfFiles.mockResolvedValue(['/test.pdf']);
    render(<DropZone onFilesSelected={onFilesSelected} />);
    const zone = screen.getByRole('button', { name: 'Drop zone for PDF files' });
    fireEvent.keyDown(zone, { key: 'Enter' });
    await vi.waitFor(() => {
      expect(openPdfFiles).toHaveBeenCalled();
    });
  });

  it('triggers file dialog on Space key', async () => {
    const onFilesSelected = vi.fn();
    openPdfFiles.mockResolvedValue(['/test.pdf']);
    render(<DropZone onFilesSelected={onFilesSelected} />);
    const zone = screen.getByRole('button', { name: 'Drop zone for PDF files' });
    fireEvent.keyDown(zone, { key: ' ' });
    await vi.waitFor(() => {
      expect(openPdfFiles).toHaveBeenCalled();
    });
  });

  it('calls onFilesSelected when file dialog returns paths', async () => {
    const onFilesSelected = vi.fn();
    openPdfFiles.mockResolvedValue(['/test.pdf']);
    render(<DropZone onFilesSelected={onFilesSelected} />);
    const zone = screen.getByRole('button', { name: 'Drop zone for PDF files' });
    fireEvent.keyDown(zone, { key: 'Enter' });
    await vi.waitFor(() => {
      expect(onFilesSelected).toHaveBeenCalledWith(['/test.pdf']);
    });
  });

  it('does not call onFilesSelected when dialog returns null', async () => {
    const onFilesSelected = vi.fn();
    openPdfFiles.mockResolvedValue(null);
    render(<DropZone onFilesSelected={onFilesSelected} />);
    const zone = screen.getByRole('button', { name: 'Drop zone for PDF files' });
    fireEvent.keyDown(zone, { key: 'Enter' });
    await vi.waitFor(() => {
      expect(openPdfFiles).toHaveBeenCalled();
    });
    expect(onFilesSelected).not.toHaveBeenCalled();
  });
});
