import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import { ImportProvider, useImportContext } from '../../contexts/ImportContext';
import { ConversionOptions } from './ConversionOptions';
import { useEffect } from 'react';

function SeedAndRender({ file }) {
  const { dispatch } = useImportContext();
  useEffect(() => {
    dispatch({ type: 'ADD_FILES', files: [file] });
  }, []);
  return <ConversionOptions file={file} />;
}

function renderOptions(fileOverrides = {}) {
  const file = {
    path: '/test/doc.pdf',
    name: 'doc.pdf',
    size: 1024,
    status: 'ready',
    metadata: { pageCount: 10, fileSize: 1024 },
    ...fileOverrides,
  };

  return render(
    <ImportProvider>
      <MemoryRouter>
        <SeedAndRender file={file} />
      </MemoryRouter>
    </ImportProvider>,
  );
}

describe('ConversionOptions', () => {
  it('renders collapsed by default', () => {
    renderOptions();
    expect(screen.getByText('Conversion options')).toBeInTheDocument();
    expect(screen.queryByText('Split chapters by')).not.toBeInTheDocument();
  });

  it('expands when clicked', async () => {
    const user = userEvent.setup();
    renderOptions();

    await user.click(screen.getByText('Conversion options'));

    expect(screen.getByText('Split chapters by')).toBeInTheDocument();
    expect(screen.getByText('Heading level threshold')).toBeInTheDocument();
    expect(screen.getByText('Base font size')).toBeInTheDocument();
    expect(screen.getByText('Image quality')).toBeInTheDocument();
    expect(screen.getByText('Page range')).toBeInTheDocument();
  });

  it('shows override count when overrides exist', () => {
    renderOptions({
      overrides: {
        pageHandling: { splitChaptersBy: 'heading2' },
        output: { baseFontSize: 14 },
      },
    });
    expect(screen.getByText(/2 custom/)).toBeInTheDocument();
  });

  it('collapses back when clicked again', async () => {
    const user = userEvent.setup();
    renderOptions();

    await user.click(screen.getByText('Conversion options'));
    expect(screen.getByText('Split chapters by')).toBeInTheDocument();

    await user.click(screen.getByText('Conversion options'));
    expect(screen.queryByText('Split chapters by')).not.toBeInTheDocument();
  });
});
