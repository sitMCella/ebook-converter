# 06 — Library: Tasks

## Phase 1: State Changes

### T1: Add Per-Document Overrides to ImportContext
- [x] Add `SET_DOCUMENT_OVERRIDES` action to the import reducer
- [x] Store `overrides` as a partial settings object on the file entry
- [x] Verify existing reducer actions are unaffected

### T1b: Add Library Persistence
- [x] Add `BookMetadata` struct to Rust `storage.rs`
- [x] Add `save_book_metadata` Rust command (writes `metadata.json` to book dir)
- [x] Add `list_books` Rust command (scans book dirs for `metadata.json` files)
- [x] Register new commands in `lib.rs`
- [x] Add `saveBookMetadata()` and `listBooks()` bridge functions in `tauri.js`
- [x] Add `LOAD_LIBRARY` reducer action to ImportContext
- [x] Call `listBooks()` in `ImportProvider` `useEffect` on mount
- [x] Call `saveBookMetadata()` in `useImport` after import-to-library (not during staging)

## Phase 2: Components

### T2: Create Library Screen Shell
- [x] Create `src/components/library/LibraryScreen.jsx` with header and two-panel layout
- [x] Add search input in header (right-aligned, 180px)
- [x] Implement search filtering (case-insensitive substring on file name)
- [x] Auto-select first document when none is selected
- [x] Show empty state when no files are imported

### T3: Create Document List
- [x] Create `src/components/library/DocumentList.jsx` (260px fixed width, scrollable)
- [x] Create `src/components/library/DocumentListItem.jsx` with name and size
- [x] Implement selected state styling (accent background, bold text)
- [x] Implement hover state styling

### T4: Create Detail Panel
- [x] Create `src/components/library/DetailPanel.jsx` as the right panel container
- [x] Create `src/components/library/PagePreview.jsx` with cover image extraction
  - [x] Add `get_pdf_cover` Rust command (reuses `extract_cover_image` from conversion pipeline)
  - [x] Register `get_pdf_cover` in `lib.rs` invoke handler
  - [x] Add `getPdfCover()` bridge function in `tauri.js`
  - [x] Implement async cover loading with loading spinner, image display, and fallback states
- [x] Create `src/components/library/MetadataSection.jsx` with labelled properties
- [x] Hide metadata rows when values are absent
- [x] Create `src/components/library/ConversionOptions.jsx` with collapsible section
- [x] Implement top-level override controls
- [x] Show "(default)" label for inherited values, reset button for overridden values
- [x] Show override count on collapsed header

### T5: Convert Button
- [x] Add full-width convert button below conversion options
- [x] Disable when status is `converting`
- [x] Change to "Reconvert to EPUB" (secondary) when status is `converted`
- [x] Wire to `startConversion()` and navigate to `/converting`

## Phase 3: Wiring

### T6: Route and Navigation
- [x] Replace Library placeholder in `src/App.jsx` with `LibraryScreen`
- [x] Library reads from `state.files` (library Map, not staging)

## Phase 4: Tests

### T7: Unit Tests
- [x] Test LibraryScreen empty state renders correctly
- [x] Test LibraryScreen renders document list and detail panel
- [x] Test search filtering reduces document list
- [x] Test DocumentListItem selected and hover states
- [x] Test MetadataSection shows available fields, hides absent ones
- [x] Test ConversionOptions collapse/expand and override count
- [x] Test convert button states (ready, converting, converted)

### T7b: Persistence Unit Tests
- [x] Test `LOAD_LIBRARY` reducer action loads books into file Map
- [x] Test `LOAD_LIBRARY` does not overwrite existing files
- [x] Test startup loading populates library on mount
- [x] Test `saveBookMetadata` is called during import-to-library (not during staging)
- [x] Test `saveBookMetadata` is skipped when `bookId` is null
- [x] Rust: test persistence round-trips

### T8: E2E Tests
- [x] Test files appear in library after "Import to library"
- [x] Test selecting different documents updates the detail panel
- [x] Test search filters the document list
- [x] Test conversion options overrides persist when switching documents
- [x] Test empty state when no files imported

## Acceptance Criteria

- [x] `/library` route renders the Library screen (not a placeholder)
- [x] All imported PDFs appear in the document list (only after "Import to library", not during staging)
- [x] Clicking a document shows its metadata in the detail panel
- [x] Search filters the document list by name in real time
- [x] Empty state shown when no files are imported, with a working "Go to Import" link
- [x] Per-document conversion overrides can be set and show override count
- [x] Convert button starts conversion for the selected document
- [x] Library persists across app restarts (books loaded from storage on startup)
