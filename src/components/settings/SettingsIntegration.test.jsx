import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import { useEffect } from 'react';
import { SettingsScreen } from './SettingsScreen';
import { SettingsProvider, useSettings } from '../../contexts/SettingsContext';
import { ImportProvider, useImportContext } from '../../contexts/ImportContext';
import { ConversionOptions } from '../library/ConversionOptions';

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

vi.mock('sonner', () => ({
  toast: {
    warning: vi.fn(),
    success: vi.fn(),
    error: vi.fn(),
  },
}));

function SeedFile({ file, children }) {
  const { dispatch } = useImportContext();
  useEffect(() => {
    dispatch({ type: 'ADD_FILES', files: [file] });
  }, []);
  return children;
}

const testFile = {
  path: '/test/doc.pdf',
  name: 'doc.pdf',
  size: 1024,
  status: 'ready',
  metadata: { pageCount: 10, fileSize: 1024 },
};

describe('Settings Integration', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('changing a setting on SettingsScreen is immediately visible in ConversionOptions', async () => {
    render(
      <SettingsProvider>
        <ImportProvider>
          <MemoryRouter>
            <SettingsScreen />
            <SeedFile file={testFile}>
              <ConversionOptions file={testFile} />
            </SeedFile>
          </MemoryRouter>
        </ImportProvider>
      </SettingsProvider>,
    );

    await userEvent.click(screen.getByText('Conversion options'));

    const fontSizeInputsBefore = screen.getAllByDisplayValue('12');
    expect(fontSizeInputsBefore.length).toBe(2);

    fireEvent.change(fontSizeInputsBefore[0], { target: { value: '18' } });

    const fontSizeInputsAfter = screen.getAllByDisplayValue('18');
    expect(fontSizeInputsAfter.length).toBe(2);
  });

  it('settings persist after simulated app reload', async () => {
    const { loadSettings, saveSettings } = await import('../../lib/settings');

    const { unmount } = render(
      <SettingsProvider>
        <SettingsScreen />
      </SettingsProvider>,
    );

    const toggle = screen.getByLabelText('Detect headings');
    await userEvent.click(toggle);
    expect(toggle).toHaveAttribute('aria-checked', 'false');

    await waitFor(
      () => {
        const calls = saveSettings.mock.calls;
        const lastCall = calls[calls.length - 1]?.[0];
        expect(lastCall?.structure?.detectHeadings).toBe(false);
      },
      { timeout: 500 },
    );

    const savedSettings = saveSettings.mock.calls[saveSettings.mock.calls.length - 1][0];

    unmount();

    loadSettings.mockResolvedValueOnce({ ...savedSettings });

    function ReloadConsumer() {
      const { settings, loaded } = useSettings();
      return (
        <div>
          <span data-testid="reload-loaded">{loaded.toString()}</span>
          <span data-testid="reload-headings">{settings.structure.detectHeadings.toString()}</span>
        </div>
      );
    }

    render(
      <SettingsProvider>
        <ReloadConsumer />
      </SettingsProvider>,
    );

    await waitFor(() => {
      expect(screen.getByTestId('reload-loaded')).toHaveTextContent('true');
    });
    expect(screen.getByTestId('reload-headings')).toHaveTextContent('false');
  });

  it('WebP cross-validation in SettingsScreen produces correct automatic changes and toast', async () => {
    const { toast } = await import('sonner');
    render(
      <SettingsProvider>
        <SettingsScreen />
      </SettingsProvider>,
    );

    const epubSelect = screen.getByDisplayValue('EPUB 3');
    await userEvent.selectOptions(epubSelect, 'epub2');
    expect(epubSelect).toHaveValue('epub2');

    const webpToggle = screen.getByLabelText('Convert to WebP');
    await userEvent.click(webpToggle);

    expect(webpToggle).toHaveAttribute('aria-checked', 'true');
    expect(epubSelect).toHaveValue('epub3');
    expect(toast.warning).toHaveBeenCalledWith(
      expect.stringContaining('WebP images require EPUB 3'),
    );

    toast.warning.mockClear();

    await userEvent.selectOptions(epubSelect, 'epub2');

    expect(webpToggle).toHaveAttribute('aria-checked', 'false');
    expect(epubSelect).toHaveValue('epub2');
    expect(toast.warning).toHaveBeenCalledWith(
      expect.stringContaining('WebP images are not supported in EPUB 2'),
    );
  });
});
