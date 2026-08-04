import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
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

function renderSettingsScreen() {
  return render(
    <SettingsProvider>
      <SettingsScreen />
    </SettingsProvider>,
  );
}

describe('SettingsScreen', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders all four setting groups with correct headings', () => {
    renderSettingsScreen();
    expect(screen.getByText('Structure Detection')).toBeInTheDocument();
    expect(screen.getByText('Images')).toBeInTheDocument();
    expect(screen.getByText('Output Format')).toBeInTheDocument();
    expect(screen.getByText('Page Handling')).toBeInTheDocument();
  });

  it('renders the screen title', () => {
    renderSettingsScreen();
    expect(screen.getByText('Conversion settings')).toBeInTheDocument();
  });

  it('all toggle controls display their default values', () => {
    renderSettingsScreen();
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
    renderSettingsScreen();
    const toggle = screen.getByLabelText('Detect headings');
    expect(toggle).toHaveAttribute('aria-checked', 'true');
    await userEvent.click(toggle);
    expect(toggle).toHaveAttribute('aria-checked', 'false');
  });

  it('changing a dropdown updates its value', async () => {
    renderSettingsScreen();
    const epubSelect = screen.getByDisplayValue('EPUB 3');
    await userEvent.selectOptions(epubSelect, 'epub2');
    expect(epubSelect).toHaveValue('epub2');
  });

  it('changing a number input updates its value', () => {
    renderSettingsScreen();
    const fontSizeInput = screen.getByDisplayValue('12');
    fireEvent.change(fontSizeInput, { target: { value: '16' } });
    expect(fontSizeInput).toHaveValue(16);
  });

  it('"Reset to defaults" button shows confirmation dialog', async () => {
    renderSettingsScreen();
    await userEvent.click(screen.getByText('Reset to defaults'));
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(screen.getByText('Reset all settings to factory defaults? Per-document overrides are not affected.')).toBeInTheDocument();
  });

  it('confirming reset calls resetToDefaults', async () => {
    const { saveSettings } = await import('../../lib/settings');
    renderSettingsScreen();

    const toggle = screen.getByLabelText('Detect headings');
    await userEvent.click(toggle);
    expect(toggle).toHaveAttribute('aria-checked', 'false');

    await userEvent.click(screen.getByText('Reset to defaults'));
    await userEvent.click(screen.getByText('Confirm'));
    expect(toggle).toHaveAttribute('aria-checked', 'true');
    expect(saveSettings).toHaveBeenCalledWith(DEFAULT_SETTINGS);
  });

  it('cancelling reset does not change settings', async () => {
    renderSettingsScreen();

    const toggle = screen.getByLabelText('Detect headings');
    await userEvent.click(toggle);
    expect(toggle).toHaveAttribute('aria-checked', 'false');

    await userEvent.click(screen.getByText('Reset to defaults'));
    await userEvent.click(screen.getByText('Cancel'));
    expect(toggle).toHaveAttribute('aria-checked', 'false');
  });

  it('"Custom" page range shows From/To inputs', async () => {
    renderSettingsScreen();
    expect(screen.queryByText('From')).not.toBeInTheDocument();
    const pageRangeSelect = screen.getByDisplayValue('All');
    await userEvent.selectOptions(pageRangeSelect, 'custom');
    expect(screen.getByText('From')).toBeInTheDocument();
    expect(screen.getByText('To')).toBeInTheDocument();
  });

  it('selecting "All" page range hides From/To inputs', async () => {
    renderSettingsScreen();
    const pageRangeSelect = screen.getByDisplayValue('All');
    await userEvent.selectOptions(pageRangeSelect, 'custom');
    expect(screen.getByText('From')).toBeInTheDocument();
    await userEvent.selectOptions(pageRangeSelect, 'all');
    expect(screen.queryByText('From')).not.toBeInTheDocument();
  });

  it('disabling "Extract images" disables image sub-settings', async () => {
    renderSettingsScreen();
    const extractToggle = screen.getByLabelText('Extract images');
    await userEvent.click(extractToggle);

    const qualitySelect = screen.getByDisplayValue('Medium');
    expect(qualitySelect).toBeDisabled();

    const webPToggle = screen.getByLabelText('Convert to WebP');
    expect(webPToggle).toBeDisabled();
  });

  it('enabling "Extract images" re-enables image sub-settings', async () => {
    renderSettingsScreen();
    const extractToggle = screen.getByLabelText('Extract images');

    await userEvent.click(extractToggle);
    expect(screen.getByDisplayValue('Medium')).toBeDisabled();

    await userEvent.click(extractToggle);
    expect(screen.getByDisplayValue('Medium')).not.toBeDisabled();
  });
});
