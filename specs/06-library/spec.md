# 06 — Library (Source Document Preview + Details)

## Goal

Provide a dedicated screen for inspecting imported PDF documents before conversion. Users can browse their imported files, view metadata, configure per-document conversion overrides, and initiate conversion.

## Background

The Import screen (spec 03) provides a staging area where users browse or drag-and-drop PDFs, validate them, and then explicitly import them to the library via "Import to library". Once in the library, users can inspect individual documents — checking metadata, previewing pages, and configuring per-document conversion settings before converting. The Library screen serves as the hub between import and conversion.

## Functional Requirements

### FR-1: Document List

A scrollable list panel (260 px wide) on the left side of the screen shows all imported PDFs (from `state.files`, the library Map). Each item displays:
- File name (truncated with ellipsis if needed).
- File size (muted, 12 px).

Clicking a document selects it and populates the detail panel on the right.

### FR-2: Document Search

A search input in the header filters the document list by file name in real time (case-insensitive substring match). Placeholder: "Search documents...".

### FR-3: Detail Panel — Metadata

When a document is selected, the right panel shows a labelled list of document properties extracted from PDF metadata:

| Label | Source |
|---|---|
| Title | PDF `/Title` metadata |
| Authors | PDF `/Author` metadata |
| Pages | Page count |
| File size | File system |
| Format | PDF version header |
| Created | PDF `/CreationDate` |
| Modified | PDF `/ModDate` |
| Producer | PDF `/Producer` |

If a metadata field is absent, the row is hidden (not shown as "Unknown").

### FR-4: Detail Panel — Cover Page Preview

The page preview section displays the cover image extracted from the first page of the selected PDF. The Rust backend extracts the largest embedded image from page 1 of the stored PDF, encodes it as a base64 data URI, and returns it to the frontend via the `get_pdf_cover` IPC command.

- When a cover image is found, it is displayed centred within a bordered container with a light background, along with the page count below.
- When no embedded image is found on page 1, a fallback placeholder is shown: a file icon with "No cover image available" text and the page count.
- A loading spinner is shown while the cover image is being extracted.
- The preview updates when a different document is selected in the document list.

Note: This extracts embedded images (XObject images) from page 1, not a full page render. PDFs without embedded images on page 1 (e.g. text-only first pages) will show the fallback placeholder.

### FR-5: Detail Panel — Conversion Options (Per-Document Overrides)

A collapsible section showing overridable conversion settings. When expanded, it displays a subset of the global settings with per-document override controls:

**Top-level overrides** (always visible when expanded):
- Heading level threshold (number)
- Base font size (number)
- Image quality (dropdown)
- Page range (dropdown)

Each setting shows the global default value with "(default)" label. When changed, the override is stored on the document. A reset button clears individual overrides.

The collapsed header shows an override count when overrides exist: "Conversion options · N custom".

### FR-6: Detail Panel — Action Buttons

A full-width primary button "Convert to EPUB" below the conversion options. Disabled when the document is already Converting. When the document is already Converted, two buttons appear:
- "View EPUB" (primary) — navigates to the Converted screen with the corresponding EPUB selected.
- "Reconvert to EPUB" (secondary) — starts a new conversion.

Conversion is initiated from the library, not from the import staging area.

### FR-7: Empty State

When no files have been imported to the library, the screen shows: "Your library is empty. Import some PDFs to get started." with a "Go to Import" button.

### FR-8: Library Persistence

The library persists across application restarts. When a PDF is imported to the library (via "Import to library" on the Import screen), its metadata is saved to a `metadata.json` file alongside the stored PDF in `<app_data>/books/<uuid>/`. On application startup, the `ImportProvider` loads all persisted book metadata and populates the `files` Map, so the Library screen shows previously imported books without re-importing.

## Non-Functional Requirements

### NFR-1: Responsive Layout

The two-panel layout fills the available space. The document list has a fixed width (260 px), and the detail panel fills the remaining width. Both panels scroll independently.

### NFR-2: Keyboard Navigation

- Arrow keys to navigate the document list when it has focus.
- Tab/Shift+Tab to move between interactive elements.

### NFR-3: Accessibility

- Selected document state is conveyed via `aria-selected`.
- Search input has an appropriate label.
- All interactive elements are keyboard navigable.

## Out of Scope

- Full page-by-page PDF rendering with page navigation (the current preview extracts the cover image from page 1 only).
- Drag-and-drop reordering of the document list.
- Multi-select batch conversion from library (conversion is single-file from the detail panel).
