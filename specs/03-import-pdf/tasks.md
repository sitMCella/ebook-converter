# 03 — Import PDF: Tasks

## Phase 1: Rust Backend — PDF Service

### T1: Add lopdf Dependency
- [ ] Add `lopdf = "0.34"` to `src-tauri/Cargo.toml` dependencies
- [ ] Run `cargo check` in `src-tauri/` to verify the crate resolves

### T2: Implement PDF Validation Command
- [ ] Create `src-tauri/src/pdf.rs` module
- [ ] Implement `validate_pdf(path: String) -> Result<PdfValidation, String>`:
  - Attempt to load the PDF with `lopdf::Document::load()`
  - On encryption error → return `PdfValidation::Encrypted`
  - On other load error → return `PdfValidation::Error { message }`
  - On success → return `PdfValidation::Valid`
- [ ] Define the `PdfValidation` enum with serde serialization (tagged union)
- [ ] Register the command in `src-tauri/src/lib.rs` via `.invoke_handler(tauri::generate_handler![...])`

### T3: Implement Metadata Extraction Command
- [ ] Implement `get_pdf_metadata(path: String) -> Result<PdfMetadata, String>`:
  - Load the PDF with `lopdf::Document::load()`
  - Extract page count from `doc.get_pages().len()`
  - Extract PDF version from `doc.version`
  - Read the Info dictionary for Title, Author, CreationDate, ModDate, Producer
  - Parse PDF date strings (`D:YYYYMMDDHHmmSS`) to ISO 8601
  - Get file size via `std::fs::metadata(path)?.len()`
- [ ] Define the `PdfMetadata` struct with serde serialization (camelCase)
- [ ] Register the command in `src-tauri/src/lib.rs`

### T4: Unit Tests for PDF Service
- [ ] Add test PDFs to `src-tauri/tests/fixtures/` (valid, encrypted, corrupted)
- [ ] Test `validate_pdf` with a valid PDF → returns `Valid`
- [ ] Test `validate_pdf` with an encrypted PDF → returns `Encrypted`
- [ ] Test `validate_pdf` with a corrupted file → returns `Error`
- [ ] Test `get_pdf_metadata` extracts correct page count, version, and metadata fields
- [ ] Test `get_pdf_metadata` with missing metadata fields → returns `None` for absent fields

## Phase 2: Frontend Foundation

### T5: Install Frontend Dependencies
- [ ] Install `react-router-dom`, `sonner`, `lucide-react`
- [ ] Verify all dependencies resolve and the dev server starts

### T6: Application Shell Layout
- [ ] Create `src/components/layout/AppShell.jsx` — sidebar + main content area per spec 02 layout
- [ ] Create `src/components/layout/Sidebar.jsx` — four navigation items (Import, Library, Converted, Settings) with icons from `lucide-react`
  - Active state: highlighted background, accent text, weight 500
  - Hover state: subtle background tint
  - Section divider before Settings ("Tools")
- [ ] Set up `react-router-dom` routes in `App.jsx`:
  - `/` and `/import` → Import screen
  - `/library` → placeholder
  - `/converted` → placeholder
  - `/settings` → placeholder
- [ ] Configure the sidebar with fixed 200 px width per spec 02
- [ ] Update window configuration in `tauri.conf.json`: minimum size to 960×640, default size to 1200×800 per spec 02

### T7: Toast Notification Setup
- [ ] Add `<Toaster />` from sonner to the app root
- [ ] Configure: position bottom-center, max 3 visible, auto-dismiss 3 seconds, manual dismiss button
- [ ] Style toasts to match spec 02 colour tokens (success, info, warning, error)

### T8: Shared UI Components
- [ ] Create `src/components/ui/Button.jsx` — primary and secondary variants per spec 02 visual specs (8 px radius, accent fill for primary, transparent with border for secondary)
- [ ] Create `src/components/ui/Checkbox.jsx` — 16×16 px, 4 px border radius
- [ ] Create `src/components/ui/ConfirmDialog.jsx` — modal with title, message, cancel, and confirm buttons
- [ ] Create `src/components/import/StatusBadge.jsx` — pill-shaped badge (99 px radius) with colour variants: Ready (accent), Converting (warning), Converted (success), Error (danger)

## Phase 3: Import State Management

### T9: Import Context
- [ ] Create `src/contexts/ImportContext.jsx`:
  - `ImportState` with `files` (Map) and `selectedPaths` (Set)
  - Reducer handling: `ADD_FILES`, `REMOVE_FILES`, `UPDATE_STATUS`, `SET_METADATA`, `TOGGLE_SELECTION`, `SELECT_ALL`, `DESELECT_ALL`
  - Provider component wrapping the app
- [ ] Add the provider to `App.jsx`

### T10: File Size Formatting Utility
- [ ] Create `src/lib/format.js` with `formatFileSize(bytes)` per spec 02 rules
- [ ] Test edge cases: 0 bytes, 1023 bytes, 1024 bytes, 1 MB boundary, 1 GB boundary

## Phase 4: Tauri Bridge Extensions

### T11: Extend Tauri Bridge for PDF Import
- [ ] Add `openPdfFiles()` to `src/lib/tauri.js`:
  - Opens native dialog with `multiple: true` and PDF filter
  - Returns `string[]` (array of paths) or `null` on cancel
  - Browser fallback: use `<input type="file" multiple accept=".pdf">`
- [ ] Add `validatePdf(path)` — calls `invoke('validate_pdf', { path })`
- [ ] Add `getPdfMetadata(path)` — calls `invoke('get_pdf_metadata', { path })`
- [ ] Add `getFileSize(path)` — calls `stat()` from `@tauri-apps/plugin-fs` and returns `size`
- [ ] Add `fs:allow-stat` to `src-tauri/capabilities/default.json`

## Phase 5: Import Screen UI

### T12: Drop Zone Component
- [ ] Create `src/components/import/DropZone.jsx`:
  - Dashed border (1.5 px, strong border colour), 12 px border-radius
  - Centred content: `cloud-upload` icon (32 px), primary text, secondary hint text
  - Full width of main content area
- [ ] Implement drag-over visual state: accent border, tinted background, 1.01× scale
- [ ] Implement drag-leave state: revert to default
- [ ] Use HTML5 drag events (`onDragEnter`, `onDragOver`, `onDragLeave`) for visual feedback
- [ ] Make the drop zone focusable and activatable via Enter/Space (triggers file dialog)

### T13: Drag-and-Drop Hook
- [ ] Create `src/hooks/useDragDrop.js`:
  - Listen to Tauri's `onDragDropEvent` for file paths on `drop`
  - Filter dropped paths to `.pdf` extension (case-insensitive)
  - Return the filtered paths to the caller
  - Clean up the listener on unmount
- [ ] Wire the hook into the Import screen

### T14: Import Orchestration Hook
- [ ] Create `src/hooks/useImport.js`:
  - `importFiles(paths: string[])` function:
    1. Filter out duplicates (paths already in the import list)
    2. Show toast for skipped duplicates
    3. Add new files to state with status `ready` and `metadata: null`
    4. For each new file, call `validatePdf(path)` asynchronously
    5. On validation failure → update status to `error` with message
    6. On validation success → call `getPdfMetadata(path)` and update state
  - Returns `{ importFiles, isImporting }`

### T15: Import List Component
- [ ] Create `src/components/import/ImportList.jsx`:
  - "Recent imports" muted label header
  - Scrollable container (max ~300 px visible height)
  - Empty state: "No files imported yet." with muted text
- [ ] Create `src/components/import/ImportListRow.jsx`:
  - Checkbox (16×16 px)
  - PDF icon (accent colour, from `lucide-react`)
  - File name (weight 500, truncated with ellipsis)
  - File size (muted, 12 px, formatted via `formatFileSize`)
  - Status badge
  - 0.5 px bottom border separator
  - Click on file name → navigate to `/library` (wiring only, Library screen is a placeholder)

### T16: Batch Actions Component
- [ ] Create `src/components/import/BatchActions.jsx`:
  - "Remove selected" secondary button — disabled when no rows checked
  - "Convert selected" primary button with `transform` icon — disabled when no checked rows have status `ready`
  - Right-aligned layout
- [ ] Wire "Remove selected":
  - Show `ConfirmDialog`: "Remove N file(s) from the import list? The source PDFs on disk are not affected."
  - On confirm → dispatch `REMOVE_FILES`
- [ ] Wire "Convert selected":
  - Collect checked file paths with status `ready`
  - Navigate to `/converted` (placeholder — actual conversion is a future spec)

### T17: Import Screen Assembly
- [ ] Create `src/components/import/ImportScreen.jsx`:
  - Header: title "Import PDF files" (h3, 18 px, weight 500) + "Browse files" button with `folder-open` icon
  - Drop zone
  - Import list
  - Batch actions
- [ ] Wire "Browse files" button to `openPdfFiles()` → `importFiles()`
- [ ] Wire drop zone to drag-and-drop hook → `importFiles()`
- [ ] Wire `Cmd/Ctrl + O` keyboard shortcut to trigger the file dialog

## Phase 6: Styling and Polish

### T18: Dark Mode Support
- [ ] Define CSS custom properties for light and dark themes per spec 02 colour tokens
- [ ] Use `@media (prefers-color-scheme: dark)` for automatic theme switching
- [ ] Verify all components render correctly in both themes

### T19: Accessibility Audit
- [ ] All interactive elements are keyboard navigable via Tab/Shift+Tab
- [ ] Drop zone is focusable and activatable via Enter/Space
- [ ] Status badges have `aria-label` attributes
- [ ] Confirm dialog traps focus and closes on Escape
- [ ] Checkboxes have associated labels

### T20: Error State Handling
- [ ] Corrupted PDF: import list row shows `Error` badge, clicking shows inline error message
- [ ] Password-protected PDF: `Error` badge with "password-protected" message
- [ ] Clicking an error row displays the error message below the row (expandable detail)

## Acceptance Criteria

- [ ] Clicking "Browse files" opens the native file dialog filtered to `.pdf`, and selected files appear in the import list with status `Ready`
- [ ] Dragging PDF files onto the drop zone adds them to the import list; non-PDF files are ignored
- [ ] Dropping a duplicate file shows a "File already imported" toast and does not add a second entry
- [ ] A corrupted or unreadable PDF shows an `Error` badge with an appropriate message
- [ ] A password-protected PDF shows an `Error` badge with "password-protected" message
- [ ] Valid PDFs display extracted metadata (page count, title, author if present)
- [ ] File sizes are formatted correctly (B, KB, MB, GB)
- [ ] "Remove selected" shows a confirmation dialog and removes checked files on confirm
- [ ] "Convert selected" is disabled when no `Ready` files are checked
- [ ] The sidebar highlights "Import" as the active navigation item
- [ ] `Cmd/Ctrl + O` triggers the file dialog from any screen
- [ ] The UI is keyboard-accessible: all controls reachable via Tab, drop zone activatable via Enter/Space
- [ ] Light and dark modes render correctly following OS preference
