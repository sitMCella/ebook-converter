import { describe, it, expect } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { ImportProvider, useImportContext } from './ImportContext';

function renderImportContext() {
  return renderHook(() => useImportContext(), {
    wrapper: ImportProvider,
  });
}

describe('ImportContext', () => {
  describe('initial state', () => {
    it('starts with empty files and no selections', () => {
      const { result } = renderImportContext();
      expect(result.current.state.files.size).toBe(0);
      expect(result.current.state.selectedPaths.size).toBe(0);
    });
  });

  describe('ADD_FILES', () => {
    it('adds files to the map', () => {
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

    it('does not overwrite existing files with the same path', () => {
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

  describe('REMOVE_FILES', () => {
    it('removes files by path', () => {
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

    it('also removes selections for removed files', () => {
      const { result } = renderImportContext();
      act(() => {
        result.current.dispatch({
          type: 'ADD_FILES',
          files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }],
        });
      });
      act(() => {
        result.current.dispatch({ type: 'TOGGLE_SELECTION', path: '/a.pdf' });
      });
      expect(result.current.state.selectedPaths.has('/a.pdf')).toBe(true);
      act(() => {
        result.current.dispatch({ type: 'REMOVE_FILES', paths: ['/a.pdf'] });
      });
      expect(result.current.state.selectedPaths.has('/a.pdf')).toBe(false);
    });
  });

  describe('UPDATE_STATUS', () => {
    it('updates file status and error message', () => {
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

  describe('SET_METADATA', () => {
    it('sets metadata on a file', () => {
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
          type: 'ADD_FILES',
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
          type: 'ADD_FILES',
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
    it('selects all files', () => {
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
          type: 'ADD_FILES',
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
    it('sets bookId and storedPdfPath on a file', () => {
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

  describe('useImportContext outside provider', () => {
    it('throws when used outside ImportProvider', () => {
      expect(() => {
        renderHook(() => useImportContext());
      }).toThrow('useImportContext must be used within an ImportProvider');
    });
  });
});
