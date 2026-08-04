import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { ImportProvider } from '../contexts/ImportContext';

vi.mock('../lib/tauri', () => ({
  validatePdf: vi.fn(),
  getPdfMetadata: vi.fn(),
  getFileSize: vi.fn(),
  importPdf: vi.fn(),
  saveBookMetadata: vi.fn(),
  listBooks: vi.fn(),
}));

vi.mock('sonner', () => ({
  toast: { info: vi.fn() },
}));

import { useImport } from './useImport';
import { useImportContext } from '../contexts/ImportContext';
import { validatePdf, getPdfMetadata, getFileSize, importPdf, saveBookMetadata, listBooks } from '../lib/tauri';
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
    saveBookMetadata.mockResolvedValue(undefined);
    listBooks.mockResolvedValue([]);
  });

  describe('stageFiles', () => {
    it('stages valid files with metadata', async () => {
      const { result } = renderUseImport();

      await act(async () => {
        await result.current.import.stageFiles(['/test.pdf']);
      });

      const file = result.current.context.state.stagedFiles.get('/test.pdf');
      expect(file).toBeDefined();
      expect(file.status).toBe('ready');
      expect(file.metadata.title).toBe('Test');
      expect(getFileSize).toHaveBeenCalledWith('/test.pdf');
      expect(validatePdf).toHaveBeenCalledWith('/test.pdf');
      expect(getPdfMetadata).toHaveBeenCalledWith('/test.pdf');
      expect(importPdf).not.toHaveBeenCalled();
      expect(result.current.context.state.files.size).toBe(0);
    });

    it('shows toast and skips duplicate staged files', async () => {
      const { result } = renderUseImport();

      await act(async () => {
        await result.current.import.stageFiles(['/test.pdf']);
      });

      await act(async () => {
        await result.current.import.stageFiles(['/test.pdf']);
      });

      expect(toast.info).toHaveBeenCalledWith('File already imported', { duration: 3000 });
      expect(result.current.context.state.stagedFiles.size).toBe(1);
    });

    it('shows toast and skips files already in library', async () => {
      const { result } = renderUseImport();

      act(() => {
        result.current.context.dispatch({
          type: 'ADD_FILES',
          files: [{ path: '/test.pdf', name: 'test.pdf', status: 'ready' }],
        });
      });

      await act(async () => {
        await result.current.import.stageFiles(['/test.pdf']);
      });

      expect(toast.info).toHaveBeenCalledWith('File already imported', { duration: 3000 });
      expect(result.current.context.state.stagedFiles.size).toBe(0);
    });

    it('sets error status for encrypted PDFs', async () => {
      validatePdf.mockResolvedValue({ status: 'encrypted' });
      const { result } = renderUseImport();

      await act(async () => {
        await result.current.import.stageFiles(['/encrypted.pdf']);
      });

      const file = result.current.context.state.stagedFiles.get('/encrypted.pdf');
      expect(file.status).toBe('error');
      expect(file.errorMessage).toContain('password-protected');
      expect(getPdfMetadata).not.toHaveBeenCalled();
    });

    it('sets error status for corrupted PDFs', async () => {
      validatePdf.mockResolvedValue({ status: 'error', message: 'Corrupted' });
      const { result } = renderUseImport();

      await act(async () => {
        await result.current.import.stageFiles(['/corrupted.pdf']);
      });

      const file = result.current.context.state.stagedFiles.get('/corrupted.pdf');
      expect(file.status).toBe('error');
      expect(file.errorMessage).toBe('Corrupted');
      expect(getPdfMetadata).not.toHaveBeenCalled();
    });

    it('uses default error message when validation error has no message', async () => {
      validatePdf.mockResolvedValue({ status: 'error' });
      const { result } = renderUseImport();

      await act(async () => {
        await result.current.import.stageFiles(['/bad.pdf']);
      });

      const file = result.current.context.state.stagedFiles.get('/bad.pdf');
      expect(file.errorMessage).toContain('could not be read');
    });

    it('handles exceptions during processing', async () => {
      validatePdf.mockRejectedValue(new Error('Network error'));
      const { result } = renderUseImport();

      await act(async () => {
        await result.current.import.stageFiles(['/fail.pdf']);
      });

      const file = result.current.context.state.stagedFiles.get('/fail.pdf');
      expect(file.status).toBe('error');
      expect(file.errorMessage).toContain('Network error');
    });

    it('does nothing when paths is null or empty', async () => {
      const { result } = renderUseImport();

      await act(async () => {
        await result.current.import.stageFiles(null);
      });
      expect(result.current.context.state.stagedFiles.size).toBe(0);

      await act(async () => {
        await result.current.import.stageFiles([]);
      });
      expect(result.current.context.state.stagedFiles.size).toBe(0);
      expect(validatePdf).not.toHaveBeenCalled();
    });

    it('stages multiple files in parallel', async () => {
      const { result } = renderUseImport();

      await act(async () => {
        await result.current.import.stageFiles(['/a.pdf', '/b.pdf', '/c.pdf']);
      });

      expect(result.current.context.state.stagedFiles.size).toBe(3);
      expect(validatePdf).toHaveBeenCalledTimes(3);
    });

    it('extracts file name from path', async () => {
      const { result } = renderUseImport();

      await act(async () => {
        await result.current.import.stageFiles(['/home/user/documents/report.pdf']);
      });

      const file = result.current.context.state.stagedFiles.get('/home/user/documents/report.pdf');
      expect(file.name).toBe('report.pdf');
    });

    it('sets isProcessing to false after staging completes', async () => {
      const { result } = renderUseImport();

      await act(async () => {
        await result.current.import.stageFiles(['/done.pdf']);
      });

      expect(result.current.import.isProcessing).toBe(false);
    });
  });

  describe('importStagedFiles', () => {
    it('moves staged files to library with storage info', async () => {
      const { result } = renderUseImport();

      await act(async () => {
        await result.current.import.stageFiles(['/test.pdf']);
      });

      await act(async () => {
        await result.current.import.importStagedFiles(['/test.pdf']);
      });

      expect(importPdf).toHaveBeenCalledWith('/test.pdf');
      expect(result.current.context.state.stagedFiles.size).toBe(0);
      expect(result.current.context.state.files.size).toBe(1);
      const file = result.current.context.state.files.get('/test.pdf');
      expect(file.bookId).toBe('test-uuid-1234');
      expect(file.storedPdfPath).toBe('/stored/test.pdf');
    });

    it('saves book metadata after import', async () => {
      const { result } = renderUseImport();

      await act(async () => {
        await result.current.import.stageFiles(['/test.pdf']);
      });

      await act(async () => {
        await result.current.import.importStagedFiles(['/test.pdf']);
      });

      expect(saveBookMetadata).toHaveBeenCalledWith({
        bookId: 'test-uuid-1234',
        storedPdfPath: '/stored/test.pdf',
        originalPath: '/test.pdf',
        originalName: 'test.pdf',
        fileSize: 0,
        title: 'Test',
        author: 'Author',
        pageCount: 10,
        pdfVersion: '1.7',
        createdDate: null,
        modifiedDate: null,
        producer: null,
        status: 'ready',
      });
    });

    it('does not save metadata when bookId is null', async () => {
      importPdf.mockResolvedValue({ bookId: null, storedPdfPath: '/test.pdf' });
      const { result } = renderUseImport();

      await act(async () => {
        await result.current.import.stageFiles(['/test.pdf']);
      });

      await act(async () => {
        await result.current.import.importStagedFiles(['/test.pdf']);
      });

      expect(saveBookMetadata).not.toHaveBeenCalled();
    });

    it('skips files that are not ready', async () => {
      validatePdf.mockResolvedValue({ status: 'encrypted' });
      const { result } = renderUseImport();

      await act(async () => {
        await result.current.import.stageFiles(['/encrypted.pdf']);
      });

      await act(async () => {
        await result.current.import.importStagedFiles(['/encrypted.pdf']);
      });

      expect(importPdf).not.toHaveBeenCalled();
      expect(result.current.context.state.stagedFiles.size).toBe(1);
    });

    it('sets error on staged file when importPdf fails', async () => {
      importPdf.mockRejectedValue(new Error('Disk full'));
      const { result } = renderUseImport();

      await act(async () => {
        await result.current.import.stageFiles(['/test.pdf']);
      });

      await act(async () => {
        await result.current.import.importStagedFiles(['/test.pdf']);
      });

      const file = result.current.context.state.stagedFiles.get('/test.pdf');
      expect(file.status).toBe('error');
      expect(file.errorMessage).toContain('Disk full');
    });

    it('does nothing when paths is null or empty', async () => {
      const { result } = renderUseImport();

      await act(async () => {
        await result.current.import.importStagedFiles(null);
      });
      expect(importPdf).not.toHaveBeenCalled();

      await act(async () => {
        await result.current.import.importStagedFiles([]);
      });
      expect(importPdf).not.toHaveBeenCalled();
    });
  });
});
