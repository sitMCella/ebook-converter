import { useCallback, useEffect, useRef } from 'react';
import { useConversionContext } from '../contexts/ConversionContext';
import { useImportContext } from '../contexts/ImportContext';
import { useSettings } from '../contexts/SettingsContext';
import { convertPdfToEpub, cancelConversion, onConversionProgress, saveBookMetadata } from '../lib/tauri';
import { getEffectiveSettings, settingsToConversionOptions } from '../lib/settings';

let isProcessing = false;
const pendingQueue = [];

export function _resetProcessingState() {
  isProcessing = false;
  pendingQueue.length = 0;
}

export function useConversion() {
  const { state: conversionState, dispatch: conversionDispatch } = useConversionContext();
  const { state: importState, dispatch: importDispatch } = useImportContext();
  const { settings: globalSettings } = useSettings();

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
        const file = importState.files.get(path);
        const settings = getEffectiveSettings(globalSettings, file?.overrides);
        const bookId = file?.bookId;
        const pdfPath = file?.storedPdfPath || path;
        const options = settingsToConversionOptions(settings, { bookId });

        const result = await convertPdfToEpub(pdfPath, options);

        importDispatch({
          type: 'SET_CONVERSION_RESULT',
          path,
          outputPath: result.outputPath,
          result,
        });

        if (bookId) {
          try {
            await saveBookMetadata({
              bookId: file.bookId,
              storedPdfPath: file.storedPdfPath,
              originalPath: path,
              originalName: file.name,
              fileSize: file.metadata?.fileSize || file.size || 0,
              title: file.metadata?.title || null,
              author: file.metadata?.author || null,
              pageCount: file.metadata?.pageCount || 0,
              pdfVersion: file.metadata?.pdfVersion || null,
              createdDate: file.metadata?.createdDate || null,
              modifiedDate: file.metadata?.modifiedDate || null,
              producer: file.metadata?.producer || null,
              status: 'converted',
              outputPath: result.outputPath,
              images: result.images,
              epubFileSize: result.fileSize,
              conversionSettings: file.overrides || null,
            });
          } catch {
            // metadata persistence is best-effort
          }
        }

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
    [importState.files, importDispatch, conversionDispatch, globalSettings],
  );

  const convertFileRef = useRef(convertFile);
  convertFileRef.current = convertFile;

  const processQueue = useCallback(
    async () => {
      isProcessing = true;

      while (pendingQueue.length > 0 && isProcessing) {
        const path = pendingQueue.shift();
        await convertFileRef.current(path);

        if (isProcessing) {
          conversionDispatch({ type: 'START_NEXT' });
        }
      }

      isProcessing = false;
    },
    [conversionDispatch],
  );

  const startConversion = useCallback(
    (paths) => {
      if (!paths || paths.length === 0) return;

      for (const path of paths) {
        importDispatch({ type: 'UPDATE_STATUS', path, status: 'converting' });
      }

      if (isProcessing) {
        pendingQueue.push(...paths);
        conversionDispatch({ type: 'APPEND_TO_QUEUE', paths });
      } else {
        pendingQueue.length = 0;
        pendingQueue.push(...paths);
        conversionDispatch({ type: 'ENQUEUE_FILES', paths });
        processQueue();
      }
    },
    [conversionDispatch, importDispatch, processQueue],
  );

  const cancelAll = useCallback(async () => {
    isProcessing = false;
    pendingQueue.length = 0;

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
