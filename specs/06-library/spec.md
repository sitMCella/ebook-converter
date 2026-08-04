# 06 — Library (Source Document Preview + Details)

## Goal

Provide a dedicated screen for inspecting imported PDF documents before conversion. Users can browse their imported files, view metadata, and optionally set per-document conversion overrides.

## Background

The Import screen (spec 03) handles bringing PDF files into the application. Once imported, users need a way to inspect individual documents — checking metadata, previewing pages, and configuring per-document conversion settings before converting. The Library screen fills this gap, serving as the middle step between import and conversion.

The Import list already links to the Library screen (clicking a file name navigates to `/library`), but the screen itself is currently a placeholder.

## Functional Requirements

### FR-1: Document List

A scrollable list panel (260 px wide) on the left side of the screen shows all imported PDFs. Each item displays:
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

### FR-4: Detail Panel — Page Preview (Deferred)

A rendered preview of the selected PDF page. This requires a backend command to render PDF pages as images, which is not yet implemented. The UI shows a placeholder indicating preview is not yet available.

### FR-5: Detail Panel — Conversion Options (Per-Document Overrides)

A collapsible section showing overridable conversion settings. When expanded, it displays a subset of the global settings with per-document override controls:

**Top-level overrides** (always visible when expanded):
- Split chapters by (dropdown)
- Heading level threshold (number)
- Base font size (number)
- Image quality (dropdown)
- Page range (dropdown)

Each setting shows the global default value with "(default)" label. When changed, the override is stored on the document. A reset button clears individual overrides.

The collapsed header shows an override count when overrides exist: "Conversion options · N custom".

### FR-6: Detail Panel — Convert Button

A full-width primary button "Convert to EPUB" below the conversion options. Disabled when the document is already Converting. Changes to "Reconvert to EPUB" (secondary style) when already Converted.

### FR-7: Empty State

When no files have been imported, the screen shows: "Your library is empty. Import some PDFs to get started." with a "Go to Import" button.

### FR-8: Navigation from Import

Clicking a file name in the Import list navigates to the Library screen with that document pre-selected.

### FR-9: Library Persistence

The library persists across application restarts. When a PDF is imported, its metadata is saved to a `metadata.json` file alongside the stored PDF in `<app_data>/books/<uuid>/`. On application startup, the `ImportProvider` loads all persisted book metadata and populates the file Map, so the Library screen shows previously imported books without re-importing.

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

- Page-by-page PDF rendering in the preview area (requires backend command).
- Drag-and-drop reordering of the document list.
