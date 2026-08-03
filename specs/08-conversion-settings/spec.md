# 08 — Conversion Settings

## Goal

Implement the Settings screen (Screen 5) for configuring global PDF-to-EPUB conversion parameters. The screen provides a two-column grid of all conversion settings, with immediate persistence, a reset-to-defaults mechanism, and cross-setting validation. These global defaults feed into the conversion pipeline (spec 04) and can be overridden per-document in the Library detail panel (spec 06).

## Background

The conversion pipeline (spec 04) already reads settings from a JSON file in the app data directory via `src/lib/settings.js`. Default values are hardcoded in `DEFAULT_SETTINGS`. The per-document override UI exists in the Library screen's `ConversionOptions` component (spec 06). However, the Settings screen itself — where users configure global defaults — is a placeholder (`"Settings — coming soon"`). This spec replaces the placeholder with a fully functional settings interface.

The UI/UX design spec (02) defines the Settings screen layout, setting groups, controls, and interactions. Spec 05 (Storage Location) removed the "Output location" setting group since all output now goes to managed per-book directories.

## Functional Requirements

### FR-1: Settings Screen

The Settings screen replaces the current placeholder at the `/settings` route. It displays all global conversion parameters organized into setting groups in a two-column grid layout, following the design in spec 02.

### FR-2: Setting Groups

Four setting groups are displayed across two columns:

**Left column:**
- Structure Detection
- Images

**Right column:**
- Output Format
- Page Handling

The "Output location" group defined in spec 02 is not included — it was removed by spec 05 (Storage Location).

### FR-3: Structure Detection Settings

| Setting | Control | Default | Range | Description |
|---|---|---|---|---|
| Detect headings | Toggle | ON | — | Identify heading hierarchy from text patterns (font-size-aware detection in future versions). |
| Detect table of contents | Toggle | ON | — | Find and parse the PDF's table of contents page(s) to generate EPUB navigation. |
| Detect footnotes | Toggle | OFF | — | Identify footnotes and convert them to EPUB footnote markup. (Implementation deferred to future version; setting is exposed for forward compatibility.) |
| Heading level threshold | Number | 3 | 1–6 | Maximum heading depth to detect. Level 1 = largest headings only; level 6 = all sizes. |
| Paragraph detection | Toggle | ON | — | Merge text runs into semantic paragraphs based on spacing analysis. |
| List detection | Toggle | ON | — | Detect bulleted and numbered lists from indentation and symbol patterns. |

### FR-4: Images Settings

| Setting | Control | Default | Range | Description |
|---|---|---|---|---|
| Extract images | Toggle | ON | — | Extract embedded images from the PDF and include them in the EPUB. When OFF, images are dropped and only text is converted. |
| Image quality | Dropdown | Medium | High / Medium / Low | Compression quality for extracted images (JPEG/WebP). High = larger file, Low = smaller file. |
| Max image width (px) | Number | 800 | 200–2000, step 100 | Downscale images wider than this value. Prevents oversized images on e-reader screens. |
| Convert to WebP | Toggle | OFF | — | Convert extracted images to WebP format for smaller file sizes. Requires EPUB 3 — enabling this when EPUB 2 is selected triggers an automatic version upgrade with a warning toast. |

### FR-5: Output Format Settings

| Setting | Control | Default | Range | Description |
|---|---|---|---|---|
| EPUB version | Dropdown | EPUB 3 | EPUB 2 / EPUB 3 | EPUB 3 supports more features (WebP images, semantic markup, XHTML navigation). EPUB 2 has wider legacy e-reader support. |
| Embed fonts | Toggle | OFF | — | Embed the selected font family in the EPUB file. Increases file size but ensures consistent rendering across readers. (Implementation deferred in v1 — setting stored but not acted upon.) |
| Font family | Dropdown | Default | Default / Serif / Sans-serif / Monospace | The CSS `font-family` applied to the EPUB body text. "Default" defers to the e-reader's preference. |
| Base font size (pt) | Number | 12 | 8–24 | Default body text size in the EPUB CSS. E-readers typically allow users to override this. |
| Line height | Dropdown | 1.5 | 1.0 / 1.2 / 1.5 / 1.8 / 2.0 | CSS `line-height` for body text. Higher values improve readability at the cost of vertical space. |
| Margins (em) | Number | 1.0 | 0.5–3.0, step 0.5 | Left and right body text margins in em units. |
| Text alignment | Dropdown | Justify | Justify / Left / Right | CSS `text-align` for body paragraphs. Justify produces even edges; Left avoids uneven word spacing on narrow screens. |

### FR-6: Page Handling Settings

| Setting | Control | Default | Range | Description |
|---|---|---|---|---|
| Skip blank pages | Toggle | ON | — | Omit pages that contain no text or images from the EPUB output. |
| Page range | Dropdown | All | All / Custom | Convert all pages or a custom range. When "Custom" is selected, two number inputs appear inline: "From page" and "To page". |
| Split chapters by | Dropdown | Heading level 1 | Heading level 1 / Heading level 2 / Page break / None | How the converter decides where one EPUB chapter ends and the next begins. This is the primary chapter break strategy. |
| Keep page breaks | Toggle | OFF | — | Preserve original PDF page breaks within chapters as CSS page-break markers (`page-break-before: always`). When ON, page transitions from the source PDF are visible in the EPUB as forced breaks. When OFF (default), content flows continuously within chapters. |
| Remove page numbers | Toggle | ON | — | Strip page number artifacts from extracted text. PDF text extraction often captures headers/footers with page numbers that are meaningless in reflowable EPUB. |
| Cover page | Dropdown | Auto-detect | Auto-detect / First page / None | How to generate the EPUB cover. "Auto-detect" looks for a full-page image on page 1; "First page" renders page 1 as a cover image; "None" omits a cover entirely. |

### FR-7: Immediate Persistence

Settings are saved automatically when changed — there is no explicit "Save" button. Each change writes the updated settings object to `{app_data_dir}/settings.json` via the existing `saveSettings()` function. A brief save indicator (a small check icon that fades after 1 second) appears near the changed setting to confirm the save.

### FR-8: Reset to Defaults

A "Reset to defaults" secondary button in the screen header resets all settings to their factory defaults. Before resetting, a confirmation dialog is shown: "Reset all settings to factory defaults? Per-document overrides are not affected." On confirmation, all settings revert to `DEFAULT_SETTINGS` and the file is saved.

### FR-9: Cross-Setting Validation

Certain setting combinations trigger automatic adjustments or warnings:

| Trigger | Behaviour |
|---|---|
| Enable "Convert to WebP" when EPUB version is EPUB 2 | Warning toast: "WebP images require EPUB 3. The EPUB version setting will be changed to EPUB 3." The EPUB version dropdown updates to EPUB 3 automatically. |
| Change EPUB version to EPUB 2 when "Convert to WebP" is ON | "Convert to WebP" is automatically turned OFF. Warning toast: "WebP images are not supported in EPUB 2. Image conversion has been disabled." |
| Disable "Detect headings" when "Split chapters by" is a heading-based option | Warning toast: "Chapter splitting by headings requires heading detection. Consider changing the split strategy." (No automatic change — the user may intend a specific combination.) |
| Disable "Extract images" | "Image quality", "Max image width", and "Convert to WebP" controls become visually disabled (greyed out, non-interactive). |

### FR-10: Settings Context

A new `SettingsContext` wraps the application so any component can read global settings without prop-drilling. The context loads settings on mount and provides a dispatch function for updates. The per-document override UI (`ConversionOptions`) reads global defaults from this context instead of calling `loadSettings()` directly.

### FR-11: Settings Screen Keyboard Navigation

All settings controls are keyboard navigable per spec 02:
- `Tab` / `Shift+Tab` moves focus between controls in reading order.
- Toggle switches respond to `Space`.
- Dropdowns respond to `Enter` / arrow keys.
- Number inputs respond to arrow keys for increment/decrement.
- `Cmd/Ctrl + ,` navigates to the Settings screen from anywhere in the app.

## Non-Functional Requirements

### NFR-1: Render Performance

The settings screen should render and become interactive in under 100 ms. Settings load from disk on app startup (via SettingsContext), not on each screen visit.

### NFR-2: Save Latency

Each setting change should persist to disk in under 50 ms. Debouncing is applied to rapid changes (e.g., dragging a number input) with a 300 ms debounce window.

### NFR-3: Accessibility

All setting controls have associated labels. Toggle switches have `aria-checked`. Number inputs have `aria-valuemin`, `aria-valuemax`. Group headings are `<h4>` elements. The screen title is `<h3>`. Colour is not the sole state indicator — toggle labels show "ON"/"OFF" text alongside the visual switch.

### NFR-4: Responsive Layout

On windows narrower than 1000 px, the two-column grid collapses to a single column with all groups stacked vertically (left column groups first, then right column groups).

## UI Specification

### Header

- Title: "Conversion settings" (h3, 18 px, weight 500).
- "Reset to defaults" secondary button (right-aligned) with `rotate-ccw` icon.

### Settings Grid

Two-column CSS grid layout (`grid-template-columns: 1fr 1fr`, gap 24 px). Each column contains setting groups. Each group has:

- A heading (h4, 14 px, weight 500) with a 0.5 px bottom border in `var(--border)`.
- 20 px gap between groups.
- Setting rows with 6 px vertical padding each.

### Setting Row Layout

Each row is a flex container:
- Label: left-aligned, `var(--text-secondary)`, 13 px.
- Control: right-aligned.

### Control Styles

| Control | Spec |
|---|---|
| Toggle switch | 36 × 20 px, 10 px border-radius. OFF: `var(--surface-2)` background. ON: `var(--fill-accent)` background. Knob: 16 px circle, white. Transition: 150 ms ease. |
| Dropdown | `<select>`, 13 px text, `var(--radius)` border-radius, `var(--border)` border, `var(--surface-0)` background. |
| Number input | `<input type="number">`, 64 px width, 13 px text, same border/background as dropdown. |

### Save Indicator

A small check icon (`check`, 14 px, `var(--text-success)`) that appears beside the changed control, then fades out over 1 second using a CSS opacity transition.

## Out of Scope

- Preset profiles ("Technical book", "Novel", etc.) — future feature per spec 02.
- Font embedding implementation (setting is stored but not functional in v1).
- Footnote detection implementation (setting is stored for forward compatibility).
- Export/import settings to file.
- Undo for individual setting changes.
