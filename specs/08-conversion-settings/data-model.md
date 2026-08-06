# 08 — Conversion Settings: Data Model

## Extended Settings Schema

The existing `DEFAULT_SETTINGS` in `src/lib/settings.js` is extended with three new fields across two groups.

### New Fields

#### `pageHandling.keepPageBreaks` (new)

```typescript
keepPageBreaks: boolean  // default: false
```

Preserve original PDF page breaks as CSS `page-break-before: always` markers in the EPUB output. When `false` (default), content flows continuously.

#### `pageHandling.removePageNumbers` (new)

```typescript
removePageNumbers: boolean  // default: true
```

Strip page number artifacts from extracted text. Detected patterns: standalone numbers (`^\s*\d+\s*$`), `- N -`, `Page N`, `N of M` at the start or end of each page's text.

#### `pageHandling.coverPage` (new)

```typescript
coverPage: "auto" | "firstPage" | "none"  // default: "auto"
```

How to generate the EPUB cover image. `"auto"` looks for a full-page image on page 1; `"firstPage"` extracts the largest image from page 1 as the cover; `"none"` omits a cover entirely.

#### `output.textAlignment` (new)

```typescript
textAlignment: "justify" | "left" | "right"  // default: "justify"
```

CSS `text-align` for body paragraphs in the EPUB stylesheet.

### Updated `DEFAULT_SETTINGS`

```javascript
export const DEFAULT_SETTINGS = {
  structure: {
    detectHeadings: true,
    detectFootnotes: false,
    headingLevelThreshold: 3,
    paragraphDetection: true,
    listDetection: true,
  },
  images: {
    extractImages: true,
    imageQuality: 'medium',
    maxImageWidth: 800,
    convertToWebP: false,
  },
  output: {
    epubVersion: 'epub3',
    embedFonts: false,
    fontFamily: 'default',
    baseFontSize: 12,
    lineHeight: 1.5,
    margins: 1.0,
    textAlignment: 'justify',        // NEW
  },
  pageHandling: {
    skipBlankPages: true,
    pageRange: 'all',
    pageRangeFrom: null,
    pageRangeTo: null,
    keepPageBreaks: false,            // NEW
    removePageNumbers: true,          // NEW
    coverPage: 'auto',               // NEW
  },
};
```

---

## Settings File Schema

**File path**: `{app_data_dir}/settings.json`

The persisted JSON mirrors `DEFAULT_SETTINGS`. Missing keys are filled with defaults on load (forward-compatible). The `outputLocation` group is retained in the schema for backward compatibility but is ignored at runtime (spec 05).

```json
{
  "structure": {
    "detectHeadings": true,
    "detectFootnotes": false,
    "headingLevelThreshold": 3,
    "paragraphDetection": true,
    "listDetection": true
  },
  "images": {
    "extractImages": true,
    "imageQuality": "medium",
    "maxImageWidth": 800,
    "convertToWebP": false
  },
  "output": {
    "epubVersion": "epub3",
    "embedFonts": false,
    "fontFamily": "default",
    "baseFontSize": 12,
    "lineHeight": 1.5,
    "margins": 1.0,
    "textAlignment": "justify"
  },
  "pageHandling": {
    "skipBlankPages": true,
    "pageRange": "all",
    "pageRangeFrom": null,
    "pageRangeTo": null,
    "keepPageBreaks": false,
    "removePageNumbers": true,
    "coverPage": "auto"
  }
}
```

---

## SettingsContext

### Context Shape

```typescript
interface SettingsContextValue {
  /** Current global settings (always populated — defaults used until load completes). */
  settings: Settings;

  /** Whether settings have been loaded from disk. */
  loaded: boolean;

  /**
   * Update a single setting value. Persists immediately (debounced).
   * Handles cross-setting validation (WebP ↔ EPUB version).
   */
  updateSetting: (group: string, key: string, value: any) => void;

  /**
   * Reset all settings to DEFAULT_SETTINGS. Persists immediately.
   */
  resetToDefaults: () => Promise<void>;
}
```

### Context Provider

```jsx
<SettingsProvider>
  <ImportProvider>
    <ConversionProvider>
      <BrowserRouter>...</BrowserRouter>
    </ConversionProvider>
  </ImportProvider>
</SettingsProvider>
```

The `SettingsProvider` wraps the entire app at the outermost level. It loads settings from disk on mount and makes them available to all descendants.

### Hook

```javascript
export function useSettings() {
  const context = useContext(SettingsContext);
  if (!context) {
    throw new Error('useSettings must be used within a SettingsProvider');
  }
  return context;
}
```

---

## Rust Backend Extensions

### Extended `PageHandlingOptions`

Three new fields are added to the Rust `PageHandlingOptions` struct to accept the new settings from the frontend:

```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PageHandlingOptions {
    skip_blank_pages: bool,
    page_range: String,
    page_range_from: Option<u32>,
    page_range_to: Option<u32>,
    keep_page_breaks: bool,          // NEW
    remove_page_numbers: bool,       // NEW
    cover_page: String,              // NEW: "auto" | "firstPage" | "none"
}
```

### Extended `OutputOptions`

One new field for text alignment:

```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OutputOptions {
    epub_version: String,
    embed_fonts: bool,
    font_family: String,
    base_font_size: u8,
    line_height: f32,
    margins: f32,
    text_alignment: String,          // NEW: "justify" | "left" | "right"
}
```

### Extended `settingsToConversionOptions`

The frontend `settingsToConversionOptions()` function already passes through all setting groups. The new fields are included automatically since it spreads `settings.pageHandling` and `settings.output`.

---

## Component Interfaces

### SettingsScreen

```typescript
// No props — reads from SettingsContext
function SettingsScreen(): JSX.Element
```

### Toggle

```typescript
interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: string;            // aria-label
  disabled?: boolean;
}
```

### SettingRow

```typescript
interface SettingRowProps {
  label: string;             // Visible label text
  children: React.ReactNode; // Control element (toggle, select, number input)
  disabled?: boolean;        // Greyed out state
}
```

### SettingGroup

```typescript
interface SettingGroupProps {
  title: string;             // Group heading (h4)
  children: React.ReactNode; // Setting rows
}
```

### SaveIndicator

```typescript
interface SaveIndicatorProps {
  visible: boolean;          // Triggers fade-in; auto-fades out after 1s
}
```

---

## Impact on Existing Modules

### `src/lib/settings.js`

- `DEFAULT_SETTINGS`: add `textAlignment` to `output`, add `keepPageBreaks`, `removePageNumbers`, `coverPage` to `pageHandling`.
- Remove `outputLocation` from `DEFAULT_SETTINGS` (it was removed by spec 05 but may still be present in the code for backward compat — verify and clean up).
- No other changes needed. `loadSettings`, `saveSettings`, `mergeSettings`, `getEffectiveSettings`, and `settingsToConversionOptions` work as-is with the expanded schema.

### `src/components/library/ConversionOptions.jsx`

- Replace `loadSettings()` call with `useSettings()` hook to read global defaults from context.
- No structural changes. The component's override logic remains the same.

### `src/App.jsx`

- Replace `<Placeholder title="Settings" />` with `<SettingsScreen />`.
- Wrap the app with `<SettingsProvider>`.

### `src/hooks/useConversion.js`

- Replace `loadSettings()` call with `useSettings()` hook for immediate access.
- No other changes.
