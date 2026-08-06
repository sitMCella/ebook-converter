import { useState } from 'react';
import { ChevronRight, ChevronDown, X } from 'lucide-react';
import { useImportContext } from '../../contexts/ImportContext';
import { useSettings } from '../../contexts/SettingsContext';
import { saveBookMetadata } from '../../lib/tauri';

const QUALITY_OPTIONS = [
  { value: 'high', label: 'High' },
  { value: 'medium', label: 'Medium' },
  { value: 'low', label: 'Low' },
];

const PAGE_RANGE_OPTIONS = [
  { value: 'all', label: 'All' },
  { value: 'custom', label: 'Custom' },
];

const COVER_PAGE_OPTIONS = [
  { value: 'auto', label: 'Auto-detect' },
  { value: 'firstPage', label: 'First page' },
  { value: 'none', label: 'None' },
];

function OverrideRow({ label, children }) {
  return (
    <div className="flex items-center justify-between gap-3 py-1.5">
      <span className="text-[13px] text-[var(--text-secondary)]">{label}</span>
      <div className="flex items-center gap-1.5">{children}</div>
    </div>
  );
}

function SelectControl({ value, defaultValue, options, onChange, onReset }) {
  const isOverridden = value !== undefined;
  const displayValue = isOverridden ? value : defaultValue;

  return (
    <>
      <select
        value={displayValue}
        onChange={(e) => onChange(e.target.value)}
        className={`text-[13px] px-2 py-1 rounded-[var(--radius)] border border-[var(--border)] bg-[var(--surface-0)] cursor-pointer focus:outline-none focus:ring-2 focus:ring-[var(--fill-accent)] ${
          isOverridden
            ? 'text-[var(--text-primary)]'
            : 'text-[var(--text-muted)]'
        }`}
      >
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
            {!isOverridden && opt.value === defaultValue ? ' (default)' : ''}
          </option>
        ))}
      </select>
      {isOverridden && (
        <button
          type="button"
          onClick={onReset}
          className="p-0.5 rounded hover:bg-[var(--surface-2)] text-[var(--text-muted)] cursor-pointer"
          aria-label={`Reset to default`}
        >
          <X size={14} />
        </button>
      )}
    </>
  );
}

function NumberControl({
  value,
  defaultValue,
  min,
  max,
  step = 1,
  onChange,
  onReset,
  suffix = '',
}) {
  const isOverridden = value !== undefined;
  const displayValue = isOverridden ? value : defaultValue;

  return (
    <>
      <div className="flex items-center gap-1">
        <input
          type="number"
          value={displayValue}
          min={min}
          max={max}
          step={step}
          onChange={(e) => onChange(Number(e.target.value))}
          className={`w-16 text-[13px] px-2 py-1 rounded-[var(--radius)] border border-[var(--border)] bg-[var(--surface-0)] focus:outline-none focus:ring-2 focus:ring-[var(--fill-accent)] ${
            isOverridden
              ? 'text-[var(--text-primary)]'
              : 'text-[var(--text-muted)]'
          }`}
        />
        {suffix && (
          <span className="text-[12px] text-[var(--text-muted)]">{suffix}</span>
        )}
        {!isOverridden && (
          <span className="text-[12px] text-[var(--text-muted)]">
            (default)
          </span>
        )}
      </div>
      {isOverridden && (
        <button
          type="button"
          onClick={onReset}
          className="p-0.5 rounded hover:bg-[var(--surface-2)] text-[var(--text-muted)] cursor-pointer"
          aria-label={`Reset to default`}
        >
          <X size={14} />
        </button>
      )}
    </>
  );
}

export function ConversionOptions({ file }) {
  const { dispatch } = useImportContext();
  const [expanded, setExpanded] = useState(false);
  const { settings: globalSettings } = useSettings();

  const overrides = file.overrides || {};

  const overrideCount = countOverrides(overrides);

  function persistOverrides(newOverrides) {
    dispatch({
      type: 'SET_DOCUMENT_OVERRIDES',
      path: file.path,
      overrides: newOverrides,
    });
    if (file.bookId) {
      saveBookMetadata({
        bookId: file.bookId,
        storedPdfPath: file.storedPdfPath,
        originalPath: file.path,
        originalName: file.name,
        fileSize: file.metadata?.fileSize || file.size || 0,
        title: file.metadata?.title || null,
        author: file.metadata?.author || null,
        pageCount: file.metadata?.pageCount || 0,
        pdfVersion: file.metadata?.pdfVersion || null,
        createdDate: file.metadata?.createdDate || null,
        modifiedDate: file.metadata?.modifiedDate || null,
        producer: file.metadata?.producer || null,
        status: file.status || 'ready',
        outputPath: file.outputPath || null,
        images: file.conversionResult?.images ?? null,
        epubFileSize: file.conversionResult?.fileSize ?? null,
        conversionSettings: newOverrides || null,
      }).catch(() => {});
    }
  }

  function setOverride(group, key, value) {
    const newOverrides = {
      ...overrides,
      [group]: { ...overrides[group], [key]: value },
    };
    persistOverrides(newOverrides);
  }

  function resetOverride(group, key) {
    const newGroup = { ...overrides[group] };
    delete newGroup[key];
    const newOverrides = { ...overrides };
    if (Object.keys(newGroup).length === 0) {
      delete newOverrides[group];
    } else {
      newOverrides[group] = newGroup;
    }
    const finalOverrides = Object.keys(newOverrides).length > 0 ? newOverrides : undefined;
    persistOverrides(finalOverrides);
  }

  return (
    <div>
      <button
        type="button"
        onClick={() => setExpanded(!expanded)}
        className="flex items-center gap-1.5 w-full text-left text-[14px] font-medium py-2 cursor-pointer bg-transparent border-none text-[var(--text-primary)]"
      >
        {expanded ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
        Conversion options
        {overrideCount > 0 && (
          <span className="text-[12px] font-normal text-[var(--text-accent)] ml-1">
            · {overrideCount} custom
          </span>
        )}
      </button>

      {expanded && (
        <div className="pl-6 pb-2 flex flex-col gap-1">
          <OverrideRow label="Heading level threshold">
            <NumberControl
              value={overrides.structure?.headingLevelThreshold}
              defaultValue={globalSettings.structure.headingLevelThreshold}
              min={1}
              max={6}
              onChange={(v) =>
                setOverride('structure', 'headingLevelThreshold', v)
              }
              onReset={() =>
                resetOverride('structure', 'headingLevelThreshold')
              }
            />
          </OverrideRow>

          <OverrideRow label="Base font size">
            <NumberControl
              value={overrides.output?.baseFontSize}
              defaultValue={globalSettings.output.baseFontSize}
              min={8}
              max={24}
              suffix="pt"
              onChange={(v) => setOverride('output', 'baseFontSize', v)}
              onReset={() => resetOverride('output', 'baseFontSize')}
            />
          </OverrideRow>

          <OverrideRow label="Image quality">
            <SelectControl
              value={overrides.images?.imageQuality}
              defaultValue={globalSettings.images.imageQuality}
              options={QUALITY_OPTIONS}
              onChange={(v) => setOverride('images', 'imageQuality', v)}
              onReset={() => resetOverride('images', 'imageQuality')}
            />
          </OverrideRow>

          <OverrideRow label="Page range">
            <SelectControl
              value={overrides.pageHandling?.pageRange}
              defaultValue={globalSettings.pageHandling.pageRange}
              options={PAGE_RANGE_OPTIONS}
              onChange={(v) => setOverride('pageHandling', 'pageRange', v)}
              onReset={() => resetOverride('pageHandling', 'pageRange')}
            />
          </OverrideRow>

          <OverrideRow label="Cover page">
            <SelectControl
              value={overrides.pageHandling?.coverPage}
              defaultValue={globalSettings.pageHandling.coverPage}
              options={COVER_PAGE_OPTIONS}
              onChange={(v) => setOverride('pageHandling', 'coverPage', v)}
              onReset={() => resetOverride('pageHandling', 'coverPage')}
            />
          </OverrideRow>
        </div>
      )}
    </div>
  );
}

function countOverrides(overrides) {
  let count = 0;
  for (const group of Object.values(overrides)) {
    if (group && typeof group === 'object') {
      count += Object.keys(group).length;
    }
  }
  return count;
}
