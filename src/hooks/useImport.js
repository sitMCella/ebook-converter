import { useState, useCallback } from 'react';
import { toast } from 'sonner';
import { useImportContext } from '../contexts/ImportContext';
import { validatePdf, getPdfMetadata, getFileSize, importPdf, saveBookMetadata } from '../lib/tauri';

export function useImport() {
  const { state, dispatch } = useImportContext();
  const [isProcessing, setIsProcessing] = useState(false);

  const stageFiles = useCallback(
    async (paths) => {
      if (!paths || paths.length === 0) return;

      const newPaths = [];
      for (const path of paths) {
        if (state.stagedFiles.has(path) || state.files.has(path)) {
          toast.info('File already imported', { duration: 3000 });
        } else {
          newPaths.push(path);
        }
      }

      if (newPaths.length === 0) return;

      setIsProcessing(true);

      const newFiles = newPaths.map((path) => ({
        path,
        name: path.split(/[\\/]/).pop(),
        size: 0,
        status: 'ready',
        errorMessage: undefined,
        metadata: null,
      }));

      dispatch({ type: 'STAGE_FILES', files: newFiles });

      await Promise.all(
        newPaths.map(async (path) => {
          try {
            const size = await getFileSize(path);
            dispatch({
              type: 'SET_STAGED_METADATA',
              path,
              metadata: { fileSize: size },
            });

            const validation = await validatePdf(path);

            if (validation.status === 'encrypted') {
              dispatch({
                type: 'UPDATE_STAGED_STATUS',
                path,
                status: 'error',
                errorMessage:
                  'This file is password-protected. Encrypted PDFs are not supported.',
              });
              return;
            }

            if (validation.status === 'error') {
              dispatch({
                type: 'UPDATE_STAGED_STATUS',
                path,
                status: 'error',
                errorMessage:
                  validation.message ||
                  'This file could not be read. It may be corrupted or password-protected.',
              });
              return;
            }

            const metadata = await getPdfMetadata(path);
            dispatch({ type: 'SET_STAGED_METADATA', path, metadata });
          } catch (err) {
            dispatch({
              type: 'UPDATE_STAGED_STATUS',
              path,
              status: 'error',
              errorMessage: `Failed to process file: ${err.message || err}`,
            });
          }
        })
      );

      setIsProcessing(false);
    },
    [state.stagedFiles, state.files, dispatch]
  );

  const importStagedFiles = useCallback(
    async (paths) => {
      if (!paths || paths.length === 0) return;

      setIsProcessing(true);

      let successCount = 0;

      await Promise.all(
        paths.map(async (path) => {
          const file = state.stagedFiles.get(path);
          if (!file || file.status !== 'ready') return;

          try {
            const stored = await importPdf(path);
            dispatch({
              type: 'IMPORT_TO_LIBRARY',
              path,
              bookId: stored.bookId,
              storedPdfPath: stored.storedPdfPath,
            });
            successCount++;

            if (stored.bookId) {
              const name = path.split(/[\\/]/).pop();
              const metadata = file.metadata || {};
              await saveBookMetadata({
                bookId: stored.bookId,
                storedPdfPath: stored.storedPdfPath,
                originalPath: path,
                originalName: name,
                fileSize: metadata.fileSize || 0,
                title: metadata.title || null,
                author: metadata.author || null,
                pageCount: metadata.pageCount || 0,
                pdfVersion: metadata.pdfVersion || null,
                createdDate: metadata.createdDate || null,
                modifiedDate: metadata.modifiedDate || null,
                producer: metadata.producer || null,
                status: 'ready',
              });
            }
          } catch (err) {
            dispatch({
              type: 'UPDATE_STAGED_STATUS',
              path,
              status: 'error',
              errorMessage: `Failed to import file: ${err.message || err}`,
            });
          }
        })
      );

      setIsProcessing(false);
      return successCount;
    },
    [state.stagedFiles, dispatch]
  );

  return { stageFiles, importStagedFiles, isProcessing };
}
