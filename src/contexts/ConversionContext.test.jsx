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
