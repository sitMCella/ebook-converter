import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';

vi.mock('../lib/tauri', () => ({
  listBooks: vi.fn(),
}));

import { ImportProvider, useImportContext } from './ImportContext';
import { listBooks } from '../lib/tauri';

function renderImportContext() {
  return renderHook(() => useImportContext(), {
    wrapper: ImportProvider,
  });
}

describe('ImportContext', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    listBooks.mockResolvedValue([]);
  });

  describe('initial state', () => {
    it('starts with empty files, stagedFiles, and no selections', () => {
      const { result } = renderImportContext();
      expect(result.current.state.files.size).toBe(0);
      expect(result.current.state.stagedFiles.size).toBe(0);
      expect(result.current.state.selectedPaths.size).toBe(0);
    });
  });

  describe('STAGE_FILES', () => {
    it('adds files to the staged map', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'STAGE_FILES',
          files: [
            { path: '/a.pdf', name: 'a.pdf', status: 'ready' },
            { path: '/b.pdf', name: 'b.pdf', status: 'ready' },
          ],
        });
      });
      expect(result.current.state.stagedFiles.size).toBe(2);
      expect(result.current.state.stagedFiles.get('/a.pdf').name).toBe('a.pdf');
      expect(result.current.state.files.size).toBe(0);
    });

    it('does not overwrite existing staged files with the same path', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'STAGE_FILES',
          files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }],
        });
      });
      act(() => {
        result.current.dispatch({
          type: 'STAGE_FILES',
          files: [{ path: '/a.pdf', name: 'a-duplicate.pdf', status: 'error' }],
        });
      });
      expect(result.current.state.stagedFiles.size).toBe(1);
      expect(result.current.state.stagedFiles.get('/a.pdf').name).toBe('a.pdf');
    });
  });

  describe('UNSTAGE_FILES', () => {
    it('removes files from staged map', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'STAGE_FILES',
          files: [
            { path: '/a.pdf', name: 'a.pdf', status: 'ready' },
            { path: '/b.pdf', name: 'b.pdf', status: 'ready' },
          ],
        });
      });
      act(() => {
        result.current.dispatch({ type: 'UNSTAGE_FILES', paths: ['/a.pdf'] });
      });
      expect(result.current.state.stagedFiles.size).toBe(1);
      expect(result.current.state.stagedFiles.has('/a.pdf')).toBe(false);
    });

    it('also removes selections for unstaged files', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'STAGE_FILES',
          files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }],
        });
      });
      act(() => {
        result.current.dispatch({ type: 'TOGGLE_SELECTION', path: '/a.pdf' });
      });
      expect(result.current.state.selectedPaths.has('/a.pdf')).toBe(true);
      act(() => {
        result.current.dispatch({ type: 'UNSTAGE_FILES', paths: ['/a.pdf'] });
      });
      expect(result.current.state.selectedPaths.has('/a.pdf')).toBe(false);
    });
  });

  describe('UPDATE_STAGED_STATUS', () => {
    it('updates staged file status and error message', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'STAGE_FILES',
          files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }],
        });
      });
      act(() => {
        result.current.dispatch({
          type: 'UPDATE_STAGED_STATUS',
          path: '/a.pdf',
          status: 'error',
          errorMessage: 'Corrupted file',
        });
      });
      const file = result.current.state.stagedFiles.get('/a.pdf');
      expect(file.status).toBe('error');
      expect(file.errorMessage).toBe('Corrupted file');
    });

    it('ignores update for non-existent staged path', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'UPDATE_STAGED_STATUS',
          path: '/nonexistent.pdf',
          status: 'error',
        });
      });
      expect(result.current.state.stagedFiles.size).toBe(0);
    });
  });

  describe('SET_STAGED_METADATA', () => {
    it('sets metadata on a staged file', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'STAGE_FILES',
          files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready', metadata: null }],
        });
      });
      const metadata = { title: 'Test', pageCount: 5 };
      act(() => {
        result.current.dispatch({ type: 'SET_STAGED_METADATA', path: '/a.pdf', metadata });
      });
      expect(result.current.state.stagedFiles.get('/a.pdf').metadata).toEqual(metadata);
    });

    it('ignores metadata for non-existent staged path', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'SET_STAGED_METADATA',
          path: '/nonexistent.pdf',
          metadata: { title: 'Test' },
        });
      });
      expect(result.current.state.stagedFiles.size).toBe(0);
    });
  });

  describe('IMPORT_TO_LIBRARY', () => {
    it('moves a staged file to the library with storage info', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'STAGE_FILES',
          files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready', metadata: { title: 'Test' } }],
        });
      });
      act(() => {
        result.current.dispatch({
          type: 'IMPORT_TO_LIBRARY',
          path: '/a.pdf',
          bookId: 'uuid-123',
          storedPdfPath: '/books/uuid-123/a.pdf',
        });
      });
      expect(result.current.state.stagedFiles.size).toBe(0);
      expect(result.current.state.files.size).toBe(1);
      const file = result.current.state.files.get('/a.pdf');
      expect(file.name).toBe('a.pdf');
      expect(file.bookId).toBe('uuid-123');
      expect(file.storedPdfPath).toBe('/books/uuid-123/a.pdf');
      expect(file.metadata.title).toBe('Test');
    });

    it('clears selection for imported file', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'STAGE_FILES',
          files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }],
        });
      });
      act(() => {
        result.current.dispatch({ type: 'TOGGLE_SELECTION', path: '/a.pdf' });
      });
      expect(result.current.state.selectedPaths.has('/a.pdf')).toBe(true);
      act(() => {
        result.current.dispatch({
          type: 'IMPORT_TO_LIBRARY',
          path: '/a.pdf',
          bookId: 'uuid-123',
          storedPdfPath: '/books/uuid-123/a.pdf',
        });
      });
      expect(result.current.state.selectedPaths.has('/a.pdf')).toBe(false);
    });

    it('ignores import for non-existent staged path', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'IMPORT_TO_LIBRARY',
          path: '/nonexistent.pdf',
          bookId: 'uuid-123',
          storedPdfPath: '/books/uuid-123/nonexistent.pdf',
        });
      });
      expect(result.current.state.files.size).toBe(0);
    });
  });

  describe('ADD_FILES (library)', () => {
    it('adds files to the library map', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'ADD_FILES',
          files: [
            { path: '/a.pdf', name: 'a.pdf', status: 'ready' },
            { path: '/b.pdf', name: 'b.pdf', status: 'ready' },
          ],
        });
      });
      expect(result.current.state.files.size).toBe(2);
      expect(result.current.state.files.get('/a.pdf').name).toBe('a.pdf');
    });

    it('does not overwrite existing library files with the same path', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'ADD_FILES',
          files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }],
        });
      });
      act(() => {
        result.current.dispatch({
          type: 'ADD_FILES',
          files: [{ path: '/a.pdf', name: 'a-duplicate.pdf', status: 'error' }],
        });
      });
      expect(result.current.state.files.size).toBe(1);
      expect(result.current.state.files.get('/a.pdf').name).toBe('a.pdf');
    });
  });

  describe('REMOVE_FILES (library)', () => {
    it('removes files by path from library', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'ADD_FILES',
          files: [
            { path: '/a.pdf', name: 'a.pdf', status: 'ready' },
            { path: '/b.pdf', name: 'b.pdf', status: 'ready' },
          ],
        });
      });
      act(() => {
        result.current.dispatch({ type: 'REMOVE_FILES', paths: ['/a.pdf'] });
      });
      expect(result.current.state.files.size).toBe(1);
      expect(result.current.state.files.has('/a.pdf')).toBe(false);
    });
  });

  describe('UPDATE_STATUS (library)', () => {
    it('updates library file status and error message', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'ADD_FILES',
          files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }],
        });
      });
      act(() => {
        result.current.dispatch({
          type: 'UPDATE_STATUS',
          path: '/a.pdf',
          status: 'error',
          errorMessage: 'Corrupted file',
        });
      });
      const file = result.current.state.files.get('/a.pdf');
      expect(file.status).toBe('error');
      expect(file.errorMessage).toBe('Corrupted file');
    });

    it('ignores update for non-existent path', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'UPDATE_STATUS',
          path: '/nonexistent.pdf',
          status: 'error',
        });
      });
      expect(result.current.state.files.size).toBe(0);
    });
  });

  describe('SET_METADATA (library)', () => {
    it('sets metadata on a library file', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'ADD_FILES',
          files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready', metadata: null }],
        });
      });
      const metadata = { title: 'Test', pageCount: 5 };
      act(() => {
        result.current.dispatch({ type: 'SET_METADATA', path: '/a.pdf', metadata });
      });
      expect(result.current.state.files.get('/a.pdf').metadata).toEqual(metadata);
    });

    it('ignores metadata for non-existent path', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'SET_METADATA',
          path: '/nonexistent.pdf',
          metadata: { title: 'Test' },
        });
      });
      expect(result.current.state.files.size).toBe(0);
    });
  });

  describe('TOGGLE_SELECTION', () => {
    it('toggles selection on', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'STAGE_FILES',
          files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }],
        });
      });
      act(() => {
        result.current.dispatch({ type: 'TOGGLE_SELECTION', path: '/a.pdf' });
      });
      expect(result.current.state.selectedPaths.has('/a.pdf')).toBe(true);
    });

    it('toggles selection off', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'STAGE_FILES',
          files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }],
        });
      });
      act(() => {
        result.current.dispatch({ type: 'TOGGLE_SELECTION', path: '/a.pdf' });
      });
      act(() => {
        result.current.dispatch({ type: 'TOGGLE_SELECTION', path: '/a.pdf' });
      });
      expect(result.current.state.selectedPaths.has('/a.pdf')).toBe(false);
    });
  });

  describe('SELECT_ALL', () => {
    it('selects all staged files', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'STAGE_FILES',
          files: [
            { path: '/a.pdf', name: 'a.pdf', status: 'ready' },
            { path: '/b.pdf', name: 'b.pdf', status: 'ready' },
          ],
        });
      });
      act(() => {
        result.current.dispatch({ type: 'SELECT_ALL' });
      });
      expect(result.current.state.selectedPaths.size).toBe(2);
    });
  });

  describe('DESELECT_ALL', () => {
    it('deselects all files', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'STAGE_FILES',
          files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }],
        });
      });
      act(() => {
        result.current.dispatch({ type: 'SELECT_ALL' });
      });
      act(() => {
        result.current.dispatch({ type: 'DESELECT_ALL' });
      });
      expect(result.current.state.selectedPaths.size).toBe(0);
    });
  });

  describe('SET_CONVERSION_PROGRESS', () => {
    it('updates progress on a converting file', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'ADD_FILES',
          files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready', metadata: null }],
        });
      });
      act(() => {
        result.current.dispatch({ type: 'UPDATE_STATUS', path: '/a.pdf', status: 'converting' });
      });
      act(() => {
        result.current.dispatch({
          type: 'SET_CONVERSION_PROGRESS',
          path: '/a.pdf',
          percent: 42,
          stage: 'extracting_text',
        });
      });
      const file = result.current.state.files.get('/a.pdf');
      expect(file.conversionProgress).toBe(42);
      expect(file.conversionStage).toBe('extracting_text');
    });

    it('ignores progress when file is not converting', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'ADD_FILES',
          files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready', metadata: null }],
        });
      });
      act(() => {
        result.current.dispatch({
          type: 'SET_CONVERSION_PROGRESS',
          path: '/a.pdf',
          percent: 42,
          stage: 'extracting_text',
        });
      });
      const file = result.current.state.files.get('/a.pdf');
      expect(file.conversionProgress).toBeUndefined();
    });
  });

  describe('SET_CONVERSION_RESULT', () => {
    it('sets status to converted with output info', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'ADD_FILES',
          files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready', metadata: null }],
        });
      });
      act(() => {
        result.current.dispatch({ type: 'UPDATE_STATUS', path: '/a.pdf', status: 'converting' });
      });
      act(() => {
        result.current.dispatch({
          type: 'SET_CONVERSION_RESULT',
          path: '/a.pdf',
          outputPath: '/output/a.epub',
          result: { outputPath: '/output/a.epub', chapters: 5, images: 2, fileSize: 50000 },
        });
      });
      const file = result.current.state.files.get('/a.pdf');
      expect(file.status).toBe('converted');
      expect(file.outputPath).toBe('/output/a.epub');
      expect(file.conversionResult.chapters).toBe(5);
    });
  });

  describe('SET_STORAGE_INFO', () => {
    it('sets bookId and storedPdfPath on a library file', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'ADD_FILES',
          files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }],
        });
      });
      act(() => {
        result.current.dispatch({
          type: 'SET_STORAGE_INFO',
          path: '/a.pdf',
          bookId: 'uuid-1234',
          storedPdfPath: '/app/books/uuid-1234/source.pdf',
        });
      });
      const file = result.current.state.files.get('/a.pdf');
      expect(file.bookId).toBe('uuid-1234');
      expect(file.storedPdfPath).toBe('/app/books/uuid-1234/source.pdf');
    });

    it('does not modify other file properties', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'ADD_FILES',
          files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready', metadata: { title: 'Test' } }],
        });
      });
      act(() => {
        result.current.dispatch({
          type: 'SET_STORAGE_INFO',
          path: '/a.pdf',
          bookId: 'uuid-5678',
          storedPdfPath: '/app/books/uuid-5678/source.pdf',
        });
      });
      const file = result.current.state.files.get('/a.pdf');
      expect(file.name).toBe('a.pdf');
      expect(file.status).toBe('ready');
      expect(file.metadata.title).toBe('Test');
    });

    it('ignores SET_STORAGE_INFO for non-existent file', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'SET_STORAGE_INFO',
          path: '/nonexistent.pdf',
          bookId: 'uuid-1234',
          storedPdfPath: '/app/books/uuid-1234/source.pdf',
        });
      });
      expect(result.current.state.files.size).toBe(0);
    });
  });

  describe('UPDATE_STATUS clears conversion fields', () => {
    it('clears conversion fields when set back to ready', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'ADD_FILES',
          files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready', metadata: null }],
        });
      });
      act(() => {
        result.current.dispatch({ type: 'UPDATE_STATUS', path: '/a.pdf', status: 'converting' });
      });
      act(() => {
        result.current.dispatch({
          type: 'SET_CONVERSION_PROGRESS',
          path: '/a.pdf',
          percent: 50,
          stage: 'extracting_text',
        });
      });
      act(() => {
        result.current.dispatch({ type: 'UPDATE_STATUS', path: '/a.pdf', status: 'ready' });
      });
      const file = result.current.state.files.get('/a.pdf');
      expect(file.conversionProgress).toBeUndefined();
      expect(file.conversionStage).toBeUndefined();
    });
  });

  describe('unknown action', () => {
    it('returns current state for unknown action type', () => {
      const { result } = renderImportContext();
      const stateBefore = result.current.state;
      act(() => {
        result.current.dispatch({ type: 'UNKNOWN_ACTION' });
      });
      expect(result.current.state.files.size).toBe(stateBefore.files.size);
    });
  });

  describe('LOAD_LIBRARY', () => {
    it('loads books into the library files map', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'LOAD_LIBRARY',
          books: [
            {
              bookId: 'uuid-1',
              storedPdfPath: '/books/uuid-1/source.pdf',
              originalPath: '/docs/report.pdf',
              originalName: 'report.pdf',
              fileSize: 2048,
              title: 'Report',
              author: 'Author',
              pageCount: 15,
              pdfVersion: '1.7',
              createdDate: null,
              modifiedDate: null,
              producer: null,
              status: 'ready',
            },
          ],
        });
      });
      expect(result.current.state.files.size).toBe(1);
      const file = result.current.state.files.get('/docs/report.pdf');
      expect(file.name).toBe('report.pdf');
      expect(file.bookId).toBe('uuid-1');
      expect(file.storedPdfPath).toBe('/books/uuid-1/source.pdf');
      expect(file.metadata.title).toBe('Report');
      expect(file.metadata.pageCount).toBe(15);
      expect(result.current.state.stagedFiles.size).toBe(0);
    });

    it('does not overwrite existing library files', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'ADD_FILES',
          files: [{ path: '/docs/report.pdf', name: 'report.pdf', status: 'converting' }],
        });
      });
      act(() => {
        result.current.dispatch({
          type: 'LOAD_LIBRARY',
          books: [
            {
              bookId: 'uuid-1',
              storedPdfPath: '/books/uuid-1/source.pdf',
              originalPath: '/docs/report.pdf',
              originalName: 'report.pdf',
              fileSize: 2048,
              title: 'Report',
              author: null,
              pageCount: 10,
              pdfVersion: null,
              createdDate: null,
              modifiedDate: null,
              producer: null,
              status: 'ready',
            },
          ],
        });
      });
      const file = result.current.state.files.get('/docs/report.pdf');
      expect(file.status).toBe('converting');
      expect(file.bookId).toBeUndefined();
    });

    it('loads multiple books', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'LOAD_LIBRARY',
          books: [
            {
              bookId: 'uuid-1',
              storedPdfPath: '/books/uuid-1/source.pdf',
              originalPath: '/a.pdf',
              originalName: 'a.pdf',
              fileSize: 100,
              title: null,
              author: null,
              pageCount: 1,
              pdfVersion: null,
              createdDate: null,
              modifiedDate: null,
              producer: null,
              status: 'ready',
            },
            {
              bookId: 'uuid-2',
              storedPdfPath: '/books/uuid-2/source.pdf',
              originalPath: '/b.pdf',
              originalName: 'b.pdf',
              fileSize: 200,
              title: null,
              author: null,
              pageCount: 2,
              pdfVersion: null,
              createdDate: null,
              modifiedDate: null,
              producer: null,
              status: 'ready',
            },
          ],
        });
      });
      expect(result.current.state.files.size).toBe(2);
    });
  });

  describe('startup loading', () => {
    it('loads persisted books on mount', async () => {
      listBooks.mockResolvedValue([
        {
          bookId: 'uuid-startup',
          storedPdfPath: '/books/uuid-startup/source.pdf',
          originalPath: '/startup.pdf',
          originalName: 'startup.pdf',
          fileSize: 512,
          title: 'Startup Book',
          author: null,
          pageCount: 5,
          pdfVersion: '1.4',
          createdDate: null,
          modifiedDate: null,
          producer: null,
          status: 'ready',
        },
      ]);
      const { result } = renderImportContext();
      await waitFor(() => {
        expect(result.current.state.files.size).toBe(1);
      });
      const file = result.current.state.files.get('/startup.pdf');
      expect(file.bookId).toBe('uuid-startup');
      expect(file.metadata.title).toBe('Startup Book');
    });

    it('handles listBooks failure gracefully', async () => {
      listBooks.mockRejectedValue(new Error('fail'));
      const { result } = renderImportContext();
      await waitFor(() => {
        expect(listBooks).toHaveBeenCalled();
      });
      expect(result.current.state.files.size).toBe(0);
    });

    it('does not load when listBooks returns empty', async () => {
      listBooks.mockResolvedValue([]);
      const { result } = renderImportContext();
      await waitFor(() => {
        expect(listBooks).toHaveBeenCalled();
      });
      expect(result.current.state.files.size).toBe(0);
    });
  });

  describe('useImportContext outside provider', () => {
    it('throws when used outside ImportProvider', () => {
      expect(() => {
        renderHook(() => useImportContext());
      }).toThrow('useImportContext must be used within an ImportProvider');
    });
  });
});
