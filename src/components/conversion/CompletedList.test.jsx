import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useEffect } from 'react';
import { MemoryRouter } from 'react-router-dom';

vi.mock('../../lib/tauri', async (importOriginal) => ({
  ...(await importOriginal()),
  listBooks: vi.fn().mockResolvedValue([]),
}));
import { ImportProvider, useImportContext } from '../../contexts/ImportContext';
import { ConversionProvider, useConversionContext } from '../../contexts/ConversionContext';
import { CompletedList } from './CompletedList';

const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return { ...actual, useNavigate: () => mockNavigate };
});

function SeedCompleted({ files, completedPaths, children }) {
  const { dispatch: importDispatch } = useImportContext();
  const { dispatch: conversionDispatch } = useConversionContext();
  useEffect(() => {
    if (files) {
      importDispatch({ type: 'ADD_FILES', files });
    }
    if (completedPaths) {
      conversionDispatch({ type: 'ENQUEUE_FILES', paths: completedPaths });
      for (const path of completedPaths) {
        conversionDispatch({ type: 'COMPLETE_ACTIVE', path });
        conversionDispatch({ type: 'START_NEXT' });
      }
    }
  }, []);
  return children;
}

function Wrapper({ children }) {
  return (
    <ImportProvider>
      <ConversionProvider>
        <MemoryRouter>{children}</MemoryRouter>
      </ConversionProvider>
    </ImportProvider>
  );
}

describe('CompletedList', () => {
  it('renders nothing when no completed files', () => {
    const { container } = render(
      <Wrapper>
        <CompletedList />
      </Wrapper>,
    );
    expect(container.firstChild).toBeNull();
  });

  it('shows "Completed" heading when files are completed', () => {
    render(
      <Wrapper>
        <SeedCompleted
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'converted' }]}
          completedPaths={['/a.pdf']}
        >
          <CompletedList />
        </SeedCompleted>
      </Wrapper>,
    );
    expect(screen.getByText('Completed')).toBeInTheDocument();
  });

  it('shows completed file names', () => {
    render(
      <Wrapper>
        <SeedCompleted
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'converted' }]}
          completedPaths={['/a.pdf']}
        >
          <CompletedList />
        </SeedCompleted>
      </Wrapper>,
    );
    expect(screen.getByText('a.pdf')).toBeInTheDocument();
  });

  it('navigates to /converted when a completed file is clicked', async () => {
    const user = userEvent.setup();
    render(
      <Wrapper>
        <SeedCompleted
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'converted' }]}
          completedPaths={['/a.pdf']}
        >
          <CompletedList />
        </SeedCompleted>
      </Wrapper>,
    );
    await user.click(screen.getByText('a.pdf'));
    expect(mockNavigate).toHaveBeenCalledWith('/converted', { state: { selectedPath: '/a.pdf' } });
  });

  it('shows status badge for each completed file', () => {
    render(
      <Wrapper>
        <SeedCompleted
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'converted' }]}
          completedPaths={['/a.pdf']}
        >
          <CompletedList />
        </SeedCompleted>
      </Wrapper>,
    );
    expect(screen.getByText('Converted')).toBeInTheDocument();
  });
});
