# 03 — Import PDF

## Goal

Allow users to bring PDF files into the application via drag-and-drop or a native file picker, validate them, extract metadata, and manage an import list — the first step of the conversion pipeline.

## Background

The application shell (spec 01) provides Tauri's native file dialog and filesystem plugins. The UI/UX design (spec 02) defines the Import screen layout: a drop zone, a "Browse files" button, an import list with status badges, and batch actions. This spec covers everything needed to make that screen functional, including the Rust backend for PDF validation and metadata extraction.

## Functional Requirements

### FR-1: Browse Files via Native Dialog

The "Browse files" button opens Tauri's native file picker dialog filtered to `*.pdf`. The user can select one or more files. Selected files are added to the import list with status `Ready`. The global keyboard shortcut `Cmd/Ctrl + O` also triggers this dialog.

### FR-2: Drag-and-Drop Import

Users can drag PDF files from the OS file manager onto the drop zone. The drop zone provides visual feedback on drag-over (accent border, tinted background, subtle 1.01× scale). Non-PDF files are silently ignored. Multiple files can be dropped at once.

### FR-3: Duplicate Detection

If a file with the same absolute path is already in the import list, the duplicate is skipped and a toast notification is shown: "File already imported" (auto-dismiss after 3 seconds).

### FR-4: PDF Validation

When a file is added to the import list, the Rust backend validates it:
- **Corrupted/unreadable files**: the import list row shows an `Error` badge. Clicking the row shows: "This file could not be read. It may be corrupted or password-protected."
- **Password-protected files**: the import list row shows an `Error` badge with message: "This file is password-protected. Encrypted PDFs are not supported."
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

### FR-6: Import List

The import list displays all imported files with:
- Checkbox for batch selection
- PDF icon (accent colour)
- File name (truncated with ellipsis)
- File size (formatted per spec 02 rules)
- Status badge: `Ready`, `Converting`, `Converted`, or `Error`

The list is scrollable when it exceeds ~300 px of visible height. When empty, it shows: "No files imported yet."

### FR-7: Batch Actions

Below the import list, right-aligned:
- **"Remove selected"** — secondary button. Removes checked files from the list. Shows a confirmation dialog: "Remove N file(s) from the import list? The source PDFs on disk are not affected." Disabled when no rows are checked.
- **"Convert selected"** — primary button with `transform` icon. Starts conversion for all checked files with status `Ready`. Disabled when no convertible rows are checked. (Actual conversion logic is out of scope for this spec; the button triggers navigation to the Converting screen.)

### FR-8: File Name Navigation

Clicking a file name in the import list navigates to the Library screen (screen 2) with that document selected. (Library screen implementation is out of scope for this spec; wiring the navigation event is in scope.)

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

The import list is held in React state during the session. It is not persisted to disk between application restarts. (A future spec may add persistence via the Tauri store plugin.)

## Out of Scope

- PDF page rendering / preview (Library screen, spec 04+)
- Actual PDF-to-EPUB conversion (Converting screen, spec 05+)
- Persisting the import list across sessions
- Import of non-PDF formats (future consideration)
