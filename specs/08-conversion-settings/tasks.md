# 08 — Conversion Settings: Tasks

## Phase 1: Settings Schema & Context

### T1: Extend DEFAULT_SETTINGS
- [ ] In `src/lib/settings.js`, add `textAlignment: 'justify'` to `output` group
- [ ] In `src/lib/settings.js`, add `keepPageBreaks: false` to `pageHandling` group
- [ ] In `src/lib/settings.js`, add `removePageNumbers: true` to `pageHandling` group
- [ ] In `src/lib/settings.js`, add `coverPage: 'auto'` to `pageHandling` group
- [ ] Remove `outputLocation` from `DEFAULT_SETTINGS` if still present (removed by spec 05)
- [ ] Update `src/lib/settings.test.js` to verify new default values
- [ ] Verify `mergeSettings()` correctly fills in new fields when loading an old settings file (missing keys)

### T2: Create SettingsContext
- [ ] Create `src/contexts/SettingsContext.jsx`:
  - State: `settings` (initialized to `DEFAULT_SETTINGS`), `loaded` (boolean)
  - `useEffect` on mount: call `loadSettings()`, set state
  - `updateSetting(group, key, value)`: update state immediately, debounce save (300 ms)
  - Cross-setting validation within `updateSetting`:
    - Enable `convertToWebP` with `epubVersion === 'epub2'` → set `epubVersion` to `'epub3'`, emit warning toast
    - Set `epubVersion` to `'epub2'` with `convertToWebP === true` → set `convertToWebP` to `false`, emit warning toast
  - `resetToDefaults()`: set state to `DEFAULT_SETTINGS`, call `saveSettings(DEFAULT_SETTINGS)`
  - `useSettings()` hook with context guard
  - `SettingsProvider` component
- [ ] Create `src/contexts/SettingsContext.test.jsx`:
  - Test: initial state is DEFAULT_SETTINGS
  - Test: updateSetting updates a value in the correct group
  - Test: WebP + EPUB 2 cross-validation triggers version upgrade
  - Test: EPUB 2 + WebP cross-validation disables WebP
  - Test: resetToDefaults reverts all settings
  - Test: debounced save is called after updateSetting

### T3: Wire SettingsProvider into App
- [ ] Update `src/App.jsx`: wrap app with `<SettingsProvider>` (outermost provider)
- [ ] Verify all existing tests still pass after adding the provider

## Phase 2: Reusable UI Components

### T4: Toggle Component
- [ ] Create `src/components/ui/Toggle.jsx`:
  - `<button>` with `role="switch"`, `aria-checked={checked}`, `aria-label={label}`
  - 36 × 20 px, 10 px border-radius
  - OFF: `var(--surface-2)` background; ON: `var(--fill-accent)` background
  - Knob: 16 px white circle with 150 ms translate transition
  - `disabled` prop: opacity 50%, `cursor-not-allowed`
  - Responds to `Space` key (native button behaviour)
- [ ] Create `src/components/ui/Toggle.test.jsx`:
  - Test: renders unchecked state with correct aria attributes
  - Test: renders checked state
  - Test: clicking toggles the value
  - Test: disabled state prevents interaction
  - Test: has accessible label

### T5: SettingGroup Component
- [ ] Create `src/components/settings/SettingGroup.jsx`:
  - `<h4>` heading: 14 px, weight 500, with 0.5 px bottom border in `var(--border)`
  - 20 px gap below the heading
  - Renders children (SettingRow elements) in a flex column
- [ ] No separate test needed — tested through SettingsScreen integration tests

### T6: SettingRow Component
- [ ] Create `src/components/settings/SettingRow.jsx`:
  - Flex container, `justify-between`, `items-center`
  - Label: left-aligned, `var(--text-secondary)`, 13 px
  - Children (control): right-aligned
  - `disabled` prop: reduces opacity to 50%, children become non-interactive
  - 6 px vertical padding
- [ ] No separate test needed — tested through SettingsScreen integration tests

### T7: SaveIndicator Component
- [ ] Create `src/components/settings/SaveIndicator.jsx`:
  - Check icon (`check` from lucide-react), 14 px, `var(--text-success)`
  - When `visible` prop becomes `true`: opacity 1 → auto-fade to opacity 0 after 1 second
  - CSS transition: `opacity 0.5s ease` with a 0.5s delay
  - Returns to hidden after animation completes
- [ ] No separate test needed — visual component

## Phase 3: Settings Screen

### T8: SettingsScreen Component — Structure Detection Group
- [ ] Create `src/components/settings/SettingsScreen.jsx` with screen header:
  - Title: "Conversion settings" (h3, 18 px, weight 500)
  - "Reset to defaults" secondary button with `rotate-ccw` icon (right-aligned)
  - Confirmation dialog: "Reset all settings to factory defaults? Per-document overrides are not affected."
- [ ] Implement Structure Detection group (left column):
  - Detect headings — Toggle (default ON)
  - Detect table of contents — Toggle (default ON)
  - Detect footnotes — Toggle (default OFF)
  - Heading level threshold — Number input (1–6, default 3)
  - Paragraph detection — Toggle (default ON)
  - List detection — Toggle (default ON)
  - Each control wired to `updateSetting('structure', key, value)`

### T9: SettingsScreen Component — Images Group
- [ ] Implement Images group (left column):
  - Extract images — Toggle (default ON)
  - Image quality — Dropdown: High / Medium / Low (default Medium)
  - Max image width — Number input (200–2000, step 100, default 800), suffix "px"
  - Convert to WebP — Toggle (default OFF)
  - When "Extract images" is OFF: Image quality, Max image width, and Convert to WebP controls become disabled (greyed out, non-interactive)
  - Each control wired to `updateSetting('images', key, value)`

### T10: SettingsScreen Component — Output Format Group
- [ ] Implement Output Format group (right column):
  - EPUB version — Dropdown: EPUB 2 / EPUB 3 (default EPUB 3)
  - Embed fonts — Toggle (default OFF)
  - Font family — Dropdown: Default / Serif / Sans-serif / Monospace (default Default)
  - Base font size — Number input (8–24, default 12), suffix "pt"
  - Line height — Dropdown: 1.0 / 1.2 / 1.5 / 1.8 / 2.0 (default 1.5)
  - Margins — Number input (0.5–3.0, step 0.5, default 1.0), suffix "em"
  - Text alignment — Dropdown: Justify / Left / Right (default Justify)
  - Each control wired to `updateSetting('output', key, value)`

### T11: SettingsScreen Component — Page Handling Group
- [ ] Implement Page Handling group (right column):
  - Skip blank pages — Toggle (default ON)
  - Page range — Dropdown: All / Custom (default All)
    - When "Custom" selected: two inline number inputs appear: "From page" and "To page"
    - "From page" wired to `pageRangeFrom`, "To page" wired to `pageRangeTo`
  - Split chapters by — Dropdown: Heading level 1 / Heading level 2 / Page break / None (default Heading level 1)
  - Keep page breaks — Toggle (default OFF)
  - Remove page numbers — Toggle (default ON)
  - Cover page — Dropdown: Auto-detect / First page / None (default Auto-detect)
  - Each control wired to `updateSetting('pageHandling', key, value)`

### T12: Settings Grid Layout
- [ ] Implement the two-column CSS grid layout:
  - `grid-template-columns: 1fr 1fr`, gap 24 px
  - Left column: Structure Detection, Images
  - Right column: Output Format, Page Handling
  - Responsive: on windows < 1000 px wide, collapse to single column (`grid-template-columns: 1fr`)
- [ ] Add save indicator next to the last changed setting control

### T13: Wire SettingsScreen into App
- [ ] Update `src/App.jsx`:
  - Import `SettingsScreen`
  - Replace `<Placeholder title="Settings" />` with `<SettingsScreen />`
- [ ] Verify the `/settings` route renders the full settings screen

## Phase 4: Warning Toasts and Cross-Setting UX

### T14: Cross-Setting Warning Toasts
- [ ] When enabling "Convert to WebP" with EPUB 2 selected: toast warning "WebP images require EPUB 3. The EPUB version setting will be changed to EPUB 3."
- [ ] When switching to EPUB 2 with "Convert to WebP" enabled: toast warning "WebP images are not supported in EPUB 2. Image conversion has been disabled."
- [ ] When disabling "Detect headings" while "Split chapters by" is heading-based: toast warning "Chapter splitting by headings requires heading detection. Consider changing the split strategy." (advisory only — no automatic change)
- [ ] Toast notifications use the existing `sonner` Toaster configured in `App.jsx`

### T15: Disabled Control State for Image Sub-Settings
- [ ] When "Extract images" is toggled OFF:
  - "Image quality", "Max image width", and "Convert to WebP" controls render with `disabled` prop
  - Controls show at 50% opacity and do not respond to interaction
  - Underlying setting values are preserved (not reset)
- [ ] When "Extract images" is toggled back ON:
  - All image sub-settings become interactive again with their previous values

## Phase 5: Integration Updates

### T16: Update ConversionOptions to Use SettingsContext
- [ ] In `src/components/library/ConversionOptions.jsx`:
  - Replace `const [globalSettings, setGlobalSettings] = useState(DEFAULT_SETTINGS)` with `const { settings: globalSettings } = useSettings()`
  - Remove the `useEffect(() => { loadSettings().then(setGlobalSettings); }, [])` call
  - Import `useSettings` from `../../contexts/SettingsContext`
- [ ] Verify ConversionOptions tests still pass
- [ ] Verify per-document overrides still work correctly in the Library screen

### T17: Update useConversion to Use SettingsContext
- [ ] In `src/hooks/useConversion.js`:
  - Replace `const globalSettings = await loadSettings()` with reading from `useSettings()` hook
  - Ensure `getEffectiveSettings()` still receives the correct global settings
- [ ] Verify useConversion tests still pass

### T18: Extend Rust Backend Structs
- [ ] In `src-tauri/src/conversion/mod.rs`:
  - Add `keep_page_breaks: bool` to `PageHandlingOptions`
  - Add `remove_page_numbers: bool` to `PageHandlingOptions`
  - Add `cover_page: String` to `PageHandlingOptions`
  - Add `text_alignment: String` to `OutputOptions`
- [ ] In `src-tauri/src/conversion/css.rs`:
  - Update `generate_css()` to use the `text_alignment` setting for `text-align` property
- [ ] Run `cargo check` and `cargo test` in `src-tauri/` to verify compilation

## Phase 6: Testing

### T19: SettingsScreen Unit Tests
- [ ] Create `src/components/settings/SettingsScreen.test.jsx`:
  - Test: renders all four setting groups with correct headings
  - Test: all controls display their default values
  - Test: toggling a switch calls `updateSetting` with the correct group/key/value
  - Test: changing a dropdown calls `updateSetting` with the correct value
  - Test: changing a number input calls `updateSetting` with the correct value
  - Test: "Reset to defaults" button shows confirmation dialog
  - Test: confirming reset calls `resetToDefaults`
  - Test: cancelling reset does not change settings
  - Test: "Custom" page range shows From/To inputs
  - Test: selecting "All" page range hides From/To inputs
  - Test: disabling "Extract images" disables image sub-settings
  - Test: enabling "Extract images" re-enables image sub-settings

### T20: Settings Integration Test
- [ ] Test: changing a setting on the Settings screen is immediately visible in ConversionOptions (via shared SettingsContext)
- [ ] Test: the save indicator appears briefly after changing a setting
- [ ] Test: WebP + EPUB 2 cross-validation produces the correct automatic changes and toast
- [ ] Test: settings persist after simulated app reload (loadSettings returns updated values)

### T21: E2E Tests
- [ ] Test: navigate to Settings screen via sidebar — all controls render with default values
- [ ] Test: toggle "Detect headings" OFF → verify the toggle reflects the new state
- [ ] Test: change "Split chapters by" dropdown to "Page break" → verify selection
- [ ] Test: enable "Keep page breaks" toggle → verify state
- [ ] Test: set "Base font size" to 16 → verify the number input shows 16
- [ ] Test: change "Page range" to Custom → verify From/To inputs appear
- [ ] Test: click "Reset to defaults" → confirm → verify all controls return to defaults
- [ ] Test: disable "Extract images" → verify image sub-settings become disabled
- [ ] Test: keyboard navigation — Tab through controls, Space to toggle switches

## Acceptance Criteria

- [ ] The Settings screen replaces the placeholder at `/settings` with the full two-column settings grid
- [ ] All 23 settings controls (6 structure + 4 images + 7 output + 6 page handling) render with correct defaults
- [ ] Changing any setting value persists it immediately to `settings.json` (in Tauri mode)
- [ ] The save indicator briefly appears after each change
- [ ] "Reset to defaults" restores all settings after confirmation dialog
- [ ] Enabling "Convert to WebP" with EPUB 2 selected automatically upgrades to EPUB 3 with a warning toast
- [ ] Switching to EPUB 2 with "Convert to WebP" on automatically disables WebP with a warning toast
- [ ] Disabling "Extract images" greys out image sub-settings
- [ ] Selecting "Custom" page range reveals From/To number inputs
- [ ] Settings changes are immediately visible in the Library screen's ConversionOptions panel (via SettingsContext)
- [ ] The settings screen layout collapses to single column on windows narrower than 1000 px
- [ ] All controls are keyboard navigable (Tab, Space, Enter, arrow keys)
- [ ] Toggle switches have `role="switch"` and `aria-checked` attributes
- [ ] The screen works correctly in both light and dark modes
- [ ] New settings (`keepPageBreaks`, `removePageNumbers`, `coverPage`, `textAlignment`) appear in the UI with correct defaults and controls
