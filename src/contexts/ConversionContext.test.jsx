import { describe, it, expect } from 'vitest';
import { render, act } from '@testing-library/react';
import { ConversionProvider, useConversionContext } from './ConversionContext';
import { useEffect, useState } from 'react';

function TestHarness({ action, onState }) {
  const { state, dispatch } = useConversionContext();
  const [dispatched, setDispatched] = useState(false);

  useEffect(() => {
    if (!dispatched && action) {
      dispatch(action);
      setDispatched(true);
    }
  }, [dispatched, action, dispatch]);

  useEffect(() => {
    onState?.(state);
  }, [state, onState]);

  return null;
}

function renderWithDispatch(action) {
  let latestState;
  render(
    <ConversionProvider>
      <TestHarness
        action={action}
        onState={(s) => { latestState = s; }}
      />
    </ConversionProvider>
  );
  return () => latestState;
}

describe('ConversionContext', () => {
  it('enqueues files with first as active', () => {
    const getState = renderWithDispatch({
      type: 'ENQUEUE_FILES',
      paths: ['/a.pdf', '/b.pdf', '/c.pdf'],
    });
    expect(getState().activeFile).toBe('/a.pdf');
    expect(getState().queue).toEqual(['/b.pdf', '/c.pdf']);
    expect(getState().isComplete).toBe(false);
  });

  it('starts next file from queue', () => {
    let dispatch;
    render(
      <ConversionProvider>
        <TestHarness
          onState={() => {}}
          action={null}
        />
        {(() => {
          function Inner() {
            const ctx = useConversionContext();
            dispatch = ctx.dispatch;
            return null;
          }
          return <Inner />;
        })()}
      </ConversionProvider>
    );

    act(() => {
      dispatch({ type: 'ENQUEUE_FILES', paths: ['/a.pdf', '/b.pdf'] });
    });
    act(() => {
      dispatch({ type: 'COMPLETE_ACTIVE', path: '/a.pdf' });
    });
    act(() => {
      dispatch({ type: 'START_NEXT' });
    });

    // After START_NEXT, /b.pdf should be active
  });

  it('completes when queue is empty', () => {
    let dispatch, getState;
    function Inner() {
      const ctx = useConversionContext();
      dispatch = ctx.dispatch;
      getState = () => ctx.state;
      return null;
    }

    render(
      <ConversionProvider>
        <Inner />
      </ConversionProvider>
    );

    act(() => dispatch({ type: 'ENQUEUE_FILES', paths: ['/a.pdf'] }));
    act(() => dispatch({ type: 'COMPLETE_ACTIVE', path: '/a.pdf' }));
    act(() => dispatch({ type: 'START_NEXT' }));

    expect(getState().isComplete).toBe(true);
    expect(getState().activeFile).toBeNull();
  });

  it('cancel clears queue and active', () => {
    let dispatch, getState;
    function Inner() {
      const ctx = useConversionContext();
      dispatch = ctx.dispatch;
      getState = () => ctx.state;
      return null;
    }

    render(
      <ConversionProvider>
        <Inner />
      </ConversionProvider>
    );

    act(() => dispatch({ type: 'ENQUEUE_FILES', paths: ['/a.pdf', '/b.pdf'] }));
    act(() => dispatch({ type: 'CANCEL_ALL' }));

    expect(getState().activeFile).toBeNull();
    expect(getState().queue).toEqual([]);
  });

  it('adds log entries', () => {
    let dispatch, getState;
    function Inner() {
      const ctx = useConversionContext();
      dispatch = ctx.dispatch;
      getState = () => ctx.state;
      return null;
    }

    render(
      <ConversionProvider>
        <Inner />
      </ConversionProvider>
    );

    act(() => dispatch({
      type: 'ADD_LOG_ENTRY',
      entry: { timestamp: 1000, message: 'Test log', level: 'info' },
    }));

    expect(getState().logEntries).toHaveLength(1);
    expect(getState().logEntries[0].message).toBe('Test log');
  });

  it('clears log entries', () => {
    let dispatch, getState;
    function Inner() {
      const ctx = useConversionContext();
      dispatch = ctx.dispatch;
      getState = () => ctx.state;
      return null;
    }

    render(
      <ConversionProvider>
        <Inner />
      </ConversionProvider>
    );

    act(() => dispatch({
      type: 'ADD_LOG_ENTRY',
      entry: { timestamp: 1000, message: 'Test', level: 'info' },
    }));
    expect(getState().logEntries).toHaveLength(1);

    act(() => dispatch({ type: 'CLEAR_LOG' }));
    expect(getState().logEntries).toHaveLength(0);
  });

  it('complete_active adds to completed', () => {
    let dispatch, getState;
    function Inner() {
      const ctx = useConversionContext();
      dispatch = ctx.dispatch;
      getState = () => ctx.state;
      return null;
    }

    render(
      <ConversionProvider>
        <Inner />
      </ConversionProvider>
    );

    act(() => dispatch({ type: 'ENQUEUE_FILES', paths: ['/a.pdf'] }));
    act(() => dispatch({ type: 'COMPLETE_ACTIVE', path: '/a.pdf' }));

    expect(getState().completedFiles).toContain('/a.pdf');
  });

  it('enqueue resets completed and log', () => {
    let dispatch, getState;
    function Inner() {
      const ctx = useConversionContext();
      dispatch = ctx.dispatch;
      getState = () => ctx.state;
      return null;
    }

    render(
      <ConversionProvider>
        <Inner />
      </ConversionProvider>
    );

    act(() => dispatch({ type: 'ENQUEUE_FILES', paths: ['/a.pdf'] }));
    act(() => dispatch({ type: 'COMPLETE_ACTIVE', path: '/a.pdf' }));
    act(() => dispatch({
      type: 'ADD_LOG_ENTRY',
      entry: { timestamp: 1000, message: 'done', level: 'info' },
    }));

    act(() => dispatch({ type: 'ENQUEUE_FILES', paths: ['/b.pdf'] }));
    expect(getState().completedFiles).toEqual([]);
    expect(getState().logEntries).toEqual([]);
  });

  it('cancel sets isComplete when files were completed', () => {
    let dispatch, getState;
    function Inner() {
      const ctx = useConversionContext();
      dispatch = ctx.dispatch;
      getState = () => ctx.state;
      return null;
    }

    render(
      <ConversionProvider>
        <Inner />
      </ConversionProvider>
    );

    act(() => dispatch({ type: 'ENQUEUE_FILES', paths: ['/a.pdf', '/b.pdf'] }));
    act(() => dispatch({ type: 'COMPLETE_ACTIVE', path: '/a.pdf' }));
    act(() => dispatch({ type: 'CANCEL_ALL' }));

    expect(getState().isComplete).toBe(true);
  });

  it('cancel sets isComplete false when no files were completed', () => {
    let dispatch, getState;
    function Inner() {
      const ctx = useConversionContext();
      dispatch = ctx.dispatch;
      getState = () => ctx.state;
      return null;
    }

    render(
      <ConversionProvider>
        <Inner />
      </ConversionProvider>
    );

    act(() => dispatch({ type: 'ENQUEUE_FILES', paths: ['/a.pdf'] }));
    act(() => dispatch({ type: 'CANCEL_ALL' }));

    expect(getState().isComplete).toBe(false);
  });

  it('appends to queue without resetting state', () => {
    let dispatch, getState;
    function Inner() {
      const ctx = useConversionContext();
      dispatch = ctx.dispatch;
      getState = () => ctx.state;
      return null;
    }

    render(
      <ConversionProvider>
        <Inner />
      </ConversionProvider>
    );

    act(() => dispatch({ type: 'ENQUEUE_FILES', paths: ['/a.pdf', '/b.pdf'] }));
    act(() => dispatch({ type: 'COMPLETE_ACTIVE', path: '/a.pdf' }));
    act(() => dispatch({
      type: 'ADD_LOG_ENTRY',
      entry: { timestamp: 1000, message: 'progress', level: 'info' },
    }));

    act(() => dispatch({ type: 'APPEND_TO_QUEUE', paths: ['/c.pdf', '/d.pdf'] }));

    expect(getState().activeFile).toBe('/a.pdf');
    expect(getState().queue).toEqual(['/b.pdf', '/c.pdf', '/d.pdf']);
    expect(getState().completedFiles).toContain('/a.pdf');
    expect(getState().logEntries).toHaveLength(1);
  });

  it('returns state unchanged for unknown action', () => {
    let dispatch, getState;
    function Inner() {
      const ctx = useConversionContext();
      dispatch = ctx.dispatch;
      getState = () => ctx.state;
      return null;
    }

    render(
      <ConversionProvider>
        <Inner />
      </ConversionProvider>
    );

    const before = getState();
    act(() => dispatch({ type: 'NONEXISTENT_ACTION' }));
    expect(getState()).toBe(before);
  });

  it('throws when useConversionContext is used outside provider', () => {
    const spy = vi.spyOn(console, 'error').mockImplementation(() => {});
    function Bad() {
      useConversionContext();
      return null;
    }
    expect(() => render(<Bad />)).toThrow('useConversionContext must be used within a ConversionProvider');
    spy.mockRestore();
  });

  it('fail_active adds to completed', () => {
    let dispatch, getState;
    function Inner() {
      const ctx = useConversionContext();
      dispatch = ctx.dispatch;
      getState = () => ctx.state;
      return null;
    }

    render(
      <ConversionProvider>
        <Inner />
      </ConversionProvider>
    );

    act(() => dispatch({ type: 'ENQUEUE_FILES', paths: ['/a.pdf'] }));
    act(() => dispatch({ type: 'FAIL_ACTIVE', path: '/a.pdf' }));

    expect(getState().completedFiles).toContain('/a.pdf');
  });
});
