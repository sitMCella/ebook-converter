# 08 — Conversion Settings: Plan

## Architectural Approach

The settings screen is a frontend-only feature built on top of the existing settings infrastructure (`src/lib/settings.js`). The main additions are: a `SettingsContext` for reactive global state, a `SettingsScreen` component implementing the spec 02 layout, reusable UI controls (Toggle, SettingGroup), and cross-setting validation logic.

```
┌─────────────────────────────────────────────────────┐
│                     App.jsx                          │
│  ┌───────────────────────────────────────────────┐   │
│  │            SettingsProvider                    │   │
│  │  ┌─────────────────────────────────────────┐   │   │
│  │  │  settings state (loaded from disk)      │   │   │
│  │  │  updateSetting()  → validate → save     │   │   │
│  │  │  resetToDefaults() → save               │   │   │
│  │  └──────────────┬──────────────────────────┘   │   │
│  │                 │  useSettings() hook           │   │
│  │  ┌──────────────▼──────────────────────────┐   │   │
│  │  │  Consumers                               │   │   │
│  │  │  ┌──────────────┐  ┌──────────────────┐  │   │   │
│  │  │  │SettingsScreen│  │ConversionOptions │  │   │   │
│  │  │  │(read + write)│  │(read only)       │  │   │   │
│  │  │  └──────────────┘  └──────────────────┘  │   │   │
│  │  │  ┌──────────────┐                        │   │   │
│  │  │  │useConversion │                        │   │   │
│  │  │  │(read only)   │                        │   │   │
│  │  │  └──────────────┘                        │   │   │
│  │  └──────────────────────────────────────────┘   │   │
│  └───────────────────────────────────────────────┘   │
│                                                       │
│  ┌───────────────────────────────────────────────┐   │
│  │  src/lib/settings.js (unchanged)              │   │
│  │  loadSettings() → reads settings.json         │   │
│  │  saveSettings() → writes settings.json        │   │
│  └───────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

## Key Decisions

### D1: SettingsContext over Direct File Reads

Introduce a React Context rather than having each consumer call `loadSettings()` independently. This ensures:
- Settings are loaded once on app start, not on each screen visit.
- All consumers see the same state — no stale reads.
- Changes from the Settings screen are immediately visible in ConversionOptions and useConversion.
- No async waterfalls in render paths.

### D2: Debounced Persistence

Setting changes update React state immediately (for responsive UI) but debounce the disk write with a 300 ms window. This handles rapid changes to number inputs without hammering the filesystem. The debounce fires the save with the latest accumulated state.

### D3: Cross-Setting Validation in Context

Cross-setting validation (WebP ↔ EPUB version, image controls disabled state) lives in the `updateSetting` function within `SettingsContext`, not in the screen component. This ensures validation applies regardless of where settings are updated (Settings screen, per-document overrides, or future preset profiles).

### D4: New Settings Added to Schema

Four new settings are added based on user needs for EPUB conversion quality:

| Setting | Group | Why |
|---|---|---|
| `keepPageBreaks` | pageHandling | Preserves source PDF page structure in the EPUB — essential for academic/legal documents where page numbers are referenced. |
| `removePageNumbers` | pageHandling | PDF text extraction captures header/footer page numbers that are meaningless in reflowable EPUB. On by default for cleaner output. |
| `coverPage` | pageHandling | EPUB readers display covers in their library view. Auto-detecting a cover from page 1 produces a more polished reading experience. |
| `textAlignment` | output | Body text alignment is a strong user preference. Justify is the default (traditional), but left alignment avoids uneven spacing on narrow screens. |

These additions are backward-compatible: `mergeSettings()` fills missing keys with defaults when reading an older `settings.json`.

### D5: Toggle Component as Shared UI

The toggle switch is a new reusable component in `src/components/ui/Toggle.jsx`. It uses `role="switch"` + `aria-checked` for accessibility. The existing ConversionOptions controls (`SelectControl`, `NumberControl`) stay colocated in that component since they have override-specific UI logic (default labels, reset buttons).

### D6: Single Column Fallback

On narrow windows (< 1000 px), the two-column grid collapses to a single column. This uses a CSS media query or container query on the grid: `grid-template-columns: repeat(auto-fit, minmax(400px, 1fr))`. No JavaScript is needed.

### D7: Settings Screen Does Not Affect Rust Backend Directly

The new frontend settings are passed through the existing `settingsToConversionOptions()` pipeline to the Rust `convert_pdf` command. The Rust structs need new fields (`keepPageBreaks`, `removePageNumbers`, `coverPage`, `textAlignment`), and the conversion pipeline modules need to handle them — but that implementation is part of spec 04's pipeline work. This spec only defines the schema and UI; the backend changes are documented as known integration points.

## Directory Structure Changes

```
src/
├── components/
│   ├── settings/
│   │   ├── SettingsScreen.jsx        # Settings screen container (Screen 5)
│   │   ├── SettingGroup.jsx          # Group heading + border + children
│   │   ├── SettingRow.jsx            # Label + control flex row
│   │   └── SaveIndicator.jsx         # Animated check icon
│   ├── ui/
│   │   ├── Toggle.jsx                # Toggle switch (role="switch")
│   │   ├── Toggle.test.jsx           # Toggle unit tests
│   │   ├── Button.jsx                # Existing
│   │   ├── Checkbox.jsx              # Existing
│   │   └── ConfirmDialog.jsx         # Existing
│   └── library/
│       └── ConversionOptions.jsx     # Updated: use useSettings() instead of loadSettings()
├── contexts/
│   ├── SettingsContext.jsx            # NEW: global settings state
│   ├── SettingsContext.test.jsx       # NEW: context unit tests
│   ├── ImportContext.jsx              # Unchanged
│   └── ConversionContext.jsx          # Unchanged
├── lib/
│   └── settings.js                    # Updated: extend DEFAULT_SETTINGS
└── App.jsx                            # Updated: add SettingsProvider, replace placeholder

src-tauri/src/
└── conversion/
    ├── mod.rs                         # Extended structs (PageHandlingOptions, OutputOptions)
    └── css.rs                         # Updated: textAlignment in generated CSS
```

## Integration Points

### SettingsContext → settings.js

```javascript
// On mount: load from disk
const stored = await loadSettings();
setSettings(stored);

// On change: save to disk (debounced)
const next = { ...prev, [group]: { ...prev[group], [key]: value } };
debouncedSave(next);
```

### SettingsScreen → SettingsContext

```javascript
const { settings, updateSetting, resetToDefaults } = useSettings();

// Each control calls updateSetting directly
<Toggle
  checked={settings.structure.detectHeadings}
  onChange={(v) => updateSetting('structure', 'detectHeadings', v)}
/>
```

### ConversionOptions → SettingsContext

```javascript
// Before (current):
const [globalSettings, setGlobalSettings] = useState(DEFAULT_SETTINGS);
useEffect(() => { loadSettings().then(setGlobalSettings); }, []);

// After:
const { settings: globalSettings } = useSettings();
```

### useConversion → SettingsContext

```javascript
// Before (current):
const globalSettings = await loadSettings();

// After:
const { settings: globalSettings } = useSettings();
```

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Race condition between rapid changes and debounced saves | The debounce always fires with the latest state, not the state at the time of the change. Even if the user navigates away, the debounce flush runs. |
| Corrupted settings file prevents app from loading | `loadSettings()` already catches parse errors and returns defaults. The SettingsContext handles this case identically. |
| New settings break existing `settings.json` files | `mergeSettings()` fills missing keys with defaults. Old files missing `keepPageBreaks` etc. are seamlessly upgraded on next save. |
| Cross-setting validation creates unexpected changes | Validation is limited to the WebP ↔ EPUB 2 conflict (the only hard dependency). Other warnings (heading detection + chapter split) are advisory-only, no automatic changes. |
| Context re-renders entire app on every setting change | The settings object is replaced only when a setting actually changes. Components that don't use settings don't re-render. React's reconciliation handles this efficiently for the small number of settings consumers. |
