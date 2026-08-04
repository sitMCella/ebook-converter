import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { ImportProvider, useImportContext } from '../contexts/ImportContext';
import { ConversionProvider, useConversionContext } from '../contexts/ConversionContext';
import { SettingsProvider } from '../contexts/SettingsContext';

vi.mock('../lib/tauri', () => ({
  convertPdfToEpub: vi.fn(),
  cancelConversion: vi.fn(),
  onConversionProgress: vi.fn(),
  listBooks: vi.fn().mockResolvedValue([]),
}));

vi.mock('../lib/settings', async () => {
  const actual = await vi.importActual('../lib/settings');
  return {
    ...actual,
    loadSettings: vi.fn().mockResolvedValue({ ...actual.DEFAULT_SETTINGS }),
    saveSettings: vi.fn().mockResolvedValue(undefined),
    getEffectiveSettings: vi.fn((base, overrides) => actual.getEffectiveSettings(base, overrides)),
    settingsToConversionOptions: vi.fn(),
  };
});

import { useConversion } from './useConversion';
import { convertPdfToEpub, cancelConversion, onConversionProgress } from '../lib/tauri';
import { getEffectiveSettings, settingsToConversionOptions } from '../lib/settings';

function wrapper({ children }) {
  return (
    <SettingsProvider>
      <ImportProvider>
        <ConversionProvider>{children}</ConversionProvider>
      </ImportProvider>
    </SettingsProvider>
  );
}

async function renderUseConversion() {
  const result = renderHook(
    () => ({
      conversion: useConversion(),
      importCtx: useImportContext(),
      conversionCtx: useConversionContext(),
    }),
    { wrapper },
  );
  await act(async () => {});
  return result;
}

describe('useConversion', () => {
  let progressCallback;

  beforeEach(() => {
    vi.clearAllMocks();
    progressCallback = null;
    onConversionProgress.mockImplementation((cb) => {
      progressCallback = cb;
      return Promise.resolve(() => {});
    });
    settingsToConversionOptions.mockReturnValue({ outputFolder: '/out' });
    convertPdfToEpub.mockResolvedValue({ outputPath: '/out/test.epub' });
  });

  it('sets up progress listener on mount', async () => {
    await renderUseConversion();
    expect(onConversionProgress).toHaveBeenCalledWith(expect.any(Function));
  });

  it('does nothing when startConversion is called with empty paths', async () => {
    const { result } = await renderUseConversion();

    await act(async () => {
      result.current.conversion.startConversion([]);
    });

    expect(convertPdfToEpub).not.toHaveBeenCalled();
  });

  it('does nothing when startConversion is called with null', async () => {
    const { result } = await renderUseConversion();

    await act(async () => {
      result.current.conversion.startConversion(null);
    });

    expect(convertPdfToEpub).not.toHaveBeenCalled();
  });

  it('enqueues files and sets status to converting', async () => {
    const { result } = await renderUseConversion();

    await act(async () => {
      result.current.importCtx.dispatch({
        type: 'ADD_FILES',
        files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }],
      });
    });

    await act(async () => {
      result.current.conversion.startConversion(['/a.pdf']);
    });

    const file = result.current.importCtx.state.files.get('/a.pdf');
    expect(file.status).toBe('converted');
    expect(convertPdfToEpub).toHaveBeenCalledWith('/a.pdf', { outputFolder: '/out' });
  });

  it('uses same settings for all files in a batch', async () => {
    const { result } = await renderUseConversion();

    await act(async () => {
      result.current.importCtx.dispatch({
        type: 'ADD_FILES',
        files: [
          { path: '/a.pdf', name: 'a.pdf', status: 'ready' },
          { path: '/b.pdf', name: 'b.pdf', status: 'ready' },
        ],
      });
    });

    await act(async () => {
      result.current.conversion.startConversion(['/a.pdf', '/b.pdf']);
    });

    expect(convertPdfToEpub).toHaveBeenCalledTimes(2);
    expect(settingsToConversionOptions).toHaveBeenCalledTimes(2);
    const firstCallSettings = settingsToConversionOptions.mock.calls[0][0];
    const secondCallSettings = settingsToConversionOptions.mock.calls[1][0];
    expect(firstCallSettings).toEqual(secondCallSettings);
  });

  it('applies per-document overrides when converting', async () => {
    const { result } = await renderUseConversion();

    await act(async () => {
      result.current.importCtx.dispatch({
        type: 'ADD_FILES',
        files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }],
      });
    });

    await act(async () => {
      result.current.importCtx.dispatch({
        type: 'SET_DOCUMENT_OVERRIDES',
        path: '/a.pdf',
        overrides: { images: { imageQuality: 'high' } },
      });
    });

    await act(async () => {
      result.current.conversion.startConversion(['/a.pdf']);
    });

    expect(getEffectiveSettings).toHaveBeenCalledWith(
      expect.anything(),
      { images: { imageQuality: 'high' } },
    );
    const mergedSettings = settingsToConversionOptions.mock.calls[0][0];
    expect(mergedSettings.images.imageQuality).toBe('high');
  });

  it('uses global settings for files without overrides', async () => {
    const { result } = await renderUseConversion();

    await act(async () => {
      result.current.importCtx.dispatch({
        type: 'ADD_FILES',
        files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }],
      });
    });

    await act(async () => {
      result.current.conversion.startConversion(['/a.pdf']);
    });

    expect(getEffectiveSettings).toHaveBeenCalledWith(
      expect.anything(),
      undefined,
    );
  });

  it('sets error status when conversion fails', async () => {
    convertPdfToEpub.mockRejectedValue(new Error('PDF too large'));
    const { result } = await renderUseConversion();

    await act(async () => {
      result.current.importCtx.dispatch({
        type: 'ADD_FILES',
        files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }],
      });
    });

    await act(async () => {
      result.current.conversion.startConversion(['/a.pdf']);
    });

    const file = result.current.importCtx.state.files.get('/a.pdf');
    expect(file.status).toBe('error');
    expect(file.errorMessage).toBe('PDF too large');
  });

  it('uses fallback error message when error has no message', async () => {
    convertPdfToEpub.mockRejectedValue(null);
    const { result } = await renderUseConversion();

    await act(async () => {
      result.current.importCtx.dispatch({
        type: 'ADD_FILES',
        files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }],
      });
    });

    await act(async () => {
      result.current.conversion.startConversion(['/a.pdf']);
    });

    const file = result.current.importCtx.state.files.get('/a.pdf');
    expect(file.status).toBe('error');
    expect(file.errorMessage).toBe('Conversion failed');
  });

  it('uses toString when error has no message property', async () => {
    convertPdfToEpub.mockRejectedValue({ toString: () => 'custom error string' });
    const { result } = await renderUseConversion();

    await act(async () => {
      result.current.importCtx.dispatch({
        type: 'ADD_FILES',
        files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }],
      });
    });

    await act(async () => {
      result.current.conversion.startConversion(['/a.pdf']);
    });

    const file = result.current.importCtx.state.files.get('/a.pdf');
    expect(file.errorMessage).toBe('custom error string');
  });

  it('stores output path on success', async () => {
    convertPdfToEpub.mockResolvedValue({ outputPath: '/out/result.epub' });
    const { result } = await renderUseConversion();

    await act(async () => {
      result.current.importCtx.dispatch({
        type: 'ADD_FILES',
        files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }],
      });
    });

    await act(async () => {
      result.current.conversion.startConversion(['/a.pdf']);
    });

    const file = result.current.importCtx.state.files.get('/a.pdf');
    expect(file.outputPath).toBe('/out/result.epub');
  });

  it('dispatches progress events to import context', async () => {
    const { result } = await renderUseConversion();

    await act(async () => {
      result.current.importCtx.dispatch({
        type: 'ADD_FILES',
        files: [{ path: '/a.pdf', name: 'a.pdf', status: 'converting' }],
      });
      result.current.importCtx.dispatch({
        type: 'UPDATE_STATUS',
        path: '/a.pdf',
        status: 'converting',
      });
    });

    await act(async () => {
      progressCallback({
        path: '/a.pdf',
        percent: 50,
        stage: 'text_extraction',
        message: 'Extracting text...',
      });
    });

    const file = result.current.importCtx.state.files.get('/a.pdf');
    expect(file.conversionProgress).toBe(50);
    expect(file.conversionStage).toBe('text_extraction');
  });

  it('adds log entries from progress events', async () => {
    const { result } = await renderUseConversion();

    await act(async () => {
      progressCallback({
        path: '/a.pdf',
        percent: 25,
        stage: 'structure',
        message: 'Detecting structure...',
      });
    });

    expect(result.current.conversionCtx.state.logEntries).toHaveLength(1);
    expect(result.current.conversionCtx.state.logEntries[0].message).toBe('Detecting structure...');
    expect(result.current.conversionCtx.state.logEntries[0].level).toBe('info');
  });

  it('logs error-stage progress as error level', async () => {
    const { result } = await renderUseConversion();

    await act(async () => {
      progressCallback({
        path: '/a.pdf',
        percent: 0,
        stage: 'error',
        message: 'Something went wrong',
      });
    });

    expect(result.current.conversionCtx.state.logEntries[0].level).toBe('error');
  });

  it('cancelAll resets file statuses to ready', async () => {
    const { result } = await renderUseConversion();

    await act(async () => {
      result.current.importCtx.dispatch({
        type: 'ADD_FILES',
        files: [
          { path: '/a.pdf', name: 'a.pdf', status: 'ready' },
          { path: '/b.pdf', name: 'b.pdf', status: 'ready' },
        ],
      });
    });

    await act(async () => {
      result.current.conversionCtx.dispatch({
        type: 'ENQUEUE_FILES',
        paths: ['/a.pdf', '/b.pdf'],
      });
    });

    await act(async () => {
      result.current.importCtx.dispatch({
        type: 'UPDATE_STATUS',
        path: '/a.pdf',
        status: 'converting',
      });
      result.current.importCtx.dispatch({
        type: 'UPDATE_STATUS',
        path: '/b.pdf',
        status: 'converting',
      });
    });

    await act(async () => {
      await result.current.conversion.cancelAll();
    });

    const fileA = result.current.importCtx.state.files.get('/a.pdf');
    const fileB = result.current.importCtx.state.files.get('/b.pdf');
    expect(fileA.status).toBe('ready');
    expect(fileB.status).toBe('ready');
  });

  it('cancelAll calls cancelConversion for active file', async () => {
    cancelConversion.mockResolvedValue(undefined);
    const { result } = await renderUseConversion();

    await act(async () => {
      result.current.importCtx.dispatch({
        type: 'ADD_FILES',
        files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }],
      });
    });

    await act(async () => {
      result.current.conversionCtx.dispatch({
        type: 'ENQUEUE_FILES',
        paths: ['/a.pdf'],
      });
    });

    await act(async () => {
      await result.current.conversion.cancelAll();
    });

    expect(cancelConversion).toHaveBeenCalledWith('/a.pdf');
  });

  it('cancelAll handles cancelConversion failure gracefully', async () => {
    cancelConversion.mockRejectedValue(new Error('cancel failed'));
    const { result } = await renderUseConversion();

    await act(async () => {
      result.current.conversionCtx.dispatch({
        type: 'ENQUEUE_FILES',
        paths: ['/a.pdf'],
      });
    });

    await act(async () => {
      await result.current.conversion.cancelAll();
    });

    expect(result.current.conversionCtx.state.activeFile).toBeNull();
  });

  it('passes bookId to settingsToConversionOptions', async () => {
    const { result } = await renderUseConversion();

    await act(async () => {
      result.current.importCtx.dispatch({
        type: 'ADD_FILES',
        files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready', bookId: 'uuid-123', storedPdfPath: '/stored/a.pdf' }],
      });
    });

    await act(async () => {
      result.current.conversion.startConversion(['/a.pdf']);
    });

    expect(settingsToConversionOptions).toHaveBeenCalledWith(
      expect.anything(),
      { bookId: 'uuid-123' },
    );
  });

  it('uses storedPdfPath for conversion when available', async () => {
    const { result } = await renderUseConversion();

    await act(async () => {
      result.current.importCtx.dispatch({
        type: 'ADD_FILES',
        files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready', bookId: 'uuid-123', storedPdfPath: '/stored/a.pdf' }],
      });
    });

    await act(async () => {
      result.current.conversion.startConversion(['/a.pdf']);
    });

    expect(convertPdfToEpub).toHaveBeenCalledWith('/stored/a.pdf', { outputFolder: '/out' });
  });

  it('falls back to original path when storedPdfPath is absent', async () => {
    const { result } = await renderUseConversion();

    await act(async () => {
      result.current.importCtx.dispatch({
        type: 'ADD_FILES',
        files: [{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }],
      });
    });

    await act(async () => {
      result.current.conversion.startConversion(['/a.pdf']);
    });

    expect(convertPdfToEpub).toHaveBeenCalledWith('/a.pdf', { outputFolder: '/out' });
  });

  it('reports isConverting based on activeFile', async () => {
    const { result } = await renderUseConversion();

    expect(result.current.conversion.isConverting).toBe(false);

    await act(async () => {
      result.current.conversionCtx.dispatch({
        type: 'ENQUEUE_FILES',
        paths: ['/a.pdf'],
      });
    });

    expect(result.current.conversion.isConverting).toBe(true);
  });
});
