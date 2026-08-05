import { describe, it, expect, vi } from 'vitest';
import { render, screen, act } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';

vi.mock('../../lib/tauri', async (importOriginal) => ({
  ...(await importOriginal()),
  listBooks: vi.fn().mockResolvedValue([]),
}));

vi.mock('../../lib/settings', async () => {
  const actual = await vi.importActual('../../lib/settings');
  return {
    ...actual,
    loadSettings: vi.fn(() => Promise.resolve({ ...actual.DEFAULT_SETTINGS })),
    saveSettings: vi.fn(() => Promise.resolve()),
  };
});

import { ImportProvider, useImportContext } from '../../contexts/ImportContext';
import { SettingsProvider } from '../../contexts/SettingsContext';
import { ConversionOptions } from './ConversionOptions';
import { useEffect } from 'react';

function SeedAndRender({ file }) {
  const { dispatch } = useImportContext();
  useEffect(() => {
    dispatch({ type: 'ADD_FILES', files: [file] });
  }, []);
  return <ConversionOptions file={file} />;
}

async function renderOptions(fileOverrides = {}) {
  const file = {
    path: '/test/doc.pdf',
    name: 'doc.pdf',
    size: 1024,
    status: 'ready',
    metadata: { pageCount: 10, fileSize: 1024 },
    ...fileOverrides,
  };

  const result = render(
    <SettingsProvider>
      <ImportProvider>
        <MemoryRouter>
          <SeedAndRender file={file} />
        </MemoryRouter>
      </ImportProvider>
    </SettingsProvider>,
  );
  await act(async () => {});
  return result;
}

describe('ConversionOptions', () => {
  it('renders collapsed by default', async () => {
    await renderOptions();
    expect(screen.getByText('Conversion options')).toBeInTheDocument();
    expect(screen.queryByText('Split chapters by')).not.toBeInTheDocument();
  });

  it('expands when clicked', async () => {
    const user = userEvent.setup();
    await renderOptions();

    await user.click(screen.getByText('Conversion options'));

    expect(screen.getByText('Split chapters by')).toBeInTheDocument();
    expect(screen.getByText('Heading level threshold')).toBeInTheDocument();
    expect(screen.getByText('Base font size')).toBeInTheDocument();
    expect(screen.getByText('Image quality')).toBeInTheDocument();
    expect(screen.getByText('Page range')).toBeInTheDocument();
  });

  it('shows override count when overrides exist', async () => {
    await renderOptions({
      overrides: {
        pageHandling: { splitChaptersBy: 'heading2' },
        output: { baseFontSize: 14 },
      },
    });
    expect(screen.getByText(/2 custom/)).toBeInTheDocument();
  });

  it('collapses back when clicked again', async () => {
    const user = userEvent.setup();
    await renderOptions();

    await user.click(screen.getByText('Conversion options'));
    expect(screen.getByText('Split chapters by')).toBeInTheDocument();

    await user.click(screen.getByText('Conversion options'));
    expect(screen.queryByText('Split chapters by')).not.toBeInTheDocument();
  });

  it('shows cover page override when expanded', async () => {
    const user = userEvent.setup();
    await renderOptions();

    await user.click(screen.getByText('Conversion options'));
    expect(screen.getByText('Cover page')).toBeInTheDocument();
  });

  it('cover page dropdown has all three options', async () => {
    const user = userEvent.setup();
    await renderOptions();

    await user.click(screen.getByText('Conversion options'));
    const selects = screen.getAllByRole('combobox');
    const coverSelect = selects.find((s) => {
      const options = s.querySelectorAll('option');
      return Array.from(options).some((o) => o.textContent.includes('Auto-detect'));
    });
    expect(coverSelect).toBeTruthy();
    const options = coverSelect.querySelectorAll('option');
    const values = Array.from(options).map((o) => o.value);
    expect(values).toContain('auto');
    expect(values).toContain('firstPage');
    expect(values).toContain('none');
  });

  it('counts cover page override in override count', async () => {
    await renderOptions({
      overrides: {
        pageHandling: { coverPage: 'none' },
      },
    });
    expect(screen.getByText(/1 custom/)).toBeInTheDocument();
  });
});
