import { createContext, useContext, useReducer, useEffect } from 'react';
import { listBooks } from '../lib/tauri';

const ImportContext = createContext(null);

const initialState = {
  files: new Map(),
  selectedPaths: new Set(),
};

function importReducer(state, action) {
  switch (action.type) {
    case 'ADD_FILES': {
      const newFiles = new Map(state.files);
      for (const file of action.files) {
        if (!newFiles.has(file.path)) {
          newFiles.set(file.path, file);
        }
      }
      return { ...state, files: newFiles };
    }
    case 'REMOVE_FILES': {
      const newFiles = new Map(state.files);
      const newSelected = new Set(state.selectedPaths);
      for (const path of action.paths) {
        newFiles.delete(path);
        newSelected.delete(path);
      }
      return { files: newFiles, selectedPaths: newSelected };
    }
    case 'UPDATE_STATUS': {
      const newFiles = new Map(state.files);
      const file = newFiles.get(action.path);
      if (file) {
        const updated = {
          ...file,
          status: action.status,
          errorMessage: action.errorMessage,
        };
        if (action.status === 'ready' || action.status === 'error') {
          delete updated.conversionProgress;
          delete updated.conversionStage;
        }
        newFiles.set(action.path, updated);
      }
      return { ...state, files: newFiles };
    }
    case 'SET_CONVERSION_PROGRESS': {
      const newFiles = new Map(state.files);
      const file = newFiles.get(action.path);
      if (file && file.status === 'converting') {
        newFiles.set(action.path, {
          ...file,
          conversionProgress: action.percent,
          conversionStage: action.stage,
        });
      }
      return { ...state, files: newFiles };
    }
    case 'SET_CONVERSION_RESULT': {
      const newFiles = new Map(state.files);
      const file = newFiles.get(action.path);
      if (file) {
        newFiles.set(action.path, {
          ...file,
          status: 'converted',
          outputPath: action.outputPath,
          conversionResult: action.result,
          conversionProgress: undefined,
          conversionStage: undefined,
        });
      }
      return { ...state, files: newFiles };
    }
    case 'SET_STORAGE_INFO': {
      const newFiles = new Map(state.files);
      const file = newFiles.get(action.path);
      if (file) {
        newFiles.set(action.path, {
          ...file,
          bookId: action.bookId,
          storedPdfPath: action.storedPdfPath,
        });
      }
      return { ...state, files: newFiles };
    }
    case 'SET_METADATA': {
      const newFiles = new Map(state.files);
      const file = newFiles.get(action.path);
      if (file) {
        newFiles.set(action.path, { ...file, metadata: action.metadata });
      }
      return { ...state, files: newFiles };
    }
    case 'TOGGLE_SELECTION': {
      const newSelected = new Set(state.selectedPaths);
      if (newSelected.has(action.path)) {
        newSelected.delete(action.path);
      } else {
        newSelected.add(action.path);
      }
      return { ...state, selectedPaths: newSelected };
    }
    case 'SELECT_ALL': {
      const newSelected = new Set(state.files.keys());
      return { ...state, selectedPaths: newSelected };
    }
    case 'DESELECT_ALL': {
      return { ...state, selectedPaths: new Set() };
    }
    case 'SET_DOCUMENT_OVERRIDES': {
      const newFiles = new Map(state.files);
      const file = newFiles.get(action.path);
      if (file) {
        newFiles.set(action.path, {
          ...file,
          overrides: action.overrides,
        });
      }
      return { ...state, files: newFiles };
    }
    case 'LOAD_LIBRARY': {
      const newFiles = new Map(state.files);
      for (const book of action.books) {
        const path = book.originalPath;
        if (!newFiles.has(path)) {
          const fileData = {
            path,
            name: book.originalName,
            size: book.fileSize || 0,
            status: book.status || 'ready',
            errorMessage: undefined,
            bookId: book.bookId,
            storedPdfPath: book.storedPdfPath,
            metadata: {
              title: book.title,
              author: book.author,
              pageCount: book.pageCount,
              pdfVersion: book.pdfVersion,
              createdDate: book.createdDate,
              modifiedDate: book.modifiedDate,
              producer: book.producer,
              fileSize: book.fileSize,
            },
          };
          if (book.status === 'converted' && book.outputPath) {
            fileData.outputPath = book.outputPath;
            fileData.conversionResult = {
              outputPath: book.outputPath,
              chapters: book.chapters || 0,
              images: book.images || 0,
              fileSize: book.epubFileSize || 0,
            };
          }
          newFiles.set(path, fileData);
        }
      }
      return { ...state, files: newFiles };
    }
    default:
      return state;
  }
}

export function ImportProvider({ children }) {
  const [state, dispatch] = useReducer(importReducer, initialState);

  useEffect(() => {
    listBooks()
      .then((books) => {
        if (books && books.length > 0) {
          dispatch({ type: 'LOAD_LIBRARY', books });
        }
      })
      .catch(() => {});
  }, []);

  return (
    <ImportContext.Provider value={{ state, dispatch }}>
      {children}
    </ImportContext.Provider>
  );
}

export function useImportContext() {
  const context = useContext(ImportContext);
  if (!context) {
    throw new Error('useImportContext must be used within an ImportProvider');
  }
  return context;
}
