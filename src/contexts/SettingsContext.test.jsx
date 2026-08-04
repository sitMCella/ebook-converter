import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, act, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { SettingsProvider, useSettings } from './SettingsContext';
import { DEFAULT_SETTINGS } from '../lib/settings';

vi.mock('../lib/settings', async () => {
  const actual = await vi.importActual('../lib/settings');
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

function TestConsumer() {
  const { settings, loaded, updateSetting, resetToDefaults } = useSettings();
  return (
    <div>
      <span data-testid="loaded">{loaded.toString()}</span>
      <span data-testid="detect-headings">{settings.structure.detectHeadings.toString()}</span>
      <span data-testid="epub-version">{settings.output.epubVersion}</span>
      <span data-testid="convert-webp">{settings.images.convertToWebP.toString()}</span>
      <span data-testid="text-alignment">{settings.output.textAlignment}</span>
      <span data-testid="keep-page-breaks">{settings.pageHandling.keepPageBreaks.toString()}</span>
      <span data-testid="split-chapters">{settings.pageHandling.splitChaptersBy}</span>
      <button data-testid="toggle-headings" onClick={() => updateSetting('structure', 'detectHeadings', false)}>
        Toggle
      </button>
      <button data-testid="enable-webp" onClick={() => updateSetting('images', 'convertToWebP', true)}>
        Enable WebP
      </button>
      <button data-testid="set-epub2" onClick={() => updateSetting('output', 'epubVersion', 'epub2')}>
        Set EPUB 2
      </button>
      <button data-testid="set-split-heading2" onClick={() => updateSetting('pageHandling', 'splitChaptersBy', 'heading2')}>
        Split Heading 2
      </button>
      <button data-testid="reset" onClick={resetToDefaults}>
        Reset
      </button>
    </div>
  );
}

async function renderWithProvider() {
  const result = render(
    <SettingsProvider>
      <TestConsumer />
    </SettingsProvider>,
  );
  await act(async () => {});
  return result;
}

describe('SettingsContext', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('initial state is DEFAULT_SETTINGS', async () => {
    await renderWithProvider();
    expect(screen.getByTestId('detect-headings')).toHaveTextContent('true');
    expect(screen.getByTestId('epub-version')).toHaveTextContent('epub3');
    expect(screen.getByTestId('text-alignment')).toHaveTextContent('justify');
    expect(screen.getByTestId('keep-page-breaks')).toHaveTextContent('false');
  });

  it('updateSetting updates a value in the correct group', async () => {
    await renderWithProvider();
    await userEvent.click(screen.getByTestId('toggle-headings'));
    expect(screen.getByTestId('detect-headings')).toHaveTextContent('false');
  });

  it('WebP + EPUB 2 cross-validation triggers version upgrade', async () => {
    const { toast } = await import('sonner');
    await renderWithProvider();

    await act(async () => {
      await userEvent.click(screen.getByTestId('set-epub2'));
    });
    expect(screen.getByTestId('epub-version')).toHaveTextContent('epub2');

    await act(async () => {
      await userEvent.click(screen.getByTestId('enable-webp'));
    });
    expect(screen.getByTestId('epub-version')).toHaveTextContent('epub3');
    expect(toast.warning).toHaveBeenCalledWith(
      expect.stringContaining('WebP images require EPUB 3'),
    );
  });

  it('EPUB 2 + WebP cross-validation disables WebP', async () => {
    const { toast } = await import('sonner');
    await renderWithProvider();

    await act(async () => {
      await userEvent.click(screen.getByTestId('enable-webp'));
    });
    expect(screen.getByTestId('convert-webp')).toHaveTextContent('true');

    await act(async () => {
      await userEvent.click(screen.getByTestId('set-epub2'));
    });
    expect(screen.getByTestId('convert-webp')).toHaveTextContent('false');
    expect(toast.warning).toHaveBeenCalledWith(
      expect.stringContaining('WebP images are not supported in EPUB 2'),
    );
  });

  it('resetToDefaults reverts all settings', async () => {
    const { saveSettings } = await import('../lib/settings');
    await renderWithProvider();

    await act(async () => {
      await userEvent.click(screen.getByTestId('toggle-headings'));
    });
    expect(screen.getByTestId('detect-headings')).toHaveTextContent('false');

    await act(async () => {
      await userEvent.click(screen.getByTestId('reset'));
    });
    expect(screen.getByTestId('detect-headings')).toHaveTextContent('true');
    expect(saveSettings).toHaveBeenCalledWith(DEFAULT_SETTINGS);
  });

  it('debounced save is called after updateSetting', async () => {
    const { saveSettings } = await import('../lib/settings');
    await renderWithProvider();

    await act(async () => {
      await userEvent.click(screen.getByTestId('toggle-headings'));
    });

    await waitFor(
      () => expect(saveSettings).toHaveBeenCalled(),
      { timeout: 500 },
    );
  });

  it('disabling detect headings with heading-based split emits advisory warning', async () => {
    const { toast } = await import('sonner');
    await renderWithProvider();

    expect(screen.getByTestId('split-chapters')).toHaveTextContent('heading1');

    await act(async () => {
      await userEvent.click(screen.getByTestId('toggle-headings'));
    });
    expect(screen.getByTestId('detect-headings')).toHaveTextContent('false');
    expect(toast.warning).toHaveBeenCalledWith(
      expect.stringContaining('Chapter splitting by headings requires heading detection'),
    );
  });

  it('disabling detect headings with heading2 split also emits advisory warning', async () => {
    const { toast } = await import('sonner');
    await renderWithProvider();

    await act(async () => {
      await userEvent.click(screen.getByTestId('set-split-heading2'));
    });
    expect(screen.getByTestId('split-chapters')).toHaveTextContent('heading2');

    await act(async () => {
      await userEvent.click(screen.getByTestId('toggle-headings'));
    });
    expect(toast.warning).toHaveBeenCalledWith(
      expect.stringContaining('Chapter splitting by headings requires heading detection'),
    );
  });

  it('heading detection warning is advisory only — does not change splitChaptersBy', async () => {
    await renderWithProvider();

    await act(async () => {
      await userEvent.click(screen.getByTestId('toggle-headings'));
    });
    expect(screen.getByTestId('split-chapters')).toHaveTextContent('heading1');
  });

  it('loads settings from disk on mount', async () => {
    const { loadSettings } = await import('../lib/settings');
    const customSettings = {
      ...DEFAULT_SETTINGS,
      output: { ...DEFAULT_SETTINGS.output, baseFontSize: 18 },
    };
    loadSettings.mockResolvedValueOnce(customSettings);

    function FontConsumer() {
      const { settings, loaded } = useSettings();
      return (
        <div>
          <span data-testid="font-size">{settings.output.baseFontSize}</span>
          <span data-testid="is-loaded">{loaded.toString()}</span>
        </div>
      );
    }

    render(
      <SettingsProvider>
        <FontConsumer />
      </SettingsProvider>,
    );
    await act(async () => {});

    await waitFor(() => {
      expect(screen.getByTestId('is-loaded')).toHaveTextContent('true');
    });
    expect(screen.getByTestId('font-size')).toHaveTextContent('18');
  });

  it('throws when useSettings is used outside provider', () => {
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    expect(() => render(<TestConsumer />)).toThrow(
      'useSettings must be used within a SettingsProvider',
    );
    consoleSpy.mockRestore();
  });
});
