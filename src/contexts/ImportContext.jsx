import { createContext, useContext, useReducer } from 'react';

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
        newFiles.set(action.path, {
          ...file,
          status: action.status,
          errorMessage: action.errorMessage,
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
    default:
      return state;
  }
}

export function ImportProvider({ children }) {
  const [state, dispatch] = useReducer(importReducer, initialState);
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
