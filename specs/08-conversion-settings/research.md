# 08 — Conversion Settings: Research

## Existing Infrastructure

### Settings Module (`src/lib/settings.js`)

The settings infrastructure is already implemented:

- `DEFAULT_SETTINGS` — hardcoded default values for all setting groups (structure, images, output, pageHandling, outputLocation).
- `loadSettings()` — reads `settings.json` from the Tauri app data directory; returns defaults if the file is missing or unreadable. In browser mode, always returns defaults.
- `saveSettings(settings)` — writes the full settings object as formatted JSON to `settings.json`. No-op in browser mode.
- `mergeSettings(base, overrides)` — shallow merge per group (one level deep); used for both settings persistence and per-document override resolution.
- `getEffectiveSettings(global, overrides)` — convenience wrapper over `mergeSettings`.
- `settingsToConversionOptions(settings, { outputFolder, bookId })` — maps settings into the `ConversionOptions` shape expected by the Rust `convert_pdf` command.

The module already handles the happy path for load/save/merge. The settings screen needs to call `saveSettings()` on each change and provide a reactive state layer on top.

### Per-Document Overrides (`src/components/library/ConversionOptions.jsx`)

The Library detail panel already implements per-document overrides with:
- `SelectControl` and `NumberControl` reusable controls.
- Override/default state indication (muted "(default)" label vs. primary text + reset button).
- `OverrideRow` layout component.
- `countOverrides()` utility.
- Integration with `ImportContext` via `SET_DOCUMENT_OVERRIDES` action.

The existing controls call `loadSettings()` directly on mount. With a `SettingsContext`, they should read from context instead — avoiding redundant async calls and staying in sync with settings changes.

### Current Route

`App.jsx` has a `/settings` route pointing to a `<Placeholder title="Settings" />` component. The route and sidebar navigation are already wired.

---

## Settings Screen Design Patterns in Desktop Apps

### Instant Save vs. Save Button

Modern desktop apps (VS Code, Figma, macOS System Settings, Slack preferences) overwhelmingly use instant save — changes take effect immediately with no explicit save button. This matches the design in spec 02.

**Benefits**: fewer clicks, no "forgot to save" mistakes, simpler UI.
**Risk**: accidental changes. Mitigated by the "Reset to defaults" button and by the fact that most settings have safe defaults.

### Debouncing for Continuous Controls

Number inputs and sliders can fire many change events quickly. Best practice is to debounce the save operation:

```javascript
const debouncedSave = useMemo(
  () => debounce((settings) => saveSettings(settings), 300),
  []
);
```

This avoids hammering the filesystem on every keystroke in a number input.

### Save Indicator

A brief visual confirmation that the save succeeded, since there's no save button to provide implicit feedback. The spec 02 design calls for a "small check icon that fades after 1 s". Implementation:

```jsx
// CSS transition
.save-indicator {
  opacity: 1;
  transition: opacity 0.5s ease 0.5s; /* delay 0.5s, then fade 0.5s */
}
.save-indicator.hidden {
  opacity: 0;
}
```

---

## React Context for Global Settings

### Why a Context

The global settings are needed in multiple places:
1. **Settings screen** — read and write all settings.
2. **ConversionOptions** (Library) — read global defaults to show inherited values.
3. **useConversion hook** — read settings to compute effective options before calling `convert_pdf`.
4. **ConvertingScreen** — display which settings were used.

Currently, each consumer calls `loadSettings()` independently, which:
- Introduces async waterfalls (each consumer waits for the file read).
- Can produce stale reads if settings change between calls.
- Duplicates error handling.

A `SettingsContext` loads settings once on app startup and provides them synchronously to all consumers.

### Context Design

```jsx
const SettingsContext = createContext();

function SettingsProvider({ children }) {
  const [settings, setSettings] = useState(DEFAULT_SETTINGS);
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    loadSettings().then((s) => {
      setSettings(s);
      setLoaded(true);
    });
  }, []);

  const updateSetting = useCallback(async (group, key, value) => {
    setSettings(prev => {
      const next = {
        ...prev,
        [group]: { ...prev[group], [key]: value }
      };
      debouncedSave(next);
      return next;
    });
  }, []);

  const resetToDefaults = useCallback(async () => {
    setSettings({ ...DEFAULT_SETTINGS });
    await saveSettings(DEFAULT_SETTINGS);
  }, []);

  return (
    <SettingsContext.Provider value={{ settings, loaded, updateSetting, resetToDefaults }}>
      {children}
    </SettingsContext.Provider>
  );
}
```

### Provider Placement

`SettingsProvider` wraps the app at the top level in `App.jsx`, outside `BrowserRouter`:

```jsx
<SettingsProvider>
  <ImportProvider>
    <ConversionProvider>
      <BrowserRouter>...</BrowserRouter>
    </ConversionProvider>
  </ImportProvider>
</SettingsProvider>
```

---

## Cross-Setting Validation Patterns

### WebP ↔ EPUB Version Dependency

WebP image support requires EPUB 3. Two conflict scenarios exist:

1. **Enable WebP with EPUB 2 selected** → auto-upgrade to EPUB 3 + toast.
2. **Downgrade to EPUB 2 with WebP enabled** → auto-disable WebP + toast.

These are implemented as side effects in `updateSetting`:

```javascript
function updateSetting(group, key, value) {
  setSettings(prev => {
    const next = { ...prev, [group]: { ...prev[group], [key]: value } };

    // WebP → EPUB 3 dependency
    if (group === 'images' && key === 'convertToWebP' && value === true) {
      if (next.output.epubVersion === 'epub2') {
        next.output = { ...next.output, epubVersion: 'epub3' };
        toast.warning('WebP images require EPUB 3. The EPUB version has been updated.');
      }
    }

    if (group === 'output' && key === 'epubVersion' && value === 'epub2') {
      if (next.images.convertToWebP) {
        next.images = { ...next.images, convertToWebP: false };
        toast.warning('WebP images are not supported in EPUB 2. Image conversion has been disabled.');
      }
    }

    debouncedSave(next);
    return next;
  });
}
```

### Disabled State for Dependent Settings

When "Extract images" is OFF, image-related sub-settings (quality, max width, WebP) become non-interactive. This is a UI-only concern — the controls render in a disabled state with reduced opacity. The underlying setting values are preserved (not reset) so re-enabling "Extract images" restores previous choices.

---

## Toggle Switch Component

React does not have a built-in toggle switch. The project uses Tailwind CSS for styling, so a custom component is the cleanest approach (no additional dependency):

```jsx
function Toggle({ checked, onChange, label, disabled }) {
  return (
    <button
      role="switch"
      aria-checked={checked}
      aria-label={label}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={`relative w-9 h-5 rounded-full transition-colors duration-150
        ${checked ? 'bg-[var(--fill-accent)]' : 'bg-[var(--surface-2)]'}
        ${disabled ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}
      `}
    >
      <span
        className={`absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white
          transition-transform duration-150
          ${checked ? 'translate-x-4' : 'translate-x-0'}
        `}
      />
    </button>
  );
}
```

The `role="switch"` and `aria-checked` attributes make it accessible. The `Space` key toggles it when focused (native button behaviour).

---

## Additional Settings Research

### Keep Page Breaks

PDF page breaks are captured by `pdf-extract` as form-feed characters (`\x0C`). The current pipeline (spec 04) uses these to split text into per-page arrays, then structure detection and chapter splitting happen across merged text.

To preserve page breaks within chapters, the structure detector can emit a `PageBreak` variant in `StructuredContent`, and the EPUB generator can render it as a CSS `page-break-before: always` on a `<div>` or `<hr>` element. This is a common EPUB pattern for preserving source document structure.

**Default OFF**: most users want reflowable content without artificial breaks. Useful for academic/legal documents where page boundaries matter.

### Remove Page Numbers

PDF text extraction frequently captures page numbers from headers/footers. These appear as isolated short lines (usually just a number) at the start or end of each page's extracted text. Detection heuristics:

- Lines matching `^\s*\d+\s*$` (standalone number) at the first or last line of a page.
- Lines matching common patterns: `- N -`, `Page N`, `N of M`.

**Default ON**: page numbers in reflowable EPUB are meaningless and distracting. Power users working with scanned documents may want them preserved for reference.

### Cover Page

EPUB readers display a cover image in the library view. Three strategies:

1. **Auto-detect**: check if page 1 contains a single full-page image with minimal/no text. If so, extract it as the cover. Otherwise, fall back to no cover.
2. **First page**: render page 1 as an image (requires a rendering engine — in v1, extract the largest image from page 1, or skip if no images).
3. **None**: no cover image in the EPUB metadata.

The `epub-builder` crate supports cover images via:
```rust
builder.add_cover_image("cover.jpg", cover_bytes, "image/jpeg")?;
```

### Text Alignment

EPUB CSS `text-align` is a common preference. The current CSS in spec 04's research uses `text-align: justify`. Some users prefer left-aligned text (avoids uneven word spacing, better for narrow e-reader screens). Adding this as a setting gives users control without modifying the CSS manually.

---

## Reusable Controls Strategy

The Settings screen and the ConversionOptions component share the same control patterns (toggle, dropdown, number input). The following controls should be extracted to `src/components/ui/`:

| Control | Props | Used In |
|---|---|---|
| `Toggle` | `checked`, `onChange`, `label`, `disabled` | Settings screen, ConversionOptions (future "More options") |
| `SettingRow` | `label`, `description?`, `children` | Settings screen |
| `SettingGroup` | `title`, `children` | Settings screen |

The existing `SelectControl`, `NumberControl`, and `OverrideRow` in `ConversionOptions.jsx` are currently colocated. The `SelectControl` and `NumberControl` should remain in `ConversionOptions` since they have override-specific logic (default labels, reset buttons). The `Toggle`, `SettingRow`, and `SettingGroup` are new components for the settings screen.

---

## References

- [Spec 02 — Screen 5: Settings](../02-ui-ux-design/ui_ux_design.md)
- [Spec 04 — Settings Integration (FR-10)](../04-convert-pdf/spec.md)
- [Spec 05 — Output Location Setting Removed](../05-storage-location/spec.md)
- [Existing settings module](../../src/lib/settings.js)
- [Existing ConversionOptions component](../../src/components/library/ConversionOptions.jsx)
- [WAI-ARIA Switch Role](https://www.w3.org/WAI/ARIA/apg/patterns/switch/)
- [epub-builder cover image API](https://docs.rs/epub-builder/)
