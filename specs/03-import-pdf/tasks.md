# 03 — Import PDF: Tasks

## Phase 1: Rust Backend — PDF Service

### T1: Add lopdf Dependency
- [x] Add `lopdf = "0.34"` to `src-tauri/Cargo.toml` dependencies
- [x] Run `cargo check` in `src-tauri/` to verify the crate resolves

### T2: Implement PDF Validation Command
- [x] Create `src-tauri/src/pdf.rs` module
- [x] Implement `validate_pdf(path: String) -> Result<PdfValidation, String>`
- [x] Define the `PdfValidation` enum with serde serialization (tagged union)
- [x] Register the command in `src-tauri/src/lib.rs`

### T3: Implement Metadata Extraction Command
- [x] Implement `get_pdf_metadata(path: String) -> Result<PdfMetadata, String>`
- [x] Define the `PdfMetadata` struct with serde serialization (camelCase)
- [x] Register the command in `src-tauri/src/lib.rs`

### T4: Unit Tests for PDF Service
- [x] Add test PDFs to `src-tauri/tests/fixtures/`
- [x] Test `validate_pdf` with valid, encrypted, and corrupted PDFs
- [x] Test `get_pdf_metadata` extracts correct metadata fields

## Phase 2: Frontend Foundation

### T5: Install Frontend Dependencies
- [x] Install `react-router-dom`, `sonner`, `lucide-react`

### T6: Application Shell Layout
- [x] Create `AppShell.jsx`, `Sidebar.jsx`
- [x] Set up `react-router-dom` routes in `App.jsx`
- [x] Configure sidebar with four navigation items

### T7: Toast Notification Setup
- [x] Add `<Toaster />` from sonner to the app root
- [x] Configure position, stacking, and auto-dismiss

### T8: Shared UI Components
- [x] Create `Button.jsx`, `Checkbox.jsx`, `ConfirmDialog.jsx`, `StatusBadge.jsx`

## Phase 3: Import State Management

### T9: Import Context with Staging
- [x] Create `src/contexts/ImportContext.jsx` with dual Maps:
  - `stagedFiles` (Map) — files in the staging area
  - `files` (Map) — files in the library
  - `selectedPaths` (Set) — UI checkbox state
- [x] Staging reducer actions: `STAGE_FILES`, `UNSTAGE_FILES`, `UPDATE_STAGED_STATUS`, `SET_STAGED_METADATA`
- [x] Library reducer actions: `ADD_FILES`, `REMOVE_FILES`, `UPDATE_STATUS`, `SET_METADATA`, `LOAD_LIBRARY`
- [x] Bridge action: `IMPORT_TO_LIBRARY` — moves a staged file to the library with bookId and storedPdfPath
- [x] Selection actions: `TOGGLE_SELECTION`, `SELECT_ALL`, `DESELECT_ALL` (operate on staged files)
- [x] Add the provider to `App.jsx`

### T10: File Size Formatting Utility
- [x] Create `src/lib/format.js` with `formatFileSize(bytes)` per spec 02 rules

## Phase 4: Tauri Bridge Extensions

### T11: Extend Tauri Bridge for PDF Import
- [x] Add `openPdfFiles()`, `validatePdf()`, `getPdfMetadata()`, `getFileSize()`
- [x] Add `importPdf(sourcePath)` — copies PDF to managed storage, returns bookId and storedPdfPath
- [x] Add `saveBookMetadata(metadata)` — persists metadata to the book's storage directory
- [x] Browser fallback: `importPdf` returns `{ bookId: null, storedPdfPath: sourcePath }`

## Phase 5: Import Screen UI

### T12: Drop Zone Component
- [x] Create `DropZone.jsx` with dashed border, drag-over visual states
- [x] Make focusable and activatable via Enter/Space

### T13: Drag-and-Drop Hook
- [x] Create `useDragDrop.js` — listens to Tauri drag-drop events
- [x] Wire into Import screen

### T14: Import Orchestration Hook
- [x] Create `useImport.js` with two functions:
  - `stageFiles(paths)` — duplicate check (against both staged and library files), validate, extract metadata. Dispatches: `STAGE_FILES`, `SET_STAGED_METADATA`, `UPDATE_STAGED_STATUS`. Does NOT call `importPdf` or `saveBookMetadata`.
  - `importStagedFiles(paths)` — for each staged file with status `ready`: calls `importPdf`, `saveBookMetadata`, dispatches `IMPORT_TO_LIBRARY` to move from staged to library.
- [x] Returns `{ stageFiles, importStagedFiles, isProcessing }`

### T15: Staging List Component
- [x] Create `ImportList.jsx` — reads from `state.stagedFiles` (not `state.files`)
  - "Ready to import" header label
  - Empty state: "No files staged yet."
- [x] Create `ImportListRow.jsx` — file name as plain text (no navigation link)
  - Checkbox, PDF icon, file name, file size, status badge

### T16: Batch Actions Component
- [x] Create `BatchActions.jsx`:
  - "Remove selected" secondary button — non-destructive unstaging (dispatches `UNSTAGE_FILES`, no confirmation dialog)
  - "Import to library" primary button with `BookPlus` icon — calls `importStagedFiles` for checked files with status `ready`
  - Disabled states when no applicable rows are checked

### T17: Import Screen Assembly
- [x] Create `ImportScreen.jsx`:
  - Header: title "Import PDF files" + "Browse files" button
  - Drop zone (wired to `stageFiles`)
  - Staging list
  - Batch actions
- [x] Wire `Cmd/Ctrl + O` keyboard shortcut

## Phase 6: Styling and Polish

### T18: Dark Mode Support
- [x] CSS custom properties for light and dark themes
- [x] `@media (prefers-color-scheme: dark)` for automatic switching

### T19: Accessibility Audit
- [x] Keyboard navigation, focus trapping, aria-labels

### T20: Error State Handling
- [x] Error badges for corrupted and password-protected PDFs

## Acceptance Criteria

- [x] Clicking "Browse files" opens the native file dialog filtered to `.pdf`, and selected files appear in the staging list with status `Ready`
- [x] Dragging PDF files onto the drop zone adds them to the staging list; non-PDF files are ignored
- [x] Dropping a duplicate file (already staged or in library) shows a toast and does not add a second entry
- [x] A corrupted or unreadable PDF shows an `Error` badge with an appropriate message
- [x] A password-protected PDF shows an `Error` badge with "password-protected" message
- [x] Valid PDFs display extracted metadata (page count, title, author if present)
- [x] "Remove selected" removes staged files non-destructively (no confirmation dialog, no storage deletion)
- [x] "Import to library" copies files to managed storage, persists metadata, and moves them to the library
- [x] After import, files no longer appear in the staging list but are visible in the library
- [x] The sidebar highlights "Import" as the active navigation item
- [x] `Cmd/Ctrl + O` triggers the file dialog from any screen
- [x] The UI is keyboard-accessible
- [x] Light and dark modes render correctly following OS preference
