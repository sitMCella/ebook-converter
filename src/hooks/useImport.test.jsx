import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { ImportProvider } from '../contexts/ImportContext';

vi.mock('../lib/tauri', () => ({
  validatePdf: vi.fn(),
  getPdfMetadata: vi.fn(),
  getFileSize: vi.fn(),
  importPdf: vi.fn(),
}));

vi.mock('sonner', () => ({
  toast: { info: vi.fn() },
}));

import { useImport } from './useImport';
import { useImportContext } from '../contexts/ImportContext';
import { validatePdf, getPdfMetadata, getFileSize, importPdf } from '../lib/tauri';
import { toast } from 'sonner';

function wrapper({ children }) {
  return <ImportProvider>{children}</ImportProvider>;
}

function renderUseImport() {
  return renderHook(
    () => ({
      import: useImport(),
      context: useImportContext(),
    }),
    { wrapper }
  );
}

describe('useImport', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    getFileSize.mockResolvedValue(1024);
    validatePdf.mockResolvedValue({ status: 'valid' });
    importPdf.mockResolvedValue({ bookId: 'test-uuid-1234', storedPdfPath: '/stored/test.pdf' });
    getPdfMetadata.mockResolvedValue({
      title: 'Test',
      author: 'Author',
      pageCount: 10,
      pdfVersion: '1.7',
    });
  });

  it('imports valid files and sets metadata', async () => {
    const { result } = renderUseImport();

    await act(async () => {
      await result.current.import.importFiles(['/test.pdf']);
    });

    const file = result.current.context.state.files.get('/test.pdf');
    expect(file).toBeDefined();
    expect(file.status).toBe('ready');
    expect(file.metadata.title).toBe('Test');
    expect(getFileSize).toHaveBeenCalledWith('/test.pdf');
    expect(validatePdf).toHaveBeenCalledWith('/test.pdf');
    expect(getPdfMetadata).toHaveBeenCalledWith('/test.pdf');
  });

  it('shows toast and skips duplicate files', async () => {
    const { result } = renderUseImport();

    await act(async () => {
      await result.current.import.importFiles(['/test.pdf']);
    });

    await act(async () => {
      await result.current.import.importFiles(['/test.pdf']);
    });

    expect(toast.info).toHaveBeenCalledWith('File already imported', { duration: 3000 });
    expect(result.current.context.state.files.size).toBe(1);
  });

  it('sets error status for encrypted PDFs', async () => {
    validatePdf.mockResolvedValue({ status: 'encrypted' });
    const { result } = renderUseImport();

    await act(async () => {
      await result.current.import.importFiles(['/encrypted.pdf']);
    });

    const file = result.current.context.state.files.get('/encrypted.pdf');
    expect(file.status).toBe('error');
    expect(file.errorMessage).toContain('password-protected');
    expect(getPdfMetadata).not.toHaveBeenCalled();
  });

  it('sets error status for corrupted PDFs', async () => {
    validatePdf.mockResolvedValue({ status: 'error', message: 'Corrupted' });
    const { result } = renderUseImport();

    await act(async () => {
      await result.current.import.importFiles(['/corrupted.pdf']);
    });

    const file = result.current.context.state.files.get('/corrupted.pdf');
    expect(file.status).toBe('error');
    expect(file.errorMessage).toBe('Corrupted');
    expect(getPdfMetadata).not.toHaveBeenCalled();
  });

  it('uses default error message when validation error has no message', async () => {
    validatePdf.mockResolvedValue({ status: 'error' });
    const { result } = renderUseImport();

    await act(async () => {
      await result.current.import.importFiles(['/bad.pdf']);
    });

    const file = result.current.context.state.files.get('/bad.pdf');
    expect(file.errorMessage).toContain('could not be read');
  });

  it('handles exceptions during processing', async () => {
    validatePdf.mockRejectedValue(new Error('Network error'));
    const { result } = renderUseImport();

    await act(async () => {
      await result.current.import.importFiles(['/fail.pdf']);
    });

    const file = result.current.context.state.files.get('/fail.pdf');
    expect(file.status).toBe('error');
    expect(file.errorMessage).toContain('Network error');
  });

  it('does nothing when paths is null or empty', async () => {
    const { result } = renderUseImport();

    await act(async () => {
      await result.current.import.importFiles(null);
    });
    expect(result.current.context.state.files.size).toBe(0);

    await act(async () => {
      await result.current.import.importFiles([]);
    });
    expect(result.current.context.state.files.size).toBe(0);
    expect(validatePdf).not.toHaveBeenCalled();
  });

  it('imports multiple files in parallel', async () => {
    const { result } = renderUseImport();

    await act(async () => {
      await result.current.import.importFiles(['/a.pdf', '/b.pdf', '/c.pdf']);
    });

    expect(result.current.context.state.files.size).toBe(3);
    expect(validatePdf).toHaveBeenCalledTimes(3);
  });

  it('extracts file name from path', async () => {
    const { result } = renderUseImport();

    await act(async () => {
      await result.current.import.importFiles(['/home/user/documents/report.pdf']);
    });

    const file = result.current.context.state.files.get('/home/user/documents/report.pdf');
    expect(file.name).toBe('report.pdf');
  });

  it('calls importPdf after validation and stores bookId', async () => {
    const { result } = renderUseImport();

    await act(async () => {
      await result.current.import.importFiles(['/test.pdf']);
    });

    expect(importPdf).toHaveBeenCalledWith('/test.pdf');
    const file = result.current.context.state.files.get('/test.pdf');
    expect(file.bookId).toBe('test-uuid-1234');
    expect(file.storedPdfPath).toBe('/stored/test.pdf');
  });

  it('does not call importPdf when validation fails', async () => {
    validatePdf.mockResolvedValue({ status: 'encrypted' });
    const { result } = renderUseImport();

    await act(async () => {
      await result.current.import.importFiles(['/encrypted.pdf']);
    });

    expect(importPdf).not.toHaveBeenCalled();
  });

  it('sets error when importPdf fails', async () => {
    importPdf.mockRejectedValue(new Error('Disk full'));
    const { result } = renderUseImport();

    await act(async () => {
      await result.current.import.importFiles(['/test.pdf']);
    });

    const file = result.current.context.state.files.get('/test.pdf');
    expect(file.status).toBe('error');
    expect(file.errorMessage).toContain('Disk full');
  });

  it('sets isImporting to false after import completes', async () => {
    const { result } = renderUseImport();

    await act(async () => {
      await result.current.import.importFiles(['/done.pdf']);
    });

    expect(result.current.import.isImporting).toBe(false);
  });
});
