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
      <button data-testid="toggle-headings" onClick={() => updateSetting('structure', 'detectHeadings', false)}>
        Toggle
      </button>
      <button data-testid="enable-webp" onClick={() => updateSetting('images', 'convertToWebP', true)}>
        Enable WebP
      </button>
      <button data-testid="set-epub2" onClick={() => updateSetting('output', 'epubVersion', 'epub2')}>
        Set EPUB 2
      </button>
      <button data-testid="reset" onClick={resetToDefaults}>
        Reset
      </button>
    </div>
  );
}

function renderWithProvider() {
  return render(
    <SettingsProvider>
      <TestConsumer />
    </SettingsProvider>,
  );
}

describe('SettingsContext', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('initial state is DEFAULT_SETTINGS', () => {
    renderWithProvider();
    expect(screen.getByTestId('detect-headings')).toHaveTextContent('true');
    expect(screen.getByTestId('epub-version')).toHaveTextContent('epub3');
    expect(screen.getByTestId('text-alignment')).toHaveTextContent('justify');
    expect(screen.getByTestId('keep-page-breaks')).toHaveTextContent('false');
  });

  it('updateSetting updates a value in the correct group', async () => {
    renderWithProvider();
    await userEvent.click(screen.getByTestId('toggle-headings'));
    expect(screen.getByTestId('detect-headings')).toHaveTextContent('false');
  });

  it('WebP + EPUB 2 cross-validation triggers version upgrade', async () => {
    const { toast } = await import('sonner');
    renderWithProvider();

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
    renderWithProvider();

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
    renderWithProvider();

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
    renderWithProvider();

    await act(async () => {
      await userEvent.click(screen.getByTestId('toggle-headings'));
    });

    await waitFor(
      () => expect(saveSettings).toHaveBeenCalled(),
      { timeout: 500 },
    );
  });

  it('throws when useSettings is used outside provider', () => {
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    expect(() => render(<TestConsumer />)).toThrow(
      'useSettings must be used within a SettingsProvider',
    );
    consoleSpy.mockRestore();
  });
});
