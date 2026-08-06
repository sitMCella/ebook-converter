# 07 — Converted (EPUB Library + Preview)

## Goal

Provide a dedicated screen for browsing, inspecting, and managing converted EPUB files. Users can see all successfully converted documents, view EPUB-specific metadata, and take actions like opening in an external reader, saving a copy, or reconverting with different settings.

## Background

The conversion pipeline (spec 04) produces EPUB files from imported PDFs. Once conversion completes, the file's status changes to `converted` and a `conversionResult` object is stored on the file entry in `ImportContext` (containing `outputPath`, `images`, `fileSize`). The conversion result is also persisted to the book's `metadata.json` on disk so that converted files survive app restarts. On startup, `LOAD_LIBRARY` restores the conversion result data for books with `status === 'converted'`. The Converting screen (spec 04) already links completed files to `/converted`, but that route currently renders a placeholder. This spec replaces the placeholder with a full Converted screen following the UI/UX design spec (Screen 4).

The Converted screen mirrors the Library screen (spec 06) layout — a two-panel master-detail view — but shows EPUB-specific information instead of PDF source metadata.

## Functional Requirements

### FR-1: EPUB List

A scrollable list panel (260 px wide) on the left side of the screen shows all files with status `converted`. Each item displays:
- File name with `.epub` extension (derived from source PDF name).
- EPUB file size (from `conversionResult.fileSize`, muted, 12 px).
- Conversion date (muted, 12 px).

Clicking an EPUB selects it and populates the detail panel on the right.

### FR-2: Search

A search input in the header filters the EPUB list by file name in real time (case-insensitive substring match). Placeholder: "Search converted...".

### FR-3: Detail Panel — EPUB Preview (Deferred)

A rendered preview of EPUB content. This requires parsing EPUB HTML content and rendering it in a sandboxed container, which is complex and out of scope for the initial implementation. The UI shows the cover image if available, or a "No cover image available" placeholder otherwise.

### FR-4: Detail Panel — Metadata

When an EPUB is selected, the detail panel shows a labelled list of EPUB-specific properties:

| Label | Source | Example |
|---|---|---|
| Source | Original PDF file name | Design patterns.pdf |
| EPUB size | `conversionResult.fileSize` | 3.1 MB |
| Images | `conversionResult.images` | 47 extracted |
| Converted | Conversion timestamp | 2026-08-02 14:32 |

If a metadata field is absent or zero, the row is hidden.

### FR-5: Detail Panel — Action Buttons

Stacked vertically, full width, below the metadata:

1. **"Open in reader"** — primary button, `ExternalLink` icon. Opens the EPUB file in the system's default EPUB reader application via a Tauri shell command. In browser mode, triggers a download.
2. **"Reconvert"** — secondary button, `RefreshCw` icon. Navigates to the Library screen with the source PDF selected and the conversion options panel expanded.

### FR-6: Header — Open Folder

The header contains a "Open folder" secondary button with a `FolderOpen` icon. Opens the output folder in the OS file manager. In browser mode, this button is hidden.

### FR-7: Empty State

When no files have been converted, the screen shows: "No converted files yet. Import and convert a PDF to see it here." with a "Go to Import" button that navigates to `/import`.

### FR-8: Persistence Across Restarts

Converted EPUB data persists across app restarts. When conversion completes, `useConversion` updates the book's `metadata.json` via `saveBookMetadata` with `status: 'converted'` and the conversion result fields (`outputPath`, `images`, `epubFileSize`). On startup, `listBooks()` loads these metadata files and the `LOAD_LIBRARY` reducer restores `outputPath` and `conversionResult` on the file entry, so the Converted screen displays previously converted files without requiring reconversion.

### FR-9: Navigation from Converting Screen

Clicking a completed row in the Converting screen navigates to `/converted` with the EPUB pre-selected via React Router location state (`{ selectedPath }`).

### FR-10: Navigation from Sidebar

The "Converted" sidebar item navigates to `/converted`. The active state styling matches other sidebar items (accent background, font-weight 500).

## Non-Functional Requirements

### NFR-1: Responsive Layout

The two-panel layout fills the available space. The EPUB list has a fixed width (260 px), and the detail panel fills the remaining width. Both panels scroll independently.

### NFR-2: Keyboard Navigation

- Arrow keys to navigate the EPUB list when it has focus.
- Tab/Shift+Tab to move between interactive elements.

### NFR-3: Accessibility

- Selected EPUB state is conveyed via `aria-selected`.
- Search input has an appropriate label.
- All interactive elements are keyboard navigable.
- Action buttons have descriptive labels.

### NFR-4: Design Consistency

The screen follows the same visual patterns as the Library screen (spec 06):
- Same header style (h3, 18 px, weight 500).
- Same two-panel layout dimensions and spacing.
- Same list item styling patterns.
- Same metadata label/value formatting.
- Uses CSS custom property tokens from `index.css`, no hardcoded colours.

## Out of Scope

- EPUB content rendering in the preview area (requires EPUB HTML parsing and sandboxed rendering).
- Drag-and-drop reordering of the EPUB list.
- EPUB validation (epubcheck integration).
- Conversion history / reconversion tracking.
- Batch actions on converted files (delete, export multiple).
