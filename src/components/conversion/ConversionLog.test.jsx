import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { useEffect } from 'react';
import { ConversionProvider, useConversionContext } from '../../contexts/ConversionContext';
import { ConversionLog } from './ConversionLog';

function SeedLog({ entries, children }) {
  const { dispatch } = useConversionContext();
  useEffect(() => {
    for (const entry of entries) {
      dispatch({ type: 'ADD_LOG_ENTRY', entry });
    }
  }, []);
  return children;
}

function Wrapper({ children }) {
  return <ConversionProvider>{children}</ConversionProvider>;
}

beforeEach(() => {
  Element.prototype.scrollIntoView = () => {};
});

describe('ConversionLog', () => {
  it('shows "Conversion log" heading', () => {
    render(
      <Wrapper>
        <ConversionLog />
      </Wrapper>,
    );
    expect(screen.getByText('Conversion log')).toBeInTheDocument();
  });

  it('shows waiting message when no entries', () => {
    render(
      <Wrapper>
        <ConversionLog />
      </Wrapper>,
    );
    expect(screen.getByText('Waiting for conversion to start...')).toBeInTheDocument();
  });

  it('renders log entries', () => {
    render(
      <Wrapper>
        <SeedLog
          entries={[
            { timestamp: 1000, message: 'Extracting text...', level: 'info' },
            { timestamp: 2000, message: 'Building chapters...', level: 'info' },
          ]}
        >
          <ConversionLog />
        </SeedLog>
      </Wrapper>,
    );
    expect(screen.getByText('Extracting text...')).toBeInTheDocument();
    expect(screen.getByText('Building chapters...')).toBeInTheDocument();
  });

  it('hides waiting message when entries exist', () => {
    render(
      <Wrapper>
        <SeedLog
          entries={[{ timestamp: 1000, message: 'Started', level: 'info' }]}
        >
          <ConversionLog />
        </SeedLog>
      </Wrapper>,
    );
    expect(screen.queryByText('Waiting for conversion to start...')).not.toBeInTheDocument();
  });

  it('has aria-live and aria-label for accessibility', () => {
    render(
      <Wrapper>
        <ConversionLog />
      </Wrapper>,
    );
    const log = screen.getByLabelText('Conversion log entries');
    expect(log).toHaveAttribute('aria-live', 'polite');
  });
});
