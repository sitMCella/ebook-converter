import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useEffect } from 'react';
import { MemoryRouter } from 'react-router-dom';
import { ImportProvider, useImportContext } from '../../contexts/ImportContext';
import { BatchActions } from './BatchActions';

const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return { ...actual, useNavigate: () => mockNavigate };
});

function Wrapper({ children }) {
  return (
    <ImportProvider>
      <MemoryRouter>{children}</MemoryRouter>
    </ImportProvider>
  );
}

function SeedState({ files = [], selectedPaths = [], children }) {
  const { dispatch } = useImportContext();
  useEffect(() => {
    if (files.length > 0) {
      dispatch({ type: 'ADD_FILES', files });
    }
    for (const path of selectedPaths) {
      dispatch({ type: 'TOGGLE_SELECTION', path });
    }
  }, []);
  return children;
}

describe('BatchActions', () => {
  it('disables "Remove selected" when no rows are selected', () => {
    render(
      <Wrapper>
        <BatchActions />
      </Wrapper>
    );
    expect(screen.getByText('Remove selected').closest('button')).toBeDisabled();
  });

  it('disables "Convert selected" when no convertible rows are selected', () => {
    render(
      <Wrapper>
        <BatchActions />
      </Wrapper>
    );
    const convertBtn = screen.getByText('Convert selected').closest('button');
    expect(convertBtn).toBeDisabled();
  });

  it('enables "Remove selected" when rows are selected', () => {
    render(
      <Wrapper>
        <SeedState
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }]}
          selectedPaths={['/a.pdf']}
        >
          <BatchActions />
        </SeedState>
      </Wrapper>
    );
    expect(screen.getByText('Remove selected').closest('button')).not.toBeDisabled();
  });

  it('enables "Convert selected" when ready rows are selected', () => {
    render(
      <Wrapper>
        <SeedState
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }]}
          selectedPaths={['/a.pdf']}
        >
          <BatchActions />
        </SeedState>
      </Wrapper>
    );
    const convertBtn = screen.getByText('Convert selected').closest('button');
    expect(convertBtn).not.toBeDisabled();
  });

  it('keeps "Convert selected" disabled when only error files are selected', () => {
    render(
      <Wrapper>
        <SeedState
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'error' }]}
          selectedPaths={['/a.pdf']}
        >
          <BatchActions />
        </SeedState>
      </Wrapper>
    );
    const convertBtn = screen.getByText('Convert selected').closest('button');
    expect(convertBtn).toBeDisabled();
  });

  it('shows confirmation dialog when "Remove selected" is clicked', async () => {
    const user = userEvent.setup();
    render(
      <Wrapper>
        <SeedState
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }]}
          selectedPaths={['/a.pdf']}
        >
          <BatchActions />
        </SeedState>
      </Wrapper>
    );
    await user.click(screen.getByText('Remove selected'));
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(screen.getByText(/Remove 1 file\(s\)/)).toBeInTheDocument();
  });

  it('removes files on confirm', async () => {
    const user = userEvent.setup();
    render(
      <Wrapper>
        <SeedState
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }]}
          selectedPaths={['/a.pdf']}
        >
          <BatchActions />
        </SeedState>
      </Wrapper>
    );
    await user.click(screen.getByText('Remove selected'));
    await user.click(screen.getByText('Confirm'));
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('navigates to /converted when "Convert selected" is clicked', async () => {
    const user = userEvent.setup();
    render(
      <Wrapper>
        <SeedState
          files={[{ path: '/a.pdf', name: 'a.pdf', status: 'ready' }]}
          selectedPaths={['/a.pdf']}
        >
          <BatchActions />
        </SeedState>
      </Wrapper>
    );
    await user.click(screen.getByText('Convert selected').closest('button'));
    expect(mockNavigate).toHaveBeenCalledWith('/converted');
  });
});
