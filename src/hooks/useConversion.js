import { useCallback, useEffect, useRef } from 'react';
import { useConversionContext } from '../contexts/ConversionContext';
import { useImportContext } from '../contexts/ImportContext';
import { convertPdfToEpub, cancelConversion, onConversionProgress } from '../lib/tauri';
import { loadSettings, settingsToConversionOptions } from '../lib/settings';

export function useConversion() {
  const { state: conversionState, dispatch: conversionDispatch } = useConversionContext();
  const { dispatch: importDispatch } = useImportContext();
  const isConvertingRef = useRef(false);
  const settingsRef = useRef(null);

  useEffect(() => {
    let unlisten;

    const setup = async () => {
      unlisten = await onConversionProgress((progress) => {
        importDispatch({
          type: 'SET_CONVERSION_PROGRESS',
          path: progress.path,
          percent: progress.percent,
          stage: progress.stage,
        });

        conversionDispatch({
          type: 'ADD_LOG_ENTRY',
          entry: {
            timestamp: Date.now(),
            message: progress.message,
            level: progress.stage === 'error' ? 'error' : 'info',
          },
        });
      });
    };

    setup();

    return () => {
      if (typeof unlisten === 'function') unlisten();
    };
  }, [importDispatch, conversionDispatch]);

  const convertFile = useCallback(
    async (path) => {
      importDispatch({ type: 'UPDATE_STATUS', path, status: 'converting' });

      try {
        const settings = settingsRef.current || await loadSettings();
        const options = settingsToConversionOptions(settings);

        const result = await convertPdfToEpub(path, options);

        importDispatch({
          type: 'SET_CONVERSION_RESULT',
          path,
          outputPath: result.outputPath,
          result,
        });

        conversionDispatch({ type: 'COMPLETE_ACTIVE', path });
      } catch (error) {
        const message = error?.message || error?.toString() || 'Conversion failed';
        importDispatch({
          type: 'UPDATE_STATUS',
          path,
          status: 'error',
          errorMessage: message,
        });

        conversionDispatch({ type: 'FAIL_ACTIVE', path });

        conversionDispatch({
          type: 'ADD_LOG_ENTRY',
          entry: {
            timestamp: Date.now(),
            message: `Error: ${message}`,
            level: 'error',
          },
        });
      }
    },
    [importDispatch, conversionDispatch],
  );

  const processQueue = useCallback(
    async (paths) => {
      isConvertingRef.current = true;
      settingsRef.current = await loadSettings();

      for (const path of paths) {
        if (!isConvertingRef.current) break;
        await convertFile(path);

        if (isConvertingRef.current) {
          conversionDispatch({ type: 'START_NEXT' });
        }
      }

      isConvertingRef.current = false;
    },
    [convertFile, conversionDispatch],
  );

  const startConversion = useCallback(
    (paths) => {
      if (!paths || paths.length === 0) return;

      conversionDispatch({ type: 'ENQUEUE_FILES', paths });

      for (const path of paths) {
        importDispatch({ type: 'UPDATE_STATUS', path, status: 'converting' });
      }

      processQueue(paths);
    },
    [conversionDispatch, importDispatch, processQueue],
  );

  const cancelAll = useCallback(async () => {
    isConvertingRef.current = false;

    if (conversionState.activeFile) {
      try {
        await cancelConversion(conversionState.activeFile);
      } catch {
        // cancellation is best-effort
      }
    }

    const pathsToReset = [
      ...(conversionState.activeFile ? [conversionState.activeFile] : []),
      ...conversionState.queue,
    ];

    for (const path of pathsToReset) {
      importDispatch({ type: 'UPDATE_STATUS', path, status: 'ready' });
    }

    conversionDispatch({ type: 'CANCEL_ALL' });
  }, [conversionState.activeFile, conversionState.queue, importDispatch, conversionDispatch]);

  return {
    startConversion,
    cancelAll,
    isConverting: conversionState.activeFile != null,
  };
}
