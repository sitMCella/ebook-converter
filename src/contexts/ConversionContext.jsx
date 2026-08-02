import { createContext, useContext, useReducer } from 'react';

const ConversionContext = createContext(null);

const initialState = {
  queue: [],
  activeFile: null,
  completedFiles: [],
  logEntries: [],
  isComplete: false,
};

function conversionReducer(state, action) {
  switch (action.type) {
    case 'ENQUEUE_FILES': {
      const first = action.paths[0] || null;
      const rest = action.paths.slice(1);
      return {
        ...state,
        activeFile: first,
        queue: rest,
        completedFiles: [],
        logEntries: [],
        isComplete: false,
      };
    }
    case 'START_NEXT': {
      if (state.queue.length === 0) {
        return { ...state, activeFile: null, isComplete: true };
      }
      const [next, ...rest] = state.queue;
      return { ...state, activeFile: next, queue: rest };
    }
    case 'COMPLETE_ACTIVE': {
      return {
        ...state,
        completedFiles: [...state.completedFiles, action.path],
      };
    }
    case 'FAIL_ACTIVE': {
      return {
        ...state,
        completedFiles: [...state.completedFiles, action.path],
      };
    }
    case 'CANCEL_ALL': {
      return {
        ...state,
        queue: [],
        activeFile: null,
        isComplete: state.completedFiles.length > 0,
      };
    }
    case 'ADD_LOG_ENTRY': {
      return {
        ...state,
        logEntries: [...state.logEntries, action.entry],
      };
    }
    case 'CLEAR_LOG': {
      return { ...state, logEntries: [] };
    }
    default:
      return state;
  }
}

export function ConversionProvider({ children }) {
  const [state, dispatch] = useReducer(conversionReducer, initialState);
  return (
    <ConversionContext.Provider value={{ state, dispatch }}>
      {children}
    </ConversionContext.Provider>
  );
}

export function useConversionContext() {
  const context = useContext(ConversionContext);
  if (!context) {
    throw new Error('useConversionContext must be used within a ConversionProvider');
  }
  return context;
}
