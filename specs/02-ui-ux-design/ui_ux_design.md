# 02 — UI/UX Design

## Goal

Define the complete user interface and user experience for the PDF-to-EPUB converter desktop application. The design prioritises readability of converted ebook content and a clear, focused workflow: import PDFs, inspect and configure, convert, and review the output.

## Background

The application is a React 19 + Vite 8 + Tailwind CSS 4 SPA running inside a Tauri v2 native window (see spec 01). The initial release targets PDF-to-EPUB conversion; the layout must accommodate additional input formats in later releases without a redesign.

## Design Principles

1. **Readability first** — every default and every setting exposed to the user should favour producing an EPUB that is comfortable to read on e-readers and tablets.
2. **Progressive disclosure** — the happy path (import → convert → open) requires no configuration. Advanced settings are reachable but not in the way.
3. **Batch-friendly** — users frequently convert several books at once; multi-select, queue management, and per-document overrides are first-class features.
4. **Predictable layout** — a persistent sidebar with four destinations keeps the user oriented at all times. The main content area adapts per screen but reuses the same preview + detail panel pattern wherever possible.
5. **Accessible** — all interactive elements are keyboard navigable, all icons have labels or are hidden from assistive technology, colour is never the sole indicator of state.

---

## Application Shell

### Window

- Native OS title bar with standard minimize / maximize / close controls.
- Window title: "Ebook Converter".
- Minimum window size: 960 × 640 px.
- Default window size: 1200 × 800 px.
- The window remembers its last size and position between sessions (Tauri window state plugin).

### Layout Structure

```
┌──────────────────────────────────────────────────────┐
│  OS Title Bar  —  "Ebook Converter"                  │
├────────────┬─────────────────────────────────────────┤
│            │                                         │
│  Sidebar   │         Main Content Area               │
│  (fixed,   │   ┌─────────────────────────────────┐   │
│   200 px)  │   │  Header (title + actions)       │   │
│            │   ├─────────────────────────────────┤   │
│  ┌──────┐  │   │                                 │   │
│  │Import│  │   │  Screen-specific content        │   │
│  │Libra.│  │   │                                 │   │
│  │Conve.│  │   │                                 │   │
│  │      │  │   │                                 │   │
│  │──────│  │   │                                 │   │
│  │Setti.│  │   │                                 │   │
│  └──────┘  │   └─────────────────────────────────┘   │
│            │                                         │
├────────────┴─────────────────────────────────────────┤
│  Status Bar (optional, conversion progress summary)  │
└──────────────────────────────────────────────────────┘
```

### Sidebar

The sidebar is a fixed 200 px column on the left. It does not collapse. It contains four navigation items and one section divider.

| Order | Icon             | Label     | Destination              |
|-------|------------------|-----------|--------------------------|
| 1     | `upload`         | Import    | Import screen            |
| 2     | `file-text`      | Library   | Source document library   |
| 3     | `book`           | Converted | Converted EPUB library   |
| —     | section divider  | Tools     | —                        |
| 4     | `settings`       | Settings  | Global conversion config |

**Active state**: highlighted background, accent text colour, font-weight 500.
**Hover state**: subtle background tint on non-active items.

The sidebar order mirrors the user's mental model of the conversion pipeline: bring files in, look at them, convert, look at the output.

### Status Bar (optional)

A thin bar at the bottom of the window. Visible only while a conversion is running. Shows:

- A global progress indicator (e.g. "Converting 1 of 3 — 65%").
- A cancel-all button.

When no conversion is active, the status bar is hidden and the main content area expands to fill the space.

---

## Screen 1 — Import

**Purpose**: bring PDF files into the application.

### Layout

```
┌─────────────────────────────────────────────────┐
│  Header                                         │
│  ┌─────────────────────────────────┬──────────┐ │
│  │  Import PDF files               │ [Browse  ││
│  │                                 │  files]  ││
│  └─────────────────────────────────┴──────────┘ │
│                                                 │
│  ┌─────────────────────────────────────────────┐│
│  │            Drop Zone                        ││
│  │                                             ││
│  │    ┌──────────┐                             ││
│  │    │  cloud   │                             ││
│  │    │  upload  │                             ││
│  │    │  icon    │                             ││
│  │    └──────────┘                             ││
│  │                                             ││
│  │    Drop PDF files here                      ││
│  │    or click "Browse files" to select        ││
│  │    from your computer                       ││
│  │                                             ││
│  └─────────────────────────────────────────────┘│
│                                                 │
│  Recent imports                                 │
│  ┌─────────────────────────────────────────────┐│
│  │ ☐  📄 Design patterns.pdf    12.4 MB  Ready ││
│  │ ☐  📄 Clean architecture.pdf  8.7 MB  Ready ││
│  │ ☐  📄 Pragmatic programmer   15.2 MB  Done  ││
│  └─────────────────────────────────────────────┘│
│                                                 │
│               [Remove selected] [Convert sel.]  │
└─────────────────────────────────────────────────┘
```

### Header

- Title: "Import PDF files" (h3, 18 px, weight 500).
- Primary action button: "Browse files" with a `folder-open` icon. Triggers the Tauri native file picker dialog filtered to `.pdf` files.

### Drop Zone

- Occupies the full width of the main content area.
- Dashed border (1.5 px, strong border colour), 12 px border-radius.
- Centre-aligned content: large `cloud-upload` icon (32 px), primary text "Drop PDF files here", secondary hint text "or click 'Browse files' to select from your computer".
- **Drag-over state**: border colour changes to accent, background tints to light accent. A subtle scale animation (1.01×) signals the zone is active.
- **Drop behaviour**: accepts one or more `.pdf` files. Non-PDF files are silently ignored. Duplicate files (same path already imported) are skipped with a brief toast notification.

### Import List

- Appears below the drop zone under a muted label "Recent imports".
- Each row contains, left to right:
  - **Checkbox** (16 × 16 px) — for batch selection.
  - **File icon** — PDF icon, accent colour.
  - **File name** — weight 500, primary text colour. Truncated with ellipsis if longer than the available space.
  - **File size** — muted text, 12 px (e.g. "12.4 MB").
  - **Status badge** — pill shape, coloured by status:
    - `Ready` — accent background/text. The file is imported and ready for conversion.
    - `Converting` — warning background/text. Conversion is in progress.
    - `Converted` — success background/text. An EPUB has been produced.
    - `Error` — danger background/text. Conversion failed.
- Rows have a 0.5 px bottom border separator.
- The list is scrollable if it exceeds the visible area. Max visible height before scrolling: ~300 px (depends on window size).
- **Empty state**: when no files have been imported, the list area shows a single line of muted text: "No files imported yet."

### Batch Actions

Below the import list, right-aligned:

- **"Remove selected"** — secondary button. Removes checked files from the import list. Does not delete the source PDF from disk. Disabled when no rows are checked.
- **"Convert selected"** — primary button with a `transform` icon. Starts conversion for all checked files that have status `Ready`. Disabled when no convertible rows are checked.

### Interactions

| Action | Result |
|---|---|
| Click "Browse files" | Tauri native file dialog opens, filtered to `*.pdf`. Selected files are added to the import list with status `Ready`. |
| Drag files onto drop zone | Valid PDFs are added to the import list. Non-PDFs are ignored. |
| Drop duplicate file | Toast: "File already imported" (auto-dismiss after 3 s). |
| Check rows + click "Convert selected" | Navigates to the Converting screen (screen 3). Conversion begins immediately. |
| Check rows + click "Remove selected" | Confirmation dialog: "Remove N file(s) from the import list? The source PDFs on disk are not affected." |
| Click a file name | Navigates to the Library screen (screen 2) with that document selected. |

---

## Screen 2 — Library (Source Document Preview + Details)

**Purpose**: inspect imported PDF documents before conversion. View page-by-page preview, read metadata, and optionally set per-document conversion overrides.

### Layout

```
┌──────────────────────────────────────────────────────┐
│  Header                                              │
│  ┌──────────────────────────────────┬──────────────┐ │
│  │  Library                         │ 🔍 Search    │ │
│  └──────────────────────────────────┴──────────────┘ │
│                                                      │
│  ┌───────────────────────┬──────────────────────────┐│
│  │   Document List       │   Detail Panel           ││
│  │   (left, 260 px)      │   (right, fills)         ││
│  │                       │                          ││
│  │  ▸ Design patterns    │  ┌────────────────────┐  ││
│  │    Clean architecture │  │   Page Preview     │  ││
│  │    Pragmatic progr.   │  │                    │  ││
│  │                       │  │   ┌──────────┐     │  ││
│  │                       │  │   │  page    │     │  ││
│  │                       │  │   │  render  │     │  ││
│  │                       │  │   └──────────┘     │  ││
│  │                       │  │   ◀ Page 1/384 ▶   │  ││
│  │                       │  └────────────────────┘  ││
│  │                       │                          ││
│  │                       │  Metadata                ││
│  │                       │  ─────────────────────── ││
│  │                       │  Title: Design patterns  ││
│  │                       │  Authors: Gamma, Helm…   ││
│  │                       │  Pages: 384              ││
│  │                       │  Size: 12.4 MB           ││
│  │                       │  Format: PDF 1.7         ││
│  │                       │  Created: 1994-10-21     ││
│  │                       │                          ││
│  │                       │  ▸ Conversion options     ││
│  │                       │                          ││
│  │                       │  [Convert to EPUB]       ││
│  └───────────────────────┴──────────────────────────┘│
└──────────────────────────────────────────────────────┘
```

### Header

- Title: "Library" (h3).
- Search input (right-aligned): filters the document list by file name. 180 px wide, with a `search` icon inside the field. Placeholder: "Search documents...".

### Document List (Left Panel)

- Fixed width: 260 px.
- A vertical list of all imported PDFs.
- Each item shows:
  - File name (truncated with ellipsis).
  - File size (muted, 12 px, below the name).
  - Status badge (same colours as the import list).
- **Selected state**: accent background, bold text.
- **Hover state**: subtle background tint.
- Clicking a document populates the detail panel on the right.
- The list scrolls independently of the detail panel.

### Detail Panel (Right Side)

Occupies the remaining width. Divided into three vertical sections.

#### Section A — Page Preview

- A rendered preview of the selected PDF page.
- The page is displayed centred within a bordered container with a light background.
- Below the preview: navigation controls.
  - Left arrow button (previous page).
  - Page indicator: "Page N of M" (muted text, 12 px).
  - Right arrow button (next page).
- Keyboard navigation: Left/Right arrow keys change pages when the preview is focused.
- The preview renders the PDF page as an image or via a canvas element. The Rust backend extracts the page and sends it to the frontend as a base64-encoded image or via a streaming mechanism.

#### Section B — Metadata

A labelled list of document properties. Each property has:
- A label (11 px, muted, uppercase, 0.3 px letter-spacing).
- A value (13 px, primary text).
- 12 px vertical spacing between properties.

Properties displayed:

| Label | Source | Example |
|---|---|---|
| Title | PDF metadata `/Title` | Design patterns |
| Authors | PDF metadata `/Author` | Gamma, Helm, Johnson, Vlissides |
| Pages | Page count | 384 |
| File size | File system | 12.4 MB |
| Format | PDF version header | PDF 1.7 |
| Created | PDF metadata `/CreationDate` | 1994-10-21 |
| Modified | PDF metadata `/ModDate` | 2004-03-15 |
| Producer | PDF metadata `/Producer` | Adobe Acrobat 6.0 |

If a metadata field is absent from the PDF, the row is hidden (not shown as "Unknown").

#### Section C — Conversion Options (Per-Document Overrides)

This section implements the per-document override model described below in the "Settings Architecture" section.

- Collapsed by default. Shown as a clickable row: `▸ Conversion options` with a chevron icon.
- When expanded (`▾ Conversion options`), it shows a subset of the global settings with per-document override controls.
- Each setting row has three states:
  1. **Default** — value is inherited from global settings. Displayed in muted text with a label "(default)" next to the value.
  2. **Overridden** — the user has set a custom value for this document. Displayed in primary text. A small "reset" icon button appears to the right to clear the override.
  3. **Editing** — the control (toggle, dropdown, number input) is active.
- The override indicator: when at least one setting is overridden, the collapsed row shows a count badge: `▸ Conversion options (2 overrides)`.
- Full details of which settings are overridable and the UI for each are in the "Settings Architecture" section.

#### Convert Button

- Below the conversion options section.
- Full-width primary button: "Convert to EPUB" with a `transform` icon.
- Disabled if the document status is already `Converting` or `Converted`.
- If the document is already converted, the button text changes to "Reconvert to EPUB" and the button style changes to secondary.

### Interactions

| Action | Result |
|---|---|
| Click document in list | Selects it; detail panel updates with preview, metadata, and conversion options. |
| Type in search field | Document list filters in real time (case-insensitive substring match on file name). |
| Click page navigation arrows | Preview updates to the next/previous page. |
| Expand "Conversion options" | Override controls become visible. |
| Change an override value | The override is saved immediately (no separate save button). A "(default)" label is removed and a reset button appears. |
| Click reset on an override | The value reverts to the global default. |
| Click "Convert to EPUB" | Conversion starts for this single document. User is navigated to the Converting screen. |

---

## Screen 3 — Converting (Progress)

**Purpose**: show real-time progress of active conversions.

### Layout

```
┌──────────────────────────────────────────────────────┐
│  Header                                              │
│  ┌──────────────────────────────────┬──────────────┐ │
│  │  Converting                      │ [Cancel all] │ │
│  └──────────────────────────────────┴──────────────┘ │
│                                                      │
│  ┌──────────────────────────────────────────────────┐│
│  │  📄 Design patterns.pdf          65%  Converting ││
│  │  ████████████████░░░░░░░░                        ││
│  ├──────────────────────────────────────────────────┤│
│  │  📄 Clean architecture.pdf       Queued  Pending ││
│  └──────────────────────────────────────────────────┘│
│                                                      │
│  ┌──────────────────────────────────────────────────┐│
│  │  ℹ  Conversion log                               ││
│  │  ─────────────────────────────────────────────── ││
│  │  Extracting text from pages 1–248...             ││
│  │  Detecting headings and structure...             ││
│  │  Rebuilding table of contents...                 ││
│  │  Extracting images (23 of 47)...                 ││
│  └──────────────────────────────────────────────────┘│
│                                                      │
│  Completed                                           │
│  ┌──────────────────────────────────────────────────┐│
│  │  📄 Pragmatic programmer.pdf     ✓     Converted ││
│  └──────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────┘
```

### Header

- Title: "Converting" (h3).
- "Cancel all" secondary button (right-aligned). Visible only when at least one conversion is active or queued.

### Conversion Queue

A vertical list of documents being converted, divided into two sections.

#### Active / Queued Section

Each row shows:
- PDF file icon.
- File name (weight 500).
- Progress percentage (muted text, right-aligned) — or "Queued" for pending items.
- Status badge: `Converting` (warning) or `Pending` (accent).

The currently converting file has a progress bar below its row:
- 4 px height, rounded, fills left to right.
- Accent colour for the filled portion, border colour for the track.

#### Conversion Log

Below the queue, a bordered panel shows real-time log output from the conversion engine:
- Info icon + "Conversion log" header (weight 500).
- Monospace text (12 px, muted colour) showing the current operation.
- The log auto-scrolls to the latest entry.
- Log entries include:
  - `Extracting text from pages N–M...`
  - `Detecting headings and structure...`
  - `Rebuilding table of contents...`
  - `Extracting images (X of Y)...`
  - `Generating EPUB structure...`
  - `Writing EPUB file...`
  - `Conversion complete.`
- If an error occurs: `Error: <message>` in danger text colour.

#### Completed Section

Below the active queue, under a "Completed" muted label:
- Documents that finished conversion in this session.
- Each row shows a check icon (success colour) and status badge `Converted`.
- Clicking a completed row navigates to the Converted screen (screen 4) with that document selected.

### Interactions

| Action | Result |
|---|---|
| Conversion completes | Row moves from active section to completed section. Next queued item begins. |
| All conversions complete | Header changes to "Conversion complete". "Cancel all" button is replaced by "View converted" primary button. |
| Click "Cancel all" | Confirmation dialog: "Cancel N remaining conversion(s)? Files already converted are not affected." Active conversion stops, queued items are removed. |
| Click a row in the active section | No action (conversion is in progress). |
| Click a completed row | Navigate to Converted screen with that EPUB selected. |
| Conversion fails for a file | Row shows danger status badge `Error`. An expandable error detail is available inline. Other queued items continue. |

---

## Screen 4 — Converted (EPUB Library + Preview)

**Purpose**: browse, preview, and manage converted EPUB files.

### Layout

Mirrors the Library screen (screen 2) layout for consistency, but shows EPUB-specific information.

```
┌──────────────────────────────────────────────────────┐
│  Header                                              │
│  ┌──────────────────────────────────┬──────────────┐ │
│  │  Converted EPUBs                 │ [Open folder]│ │
│  └──────────────────────────────────┴──────────────┘ │
│                                                      │
│  ┌───────────────────────┬──────────────────────────┐│
│  │   EPUB List           │   Detail Panel           ││
│  │   (left, 260 px)      │   (right, fills)         ││
│  │                       │                          ││
│  │  ▸ Design patterns    │  ┌────────────────────┐  ││
│  │    Pragmatic progr.   │  │   EPUB Preview     │  ││
│  │                       │  │                    │  ││
│  │                       │  │   ┌──────────┐     │  ││
│  │                       │  │   │ chapter  │     │  ││
│  │                       │  │   │ render   │     │  ││
│  │                       │  │   └──────────┘     │  ││
│  │                       │  │  ◀ Ch. 1/23 ▶      │  ││
│  │                       │  └────────────────────┘  ││
│  │                       │                          ││
│  │                       │  Metadata                ││
│  │                       │  ─────────────────────── ││
│  │                       │  Source: Design pat…pdf  ││
│  │                       │  EPUB size: 3.1 MB       ││
│  │                       │  Chapters: 23            ││
│  │                       │  Images: 47 extracted    ││
│  │                       │  Converted: 2026-08-02   ││
│  │                       │                          ││
│  │                       │  ▸ Table of contents      ││
│  │                       │                          ││
│  │                       │  [Open in reader]        ││
│  │                       │  [Save as...]            ││
│  │                       │  [Reconvert]             ││
│  └───────────────────────┴──────────────────────────┘│
└──────────────────────────────────────────────────────┘
```

### Header

- Title: "Converted EPUBs" (h3).
- "Open folder" secondary button with a `folder` icon. Opens the output folder in the OS file manager.

### EPUB List (Left Panel)

- Same layout as the Library document list (260 px width, scrollable).
- Each item shows:
  - File name (`.epub` extension shown).
  - EPUB file size (muted, 12 px).
  - Conversion date (muted, 12 px).

### Detail Panel (Right Side)

#### Section A — EPUB Preview

- Renders the EPUB content chapter by chapter.
- Navigation controls below the preview:
  - Left arrow button (previous chapter).
  - Chapter indicator: "Chapter N of M" (muted text, 12 px).
  - Right arrow button (next chapter).
- The preview renders the EPUB HTML content in a sandboxed container, applying the same font and size settings used during conversion, giving the user a representative preview of the reading experience.

#### Section B — Metadata

| Label | Source | Example |
|---|---|---|
| Source | Original PDF file name | Design patterns.pdf |
| EPUB size | File system | 3.1 MB |
| Chapters | EPUB spine count | 23 |
| Images | Count of extracted images | 47 extracted |
| Converted | Timestamp of conversion | 2026-08-02 14:32 |
| EPUB version | From conversion settings | EPUB 3 |
| Settings used | "Default" or "2 overrides" | Default |

#### Section C — Table of Contents

- Collapsed by default: `▸ Table of contents`.
- When expanded, shows the EPUB's generated TOC as an indented, clickable list.
- Clicking a TOC entry navigates the preview to that chapter.
- Nested entries are indented by 16 px per level.

#### Action Buttons

Stacked vertically, full width:

1. **"Open in reader"** — primary button, `external-link` icon. Opens the EPUB file in the system's default EPUB reader application.
2. **"Save as..."** — secondary button, `download` icon. Opens a Tauri native save dialog so the user can save a copy to a different location.
3. **"Reconvert"** — secondary button, `refresh` icon. Returns to the Library screen with this document's source PDF selected and the conversion options panel expanded. Allows the user to adjust settings and re-run the conversion.

### Interactions

| Action | Result |
|---|---|
| Click EPUB in list | Selects it; detail panel updates. |
| Click chapter navigation arrows | Preview updates to the next/previous chapter. |
| Click TOC entry | Preview jumps to that chapter. |
| Click "Open in reader" | OS launches the default EPUB reader with this file. |
| Click "Save as..." | Tauri save dialog opens with the EPUB file name pre-filled. |
| Click "Reconvert" | Navigate to Library screen; source PDF selected; conversion options expanded. |

---

## Screen 5 — Settings (Global Conversion Defaults)

**Purpose**: configure default conversion parameters that apply to all documents unless overridden per-document.

### Layout

```
┌──────────────────────────────────────────────────────┐
│  Header                                              │
│  ┌──────────────────────────────────┬──────────────┐ │
│  │  Conversion settings             │[Reset to def]│ │
│  └──────────────────────────────────┴──────────────┘ │
│                                                      │
│  ┌────────────────────────┬─────────────────────────┐│
│  │  Left Column           │  Right Column           ││
│  │                        │                         ││
│  │  Structure detection   │  Output format          ││
│  │  ───────────────────── │  ──────────────────     ││
│  │  Detect headings  [ON] │  EPUB version  [v3]    ││
│  │  Detect TOC       [ON] │  Embed fonts   [OFF]   ││
│  │  Detect footnotes[OFF] │  Font family [Default] ││
│  │  Heading thresh.  [3]  │  Base font size [12pt] ││
│  │  Paragraph det.   [ON] │  Line height   [1.5]   ││
│  │  List detection   [ON] │  Margins (em)  [1.0]   ││
│  │                        │                         ││
│  │  Images                │  Output location        ││
│  │  ───────────────────── │  ──────────────────     ││
│  │  Extract images   [ON] │  Default folder         ││
│  │  Image quality  [Med]  │  ~/Documents/Ebooks     ││
│  │  Max width (px) [800]  │  [Choose]               ││
│  │  Convert to WebP [OFF] │                         ││
│  │                        │  Page handling          ││
│  │                        │  ──────────────────     ││
│  │                        │  Skip blank pages [ON]  ││
│  │                        │  Page range     [All]   ││
│  │                        │  Split chapters by      ││
│  │                        │    [Heading level 1]    ││
│  └────────────────────────┴─────────────────────────┘│
└──────────────────────────────────────────────────────┘
```

### Header

- Title: "Conversion settings" (h3).
- "Reset to defaults" secondary button (right-aligned). Resets all settings to their factory defaults after a confirmation dialog.

### Settings Grid

A two-column grid layout. Each column contains one or more setting groups. Each group has:

- A heading (h4, 14 px, weight 500) with a 0.5 px bottom border.
- A list of setting rows.

Each setting row has:
- A label (left-aligned, secondary text colour, 13 px).
- A control (right-aligned): toggle switch, dropdown `<select>`, or number `<input>`.

### Setting Groups and Parameters

#### Group: Structure Detection (Left Column)

| Setting | Control | Default | Description |
|---|---|---|---|
| Detect headings | Toggle | ON | Identify heading hierarchy from font size and weight patterns in the PDF. |
| Detect table of contents | Toggle | ON | Find and parse the PDF's table of contents page(s) to generate EPUB navigation. |
| Detect footnotes | Toggle | OFF | Identify footnotes and convert them to EPUB footnote markup. |
| Heading level threshold | Number (1–6) | 3 | Maximum heading depth to detect. Level 1 = largest headings only; level 6 = all sizes. |
| Paragraph detection | Toggle | ON | Merge text runs into semantic paragraphs based on spacing analysis. |
| List detection | Toggle | ON | Detect bulleted and numbered lists from indentation and symbol patterns. |

#### Group: Images (Left Column)

| Setting | Control | Default | Description |
|---|---|---|---|
| Extract images | Toggle | ON | Extract embedded images from the PDF and include them in the EPUB. |
| Image quality | Dropdown: High / Medium / Low | Medium | JPEG/WebP compression quality for extracted images. High = larger file, Low = smaller file. |
| Max image width (px) | Number (200–2000, step 100) | 800 | Downscale images wider than this value. Prevents oversized images on e-reader screens. |
| Convert to WebP | Toggle | OFF | Convert extracted images to WebP format for smaller file sizes. Requires EPUB 3. |

#### Group: Output Format (Right Column)

| Setting | Control | Default | Description |
|---|---|---|---|
| EPUB version | Dropdown: EPUB 2 / EPUB 3 | EPUB 3 | EPUB 3 supports more features (WebP, footnotes, semantic markup). EPUB 2 has wider legacy reader support. |
| Embed fonts | Toggle | OFF | Embed the selected font family in the EPUB file. Increases file size but ensures consistent rendering. |
| Font family | Dropdown: Default / Serif / Sans-serif / Monospace | Default | The CSS font-family applied to the EPUB body text. "Default" defers to the e-reader's preference. |
| Base font size (pt) | Number (8–24) | 12 | The default body text size in the EPUB CSS. E-readers typically allow users to override this. |
| Line height | Dropdown: 1.0 / 1.2 / 1.5 / 1.8 / 2.0 | 1.5 | CSS line-height for body text. Higher values improve readability at the cost of vertical space. |
| Margins (em) | Number (0.5–3.0, step 0.5) | 1.0 | Left and right margins for body text in em units. |

#### Group: Output Location (Right Column)

| Setting | Control | Default | Description |
|---|---|---|---|
| Default output folder | Button: "Choose" | `~/Documents/Ebooks` | The folder where converted EPUB files are saved by default. The current path is displayed below the button as muted text. |

#### Group: Page Handling (Right Column)

| Setting | Control | Default | Description |
|---|---|---|---|
| Skip blank pages | Toggle | ON | Omit pages that contain no text or images from the EPUB output. |
| Page range | Dropdown: All / Custom | All | Convert all pages or a custom range. When "Custom" is selected, two number inputs appear: "From" and "To". |
| Split chapters by | Dropdown: Heading level 1 / Heading level 2 / Page break / None | Heading level 1 | How the converter decides where one EPUB chapter ends and the next begins. |

### Setting Persistence

- Settings are saved to a JSON file on disk via the Tauri filesystem plugin.
- Location: the application's data directory (OS-specific, managed by Tauri).
- Settings are saved immediately when changed (no save button needed).
- The settings file is human-readable and editable for power users.

### Interactions

| Action | Result |
|---|---|
| Change any setting | Value is saved immediately. A brief, non-intrusive save indicator appears (e.g., a small check icon that fades after 1 s). |
| Click "Reset to defaults" | Confirmation dialog: "Reset all settings to factory defaults? Per-document overrides are not affected." On confirm, all settings revert. |
| Click "Choose" for output folder | Tauri native folder picker dialog opens. Selected folder path replaces the displayed path. |
| Select "Custom" page range | Two number inputs appear inline: "From page" and "To page". |
| Enable "Convert to WebP" when EPUB 2 is selected | A warning appears below the toggle: "WebP images require EPUB 3. The EPUB version setting will be changed to EPUB 3." The EPUB version dropdown updates automatically. |

---

## Settings Architecture — Per-Document Overrides

### Data Model

```
GlobalSettings {
  structure: {
    detectHeadings: boolean
    detectToc: boolean
    detectFootnotes: boolean
    headingLevelThreshold: number
    paragraphDetection: boolean
    listDetection: boolean
  }
  images: {
    extractImages: boolean
    imageQuality: "high" | "medium" | "low"
    maxImageWidth: number
    convertToWebP: boolean
  }
  output: {
    epubVersion: "epub2" | "epub3"
    embedFonts: boolean
    fontFamily: "default" | "serif" | "sans-serif" | "monospace"
    baseFontSize: number
    lineHeight: number
    margins: number
  }
  pageHandling: {
    skipBlankPages: boolean
    pageRange: "all" | "custom"
    pageRangeFrom: number | null
    pageRangeTo: number | null
    splitChaptersBy: "heading1" | "heading2" | "pageBreak" | "none"
  }
  outputLocation: {
    defaultFolder: string
  }
}

DocumentOverrides {
  documentId: string
  overrides: Partial<Omit<GlobalSettings, "outputLocation">>
}
```

### Merge Logic

When a conversion starts, the effective settings for a document are computed by merging the document's overrides on top of the global settings:

```
effectiveSettings = deepMerge(globalSettings, document.overrides)
```

- Only keys explicitly set in `document.overrides` replace the global value.
- Absent keys inherit the global default.
- The `outputLocation` group is never overridable per-document; it is always global.

### Per-Document Override UI (in Library Detail Panel)

The "Conversion options" section in the Library detail panel shows overridable settings grouped into collapsible sub-sections that mirror the global settings groups.

#### Which Settings Are Overridable

All settings except `outputLocation.defaultFolder` can be overridden per document. The most commonly overridden settings are shown at the top level; the rest are in a "More options" expandable.

**Top-level overrides** (always visible when the section is expanded):

| Setting | Control | Inherited display |
|---|---|---|
| Split chapters by | Dropdown | "Heading level 1 (default)" |
| Heading level threshold | Number | "3 (default)" |
| Base font size | Number | "12 pt (default)" |
| Image quality | Dropdown | "Medium (default)" |
| Page range | Dropdown | "All (default)" |

**"More options" overrides** (collapsed by default, expandable):

All other settings from the global settings, excluding output location.

#### UI Behaviour for Each Override

Each overridable setting row in the per-document panel has three visual states:

1. **Inherited (default)**: the control shows the current global value, appended with "(default)" in muted text. The control is interactive — clicking/changing it creates an override.

2. **Overridden**: the control shows the per-document value in primary text. A small `x` (reset) icon button appears to the right of the control. The "(default)" label is removed.

3. **Indicator on collapsed section**: when the "Conversion options" section is collapsed, the header shows the override count:
   - No overrides: `▸ Conversion options`
   - With overrides: `▸ Conversion options · 2 custom` (the count "2 custom" is displayed in accent colour).

#### Reset Behaviour

- Clicking the reset icon on a single setting removes that override; the value reverts to the current global default.
- If the user changes a global setting, all documents inheriting that setting (i.e., not overriding it) immediately reflect the new value. Documents with an override for that setting are unaffected.

#### Edge Cases

| Scenario | Behaviour |
|---|---|
| User sets a per-document override to the same value as the global default | The override is still stored (it protects against future global changes). The row shows the value without "(default)" and with the reset icon. |
| User changes a global setting after per-document overrides exist | Override values are untouched. Inherited values update. |
| User resets all global settings to defaults | Per-document overrides remain; only inherited values change. |
| User deletes a document from the import list | Its overrides are also deleted. |
| User reconverts a document | Existing overrides are preserved and pre-populated in the conversion options panel. |

---

## Cross-Cutting Concerns

### Toast Notifications

Short-lived messages for non-blocking feedback. Appear at the bottom-centre of the main content area. Auto-dismiss after 3 seconds. Include:

- Success: "File converted successfully"
- Info: "File already imported"
- Warning: "WebP requires EPUB 3 — version updated"
- Error: "Conversion failed: <reason>"

Toasts stack vertically (newest on top, max 3 visible). Each toast has a manual dismiss (×) button.

### Keyboard Navigation

| Key | Context | Action |
|---|---|---|
| `Tab` / `Shift+Tab` | Global | Move focus between interactive elements in reading order. |
| `Enter` / `Space` | Button focused | Activate the button. |
| `Arrow Left` / `Arrow Right` | Page preview focused | Navigate pages (Library) or chapters (Converted). |
| `Escape` | Dialog open | Close the dialog. |
| `Cmd/Ctrl + O` | Global | Open the file picker (same as "Browse files"). |
| `Cmd/Ctrl + ,` | Global | Navigate to Settings. |
| `Delete` / `Backspace` | Import list, row selected | Remove selected files (with confirmation). |

### Dark Mode

The application follows the OS dark mode preference. All colour values use CSS custom properties that adapt automatically. No manual toggle is needed (the OS setting is the source of truth).

### Error States

| Error | Display |
|---|---|
| PDF cannot be read (corrupted) | Import list row shows `Error` badge. Clicking the row shows an inline error message: "This file could not be read. It may be corrupted or password-protected." |
| PDF is password-protected | Import list row shows `Error` badge with message: "This file is password-protected. Encrypted PDFs are not supported." |
| Conversion fails mid-process | Converting screen shows the error inline in the log panel (danger colour). The file row shows `Error` badge. Other queued files continue. |
| Disk full during conversion | Toast notification (danger): "Not enough disk space to save the EPUB. Free some space and try again." |
| Output folder does not exist | Toast notification (warning): "The output folder no longer exists. Please choose a new one in Settings." Settings screen opens. |

### File Size Formatting

- Bytes < 1 KB: show in bytes ("842 B").
- 1 KB – 999 KB: show in KB with no decimals ("342 KB").
- 1 MB – 999 MB: show in MB with one decimal ("12.4 MB").
- 1 GB+: show in GB with two decimals ("1.23 GB").

### Empty States

| Screen | Condition | Message |
|---|---|---|
| Import | No files imported | "No files imported yet. Drop PDF files here or click 'Browse files' to get started." (centred in the list area, muted text, with an `upload` icon above.) |
| Library | No files imported | "Your library is empty. Import some PDFs to get started." with a "Go to Import" button. |
| Converted | No conversions done | "No converted files yet. Import and convert a PDF to see it here." with a "Go to Import" button. |

---

## Visual Specifications

### Typography

| Element | Size | Weight | Colour |
|---|---|---|---|
| Screen title (h3) | 18 px | 500 | Primary |
| Setting group heading (h4) | 14 px | 500 | Primary |
| Body text / file names | 13 px | 400 (names: 500) | Primary |
| Labels, metadata keys | 11–12 px | 400 | Muted |
| Status badges | 12 px | 400 | Role-based |
| Conversion log | 12 px (monospace) | 400 | Muted |
| Hint text | 12 px | 400 | Muted |

### Spacing

| Context | Value |
|---|---|
| Sidebar item padding | 8 px vertical, 16 px horizontal |
| Main content padding | 20 px top/bottom, 24 px left/right |
| Gap between setting rows | 6 px (padding on each row) |
| Gap between setting groups | 20 px |
| Drop zone padding | 40 px vertical, 20 px horizontal |
| File list row padding | 10 px vertical, 14 px horizontal |

### Border Radii

| Element | Radius |
|---|---|
| Cards, drop zone, preview container | 12 px |
| Buttons, inputs, badges | 8 px (var --radius) |
| Status badge pills | 99 px (fully rounded) |
| Progress bar | 2 px |
| Checkboxes | 4 px |

### Colours (Light Mode)

All colours use CSS custom properties for automatic dark mode support. No hardcoded hex values in components.

| Element | Token |
|---|---|
| Page background | `--surface-0` |
| Sidebar background | `--surface-0` |
| Main content background | `--surface-1` |
| Card/preview background | `--surface-2` |
| Default border | `--border` (0.5 px) |
| Primary button | `--fill-accent` bg, `--on-accent` text |
| Secondary button | transparent bg, `--border-strong` border |
| Ready badge | `--bg-accent` bg, `--text-accent` text |
| Converting badge | `--bg-warning` bg, `--text-warning` text |
| Converted badge | `--bg-success` bg, `--text-success` text |
| Error badge | `--bg-danger` bg, `--text-danger` text |

---

## Future Considerations (Out of Scope for v1)

- **Additional input formats**: DJVU, MOBI, AZW3. The sidebar "Import" screen would gain a format filter dropdown. The Library and Settings screens already accommodate format-agnostic metadata.
- **Preset profiles**: named setting combinations ("Technical book", "Novel", "Scanned PDF") saved and selectable from a dropdown. Per-document overrides and profiles are complementary — a profile sets a baseline, overrides fine-tune from there.
- **Batch conversion settings**: apply the same override set to multiple selected documents at once.
- **Conversion history**: a log of past conversions with timestamps, source/output paths, and settings used. Useful for reproducibility.
- **EPUB validation**: after conversion, run an EPUB validator (epubcheck) and surface any warnings/errors in the Converted detail panel.
