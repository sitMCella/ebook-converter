# 03 — Import PDF

## Goal

Allow users to bring PDF files into the application via drag-and-drop or a native file picker, validate them, and manage a staging area — the first step before importing files to the library.

## Background

The application shell (spec 01) provides Tauri's native file dialog and filesystem plugins. The UI/UX design (spec 02) defines the Import screen layout: a drop zone, a "Browse files" button, a staging list with status badges, and batch actions. This spec covers everything needed to make that screen functional, including the Rust backend for PDF validation and metadata extraction.

## Concepts

### Staging vs. Library

The Import screen serves as a **staging area** where users preview and validate PDFs before committing them to the library. Staged files are temporary — they exist only in session memory and are discarded when the app closes. Files are moved from staging to the library via an explicit "Import to library" action, at which point they are copied to managed storage and persisted.

This two-phase approach prevents the import list from accumulating already-imported library files. The library (spec 06) is the authoritative collection of imported books; the staging area is a scratchpad for preparing the next batch.

## Functional Requirements

### FR-1: Browse Files via Native Dialog

The "Browse files" button opens Tauri's native file picker dialog filtered to `*.pdf`. The user can select one or more files. Selected files are added to the staging list with status `Ready`. The global keyboard shortcut `Cmd/Ctrl + O` also triggers this dialog.

### FR-2: Drag-and-Drop Import

Users can drag PDF files from the OS file manager onto the drop zone. The drop zone provides visual feedback on drag-over (accent border, tinted background, subtle 1.01× scale). Non-PDF files are silently ignored. Multiple files can be dropped at once.

### FR-3: Duplicate Detection

If a file with the same path is already staged or already in the library, the duplicate is skipped and a toast notification is shown: "File already imported" (auto-dismiss after 3 seconds).

### FR-4: PDF Validation

When a file is added to the staging list, the Rust backend validates it:
- **Corrupted/unreadable files**: the staging list row shows an `Error` badge. Clicking the row shows: "This file could not be read. It may be corrupted or password-protected."
- **Password-protected files**: the staging list row shows an `Error` badge with message: "This file is password-protected. Encrypted PDFs are not supported."
- **Valid files**: status is set to `Ready`.

### FR-5: Metadata Extraction

For valid PDFs, the Rust backend extracts and returns:
- Title (from PDF `/Title` metadata)
- Author(s) (from PDF `/Author` metadata)
- Page count
- PDF version (from the file header, e.g. "PDF 1.7")
- Creation date (from `/CreationDate`)
- Modification date (from `/ModDate`)
- Producer (from `/Producer`)

Missing metadata fields are omitted (not returned as empty strings). File size is determined on the frontend from the filesystem.

### FR-6: Staging List

The staging list displays files waiting to be imported with:
- Checkbox for batch selection
- PDF icon (accent colour)
- File name (truncated with ellipsis)
- File size (formatted per spec 02 rules)
- Status badge: `Ready` or `Error`

The list is scrollable when it exceeds ~300 px of visible height. When empty, it shows: "No files staged yet." The header label reads "Ready to import".

### FR-7: Batch Actions

Below the staging list, right-aligned:
- **"Remove selected"** — secondary button. Removes checked files from the staging list. This is non-destructive: it simply unstages files without affecting the library or storage. No confirmation dialog is needed. Disabled when no rows are checked.
- **"Import to library"** — primary button with `BookPlus` icon. Imports all checked files with status `Ready` to the library (copying to managed storage and persisting metadata). Disabled when no importable rows are checked. After import, files are removed from the staging list and appear in the library.

### FR-8: File Name Display

File names in the staging list are plain text (not clickable links). Staged files are not yet in the library, so there is no library entry to navigate to.

## Non-Functional Requirements

### NFR-1: Import Speed

Validation and metadata extraction for a single PDF should complete in under 500 ms for files up to 100 MB. The UI remains responsive during import — validation runs asynchronously via Tauri IPC.

### NFR-2: Memory Efficiency

The Rust backend reads only the PDF header and metadata dictionary, not the full file content. Page rendering is handled separately (out of scope for this spec).

### NFR-3: Accessibility

- All interactive elements (drop zone, browse button, checkboxes, batch action buttons) are keyboard navigable.
- The drop zone is focusable and activatable via `Enter`/`Space` (triggers the file dialog).
- Status badges are communicated to screen readers (e.g., via `aria-label`).
- Colour is not the sole indicator of status — badges include text labels.

### NFR-4: State Persistence

The staging list is held in React state during the session. It is not persisted to disk between application restarts. Staged files are discarded when the app closes — only files that have been imported to the library are persisted.

## Out of Scope

- PDF page rendering / preview (Library screen, spec 06)
- Actual PDF-to-EPUB conversion (starts from Library screen, spec 04+)
- Persisting the staging list across sessions
- Import of non-PDF formats (future consideration)
