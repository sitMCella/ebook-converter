import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, act, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { SettingsScreen } from './SettingsScreen';
import { SettingsProvider } from '../../contexts/SettingsContext';
import { DEFAULT_SETTINGS } from '../../lib/settings';

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

async function renderSettingsScreen() {
  const result = render(
    <SettingsProvider>
      <SettingsScreen />
    </SettingsProvider>,
  );
  await act(async () => {});
  return result;
}

describe('SettingsScreen', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders all four setting groups with correct headings', async () => {
    await renderSettingsScreen();
    expect(screen.getByText('Structure Detection')).toBeInTheDocument();
    expect(screen.getByText('Images')).toBeInTheDocument();
    expect(screen.getByText('Output Format')).toBeInTheDocument();
    expect(screen.getByText('Page Handling')).toBeInTheDocument();
  });

  it('renders the screen title', async () => {
    await renderSettingsScreen();
    expect(screen.getByText('Conversion settings')).toBeInTheDocument();
  });

  it('all toggle controls display their default values', async () => {
    await renderSettingsScreen();
    const detectHeadings = screen.getByLabelText('Detect headings');
    expect(detectHeadings).toHaveAttribute('aria-checked', 'true');

    const detectFootnotes = screen.getByLabelText('Detect footnotes');
    expect(detectFootnotes).toHaveAttribute('aria-checked', 'false');

    const extractImages = screen.getByLabelText('Extract images');
    expect(extractImages).toHaveAttribute('aria-checked', 'true');

    const convertToWebP = screen.getByLabelText('Convert to WebP');
    expect(convertToWebP).toHaveAttribute('aria-checked', 'false');

    const keepPageBreaks = screen.getByLabelText('Keep page breaks');
    expect(keepPageBreaks).toHaveAttribute('aria-checked', 'false');

    const removePageNumbers = screen.getByLabelText('Remove page numbers');
    expect(removePageNumbers).toHaveAttribute('aria-checked', 'true');
  });

  it('toggling a switch updates its value', async () => {
    await renderSettingsScreen();
    const toggle = screen.getByLabelText('Detect headings');
    expect(toggle).toHaveAttribute('aria-checked', 'true');
    await userEvent.click(toggle);
    expect(toggle).toHaveAttribute('aria-checked', 'false');
  });

  it('changing a dropdown updates its value', async () => {
    await renderSettingsScreen();
    const epubSelect = screen.getByDisplayValue('EPUB 3');
    await userEvent.selectOptions(epubSelect, 'epub2');
    expect(epubSelect).toHaveValue('epub2');
  });

  it('changing a number input updates its value', async () => {
    await renderSettingsScreen();
    const fontSizeInput = screen.getByDisplayValue('12');
    fireEvent.change(fontSizeInput, { target: { value: '16' } });
    expect(fontSizeInput).toHaveValue(16);
  });

  it('"Reset to defaults" button shows confirmation dialog', async () => {
    await renderSettingsScreen();
    await userEvent.click(screen.getByText('Reset to defaults'));
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(screen.getByText('Reset all settings to factory defaults? Per-document overrides are not affected.')).toBeInTheDocument();
  });

  it('confirming reset calls resetToDefaults', async () => {
    const { saveSettings } = await import('../../lib/settings');
    await renderSettingsScreen();

    const toggle = screen.getByLabelText('Detect headings');
    await userEvent.click(toggle);
    expect(toggle).toHaveAttribute('aria-checked', 'false');

    await userEvent.click(screen.getByText('Reset to defaults'));
    await userEvent.click(screen.getByText('Confirm'));
    expect(toggle).toHaveAttribute('aria-checked', 'true');
    expect(saveSettings).toHaveBeenCalledWith(DEFAULT_SETTINGS);
  });

  it('cancelling reset does not change settings', async () => {
    await renderSettingsScreen();

    const toggle = screen.getByLabelText('Detect headings');
    await userEvent.click(toggle);
    expect(toggle).toHaveAttribute('aria-checked', 'false');

    await userEvent.click(screen.getByText('Reset to defaults'));
    await userEvent.click(screen.getByText('Cancel'));
    expect(toggle).toHaveAttribute('aria-checked', 'false');
  });

  it('"Custom" page range shows From/To inputs', async () => {
    await renderSettingsScreen();
    expect(screen.queryByText('From')).not.toBeInTheDocument();
    const pageRangeSelect = screen.getByDisplayValue('All');
    await userEvent.selectOptions(pageRangeSelect, 'custom');
    expect(screen.getByText('From')).toBeInTheDocument();
    expect(screen.getByText('To')).toBeInTheDocument();
  });

  it('selecting "All" page range hides From/To inputs', async () => {
    await renderSettingsScreen();
    const pageRangeSelect = screen.getByDisplayValue('All');
    await userEvent.selectOptions(pageRangeSelect, 'custom');
    expect(screen.getByText('From')).toBeInTheDocument();
    await userEvent.selectOptions(pageRangeSelect, 'all');
    expect(screen.queryByText('From')).not.toBeInTheDocument();
  });

  it('disabling "Extract images" disables image sub-settings', async () => {
    await renderSettingsScreen();
    const extractToggle = screen.getByLabelText('Extract images');
    await userEvent.click(extractToggle);

    const qualitySelect = screen.getByDisplayValue('Medium');
    expect(qualitySelect).toBeDisabled();

    const webPToggle = screen.getByLabelText('Convert to WebP');
    expect(webPToggle).toBeDisabled();
  });

  it('enabling "Extract images" re-enables image sub-settings', async () => {
    await renderSettingsScreen();
    const extractToggle = screen.getByLabelText('Extract images');

    await userEvent.click(extractToggle);
    expect(screen.getByDisplayValue('Medium')).toBeDisabled();

    await userEvent.click(extractToggle);
    expect(screen.getByDisplayValue('Medium')).not.toBeDisabled();
  });

  it('enabling WebP with EPUB 2 selected auto-upgrades to EPUB 3 and shows toast', async () => {
    const { toast } = await import('sonner');
    await renderSettingsScreen();

    const epubSelect = screen.getByDisplayValue('EPUB 3');
    await userEvent.selectOptions(epubSelect, 'epub2');
    expect(epubSelect).toHaveValue('epub2');

    const webpToggle = screen.getByLabelText('Convert to WebP');
    await userEvent.click(webpToggle);

    expect(epubSelect).toHaveValue('epub3');
    expect(webpToggle).toHaveAttribute('aria-checked', 'true');
    expect(toast.warning).toHaveBeenCalledWith(
      expect.stringContaining('WebP images require EPUB 3'),
    );
  });

  it('switching to EPUB 2 with WebP enabled auto-disables WebP and shows toast', async () => {
    const { toast } = await import('sonner');
    await renderSettingsScreen();

    const webpToggle = screen.getByLabelText('Convert to WebP');
    await userEvent.click(webpToggle);
    expect(webpToggle).toHaveAttribute('aria-checked', 'true');

    const epubSelect = screen.getByDisplayValue('EPUB 3');
    await userEvent.selectOptions(epubSelect, 'epub2');

    expect(webpToggle).toHaveAttribute('aria-checked', 'false');
    expect(epubSelect).toHaveValue('epub2');
    expect(toast.warning).toHaveBeenCalledWith(
      expect.stringContaining('WebP images are not supported in EPUB 2'),
    );
  });

  it('save indicator appears briefly after changing a setting', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    await renderSettingsScreen();

    const toggle = screen.getByLabelText('Detect headings');
    await userEvent.click(toggle);

    await waitFor(() => {
      expect(screen.getByText((_, el) =>
        el?.getAttribute('class')?.includes('text-[var(--text-success)]') ?? false,
      )).toBeInTheDocument();
    });

    act(() => { vi.advanceTimersByTime(2000); });

    await waitFor(() => {
      expect(screen.queryByText((_, el) =>
        el?.getAttribute('class')?.includes('text-[var(--text-success)]') ?? false,
      )).not.toBeInTheDocument();
    });

    vi.useRealTimers();
  });

  it('max image width input has correct constraints', async () => {
    await renderSettingsScreen();
    const maxWidthInput = screen.getByDisplayValue('800');
    expect(maxWidthInput).toHaveAttribute('min', '200');
    expect(maxWidthInput).toHaveAttribute('max', '2000');
    expect(maxWidthInput).toHaveAttribute('step', '100');
  });

  it('heading level threshold input has correct constraints', async () => {
    await renderSettingsScreen();
    const thresholdInput = screen.getByDisplayValue('3');
    expect(thresholdInput).toHaveAttribute('min', '1');
    expect(thresholdInput).toHaveAttribute('max', '6');
  });

  it('renders all dropdown options for EPUB version', async () => {
    await renderSettingsScreen();
    const epubSelect = screen.getByDisplayValue('EPUB 3');
    const options = epubSelect.querySelectorAll('option');
    expect(options).toHaveLength(2);
    expect(options[0]).toHaveTextContent('EPUB 2');
    expect(options[1]).toHaveTextContent('EPUB 3');
  });

  it('renders all text alignment options', async () => {
    await renderSettingsScreen();
    const alignSelect = screen.getByDisplayValue('Justify');
    const options = alignSelect.querySelectorAll('option');
    expect(options).toHaveLength(3);
    expect(options[0]).toHaveTextContent('Justify');
    expect(options[1]).toHaveTextContent('Left');
    expect(options[2]).toHaveTextContent('Right');
  });

  it('renders all cover page options', async () => {
    await renderSettingsScreen();
    const coverSelect = screen.getByDisplayValue('Auto-detect');
    const options = coverSelect.querySelectorAll('option');
    expect(options).toHaveLength(3);
    expect(options[0]).toHaveTextContent('Auto-detect');
    expect(options[1]).toHaveTextContent('First page');
    expect(options[2]).toHaveTextContent('None');
  });

  it('renders unit labels for number inputs', async () => {
    await renderSettingsScreen();
    expect(screen.getByText('px')).toBeInTheDocument();
    expect(screen.getByText('pt')).toBeInTheDocument();
    expect(screen.getByText('em')).toBeInTheDocument();
  });
});
