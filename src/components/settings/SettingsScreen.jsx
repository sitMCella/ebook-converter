import { useState, useCallback } from 'react';
import { RotateCcw } from 'lucide-react';
import { useSettings } from '../../contexts/SettingsContext';
import { Button } from '../ui/Button';
import { Toggle } from '../ui/Toggle';
import { ConfirmDialog } from '../ui/ConfirmDialog';
import { SettingGroup } from './SettingGroup';
import { SettingRow } from './SettingRow';
import { SaveIndicator } from './SaveIndicator';

const LINE_HEIGHT_OPTIONS = ['1.0', '1.2', '1.5', '1.8', '2.0'];
const FONT_FAMILY_OPTIONS = [
  { value: 'default', label: 'Default' },
  { value: 'serif', label: 'Serif' },
  { value: 'sans-serif', label: 'Sans-serif' },
  { value: 'monospace', label: 'Monospace' },
];
const EPUB_VERSION_OPTIONS = [
  { value: 'epub2', label: 'EPUB 2' },
  { value: 'epub3', label: 'EPUB 3' },
];
const IMAGE_QUALITY_OPTIONS = [
  { value: 'high', label: 'High' },
  { value: 'medium', label: 'Medium' },
  { value: 'low', label: 'Low' },
];
const PAGE_RANGE_OPTIONS = [
  { value: 'all', label: 'All' },
  { value: 'custom', label: 'Custom' },
];
const TEXT_ALIGNMENT_OPTIONS = [
  { value: 'justify', label: 'Justify' },
  { value: 'left', label: 'Left' },
  { value: 'right', label: 'Right' },
];
const COVER_PAGE_OPTIONS = [
  { value: 'auto', label: 'Auto-detect' },
  { value: 'firstPage', label: 'First page' },
  { value: 'none', label: 'None' },
];

export function SettingsScreen() {
  const { settings, updateSetting, resetToDefaults } = useSettings();
  const [confirmResetOpen, setConfirmResetOpen] = useState(false);
  const [lastChanged, setLastChanged] = useState(null);

  const handleChange = useCallback((group, key, value) => {
    updateSetting(group, key, value);
    setLastChanged(`${group}.${key}`);
    setTimeout(() => setLastChanged(null), 1500);
  }, [updateSetting]);

  const handleReset = useCallback(async () => {
    await resetToDefaults();
    setConfirmResetOpen(false);
  }, [resetToDefaults]);

  const selectClass = 'text-[13px] px-2 py-1 rounded-[var(--radius)] border border-[var(--border)] bg-[var(--surface-0)] cursor-pointer focus:outline-none focus:ring-2 focus:ring-[var(--fill-accent)] text-[var(--text-primary)]';
  const numberClass = 'w-16 text-[13px] px-2 py-1 rounded-[var(--radius)] border border-[var(--border)] bg-[var(--surface-0)] focus:outline-none focus:ring-2 focus:ring-[var(--fill-accent)] text-[var(--text-primary)]';

  const imagesDisabled = !settings.images.extractImages;

  return (
    <div className="p-6 max-w-5xl">
      <div className="flex items-center justify-between mb-6">
        <h3 className="text-[18px] font-medium">Conversion settings</h3>
        <Button variant="secondary" onClick={() => setConfirmResetOpen(true)}>
          <RotateCcw size={14} />
          Reset to defaults
        </Button>
      </div>

      <div className="grid grid-cols-1 min-[1000px]:grid-cols-2 gap-6">
        {/* Left column */}
        <div className="flex flex-col gap-5">
          <SettingGroup title="Structure Detection">
            <SettingRow label="Detect headings">
              <SaveIndicator visible={lastChanged === 'structure.detectHeadings'} />
              <Toggle
                checked={settings.structure.detectHeadings}
                onChange={(v) => handleChange('structure', 'detectHeadings', v)}
                label="Detect headings"
              />
            </SettingRow>
            <SettingRow label="Detect footnotes">
              <SaveIndicator visible={lastChanged === 'structure.detectFootnotes'} />
              <Toggle
                checked={settings.structure.detectFootnotes}
                onChange={(v) => handleChange('structure', 'detectFootnotes', v)}
                label="Detect footnotes"
              />
            </SettingRow>
            <SettingRow label="Heading level threshold">
              <SaveIndicator visible={lastChanged === 'structure.headingLevelThreshold'} />
              <input
                type="number"
                value={settings.structure.headingLevelThreshold}
                min={1}
                max={6}
                onChange={(e) => handleChange('structure', 'headingLevelThreshold', Number(e.target.value))}
                className={numberClass}
              />
            </SettingRow>
            <SettingRow label="Paragraph detection">
              <SaveIndicator visible={lastChanged === 'structure.paragraphDetection'} />
              <Toggle
                checked={settings.structure.paragraphDetection}
                onChange={(v) => handleChange('structure', 'paragraphDetection', v)}
                label="Paragraph detection"
              />
            </SettingRow>
            <SettingRow label="List detection">
              <SaveIndicator visible={lastChanged === 'structure.listDetection'} />
              <Toggle
                checked={settings.structure.listDetection}
                onChange={(v) => handleChange('structure', 'listDetection', v)}
                label="List detection"
              />
            </SettingRow>
          </SettingGroup>

          <SettingGroup title="Images">
            <SettingRow label="Extract images">
              <SaveIndicator visible={lastChanged === 'images.extractImages'} />
              <Toggle
                checked={settings.images.extractImages}
                onChange={(v) => handleChange('images', 'extractImages', v)}
                label="Extract images"
              />
            </SettingRow>
            <SettingRow label="Image quality" disabled={imagesDisabled}>
              <SaveIndicator visible={lastChanged === 'images.imageQuality'} />
              <select
                value={settings.images.imageQuality}
                onChange={(e) => handleChange('images', 'imageQuality', e.target.value)}
                className={selectClass}
                disabled={imagesDisabled}
              >
                {IMAGE_QUALITY_OPTIONS.map((opt) => (
                  <option key={opt.value} value={opt.value}>{opt.label}</option>
                ))}
              </select>
            </SettingRow>
            <SettingRow label="Max image width" disabled={imagesDisabled}>
              <SaveIndicator visible={lastChanged === 'images.maxImageWidth'} />
              <input
                type="number"
                value={settings.images.maxImageWidth}
                min={200}
                max={2000}
                step={100}
                onChange={(e) => handleChange('images', 'maxImageWidth', Number(e.target.value))}
                className={numberClass}
                disabled={imagesDisabled}
              />
              <span className="text-[12px] text-[var(--text-muted)]">px</span>
            </SettingRow>
            <SettingRow label="Convert to WebP" disabled={imagesDisabled}>
              <SaveIndicator visible={lastChanged === 'images.convertToWebP'} />
              <Toggle
                checked={settings.images.convertToWebP}
                onChange={(v) => handleChange('images', 'convertToWebP', v)}
                label="Convert to WebP"
                disabled={imagesDisabled}
              />
            </SettingRow>
          </SettingGroup>
        </div>

        {/* Right column */}
        <div className="flex flex-col gap-5">
          <SettingGroup title="Output Format">
            <SettingRow label="EPUB version">
              <SaveIndicator visible={lastChanged === 'output.epubVersion'} />
              <select
                value={settings.output.epubVersion}
                onChange={(e) => handleChange('output', 'epubVersion', e.target.value)}
                className={selectClass}
              >
                {EPUB_VERSION_OPTIONS.map((opt) => (
                  <option key={opt.value} value={opt.value}>{opt.label}</option>
                ))}
              </select>
            </SettingRow>
            <SettingRow label="Embed fonts">
              <SaveIndicator visible={lastChanged === 'output.embedFonts'} />
              <Toggle
                checked={settings.output.embedFonts}
                onChange={(v) => handleChange('output', 'embedFonts', v)}
                label="Embed fonts"
              />
            </SettingRow>
            <SettingRow label="Font family">
              <SaveIndicator visible={lastChanged === 'output.fontFamily'} />
              <select
                value={settings.output.fontFamily}
                onChange={(e) => handleChange('output', 'fontFamily', e.target.value)}
                className={selectClass}
              >
                {FONT_FAMILY_OPTIONS.map((opt) => (
                  <option key={opt.value} value={opt.value}>{opt.label}</option>
                ))}
              </select>
            </SettingRow>
            <SettingRow label="Base font size">
              <SaveIndicator visible={lastChanged === 'output.baseFontSize'} />
              <input
                type="number"
                value={settings.output.baseFontSize}
                min={8}
                max={24}
                onChange={(e) => handleChange('output', 'baseFontSize', Number(e.target.value))}
                className={numberClass}
              />
              <span className="text-[12px] text-[var(--text-muted)]">pt</span>
            </SettingRow>
            <SettingRow label="Line height">
              <SaveIndicator visible={lastChanged === 'output.lineHeight'} />
              <select
                value={settings.output.lineHeight}
                onChange={(e) => handleChange('output', 'lineHeight', parseFloat(e.target.value))}
                className={selectClass}
              >
                {LINE_HEIGHT_OPTIONS.map((v) => (
                  <option key={v} value={v}>{v}</option>
                ))}
              </select>
            </SettingRow>
            <SettingRow label="Margins">
              <SaveIndicator visible={lastChanged === 'output.margins'} />
              <input
                type="number"
                value={settings.output.margins}
                min={0.5}
                max={3.0}
                step={0.5}
                onChange={(e) => handleChange('output', 'margins', parseFloat(e.target.value))}
                className={numberClass}
              />
              <span className="text-[12px] text-[var(--text-muted)]">em</span>
            </SettingRow>
            <SettingRow label="Text alignment">
              <SaveIndicator visible={lastChanged === 'output.textAlignment'} />
              <select
                value={settings.output.textAlignment}
                onChange={(e) => handleChange('output', 'textAlignment', e.target.value)}
                className={selectClass}
              >
                {TEXT_ALIGNMENT_OPTIONS.map((opt) => (
                  <option key={opt.value} value={opt.value}>{opt.label}</option>
                ))}
              </select>
            </SettingRow>
          </SettingGroup>

          <SettingGroup title="Page Handling">
            <SettingRow label="Skip blank pages">
              <SaveIndicator visible={lastChanged === 'pageHandling.skipBlankPages'} />
              <Toggle
                checked={settings.pageHandling.skipBlankPages}
                onChange={(v) => handleChange('pageHandling', 'skipBlankPages', v)}
                label="Skip blank pages"
              />
            </SettingRow>
            <SettingRow label="Page range">
              <SaveIndicator visible={lastChanged === 'pageHandling.pageRange'} />
              <select
                value={settings.pageHandling.pageRange}
                onChange={(e) => handleChange('pageHandling', 'pageRange', e.target.value)}
                className={selectClass}
              >
                {PAGE_RANGE_OPTIONS.map((opt) => (
                  <option key={opt.value} value={opt.value}>{opt.label}</option>
                ))}
              </select>
            </SettingRow>
            {settings.pageHandling.pageRange === 'custom' && (
              <SettingRow label="Page range">
                <div className="flex items-center gap-2">
                  <label className="text-[12px] text-[var(--text-muted)]">From</label>
                  <input
                    type="number"
                    value={settings.pageHandling.pageRangeFrom ?? ''}
                    min={1}
                    onChange={(e) => handleChange('pageHandling', 'pageRangeFrom', e.target.value ? Number(e.target.value) : null)}
                    className={numberClass}
                  />
                  <label className="text-[12px] text-[var(--text-muted)]">To</label>
                  <input
                    type="number"
                    value={settings.pageHandling.pageRangeTo ?? ''}
                    min={1}
                    onChange={(e) => handleChange('pageHandling', 'pageRangeTo', e.target.value ? Number(e.target.value) : null)}
                    className={numberClass}
                  />
                </div>
              </SettingRow>
            )}
            <SettingRow label="Keep page breaks">
              <SaveIndicator visible={lastChanged === 'pageHandling.keepPageBreaks'} />
              <Toggle
                checked={settings.pageHandling.keepPageBreaks}
                onChange={(v) => handleChange('pageHandling', 'keepPageBreaks', v)}
                label="Keep page breaks"
              />
            </SettingRow>
            <SettingRow label="Remove page numbers">
              <SaveIndicator visible={lastChanged === 'pageHandling.removePageNumbers'} />
              <Toggle
                checked={settings.pageHandling.removePageNumbers}
                onChange={(v) => handleChange('pageHandling', 'removePageNumbers', v)}
                label="Remove page numbers"
              />
            </SettingRow>
            <SettingRow label="Cover page">
              <SaveIndicator visible={lastChanged === 'pageHandling.coverPage'} />
              <select
                value={settings.pageHandling.coverPage}
                onChange={(e) => handleChange('pageHandling', 'coverPage', e.target.value)}
                className={selectClass}
              >
                {COVER_PAGE_OPTIONS.map((opt) => (
                  <option key={opt.value} value={opt.value}>{opt.label}</option>
                ))}
              </select>
            </SettingRow>
          </SettingGroup>
        </div>
      </div>

      <ConfirmDialog
        open={confirmResetOpen}
        title="Reset settings"
        message="Reset all settings to factory defaults? Per-document overrides are not affected."
        onConfirm={handleReset}
        onCancel={() => setConfirmResetOpen(false)}
      />
    </div>
  );
}
