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
- [x] Call `saveBookMetadata()` in `useImport` after successful import

## Phase 2: Components

### T2: Create Library Screen Shell
- [ ] Create `src/components/library/LibraryScreen.jsx` with header and two-panel layout
- [ ] Add search input in header (right-aligned, 180px)
- [ ] Implement search filtering (case-insensitive substring on file name)
- [ ] Read initial selected document from React Router location state
- [ ] Auto-select first document when none is selected
- [ ] Show empty state when no files are imported

### T3: Create Document List
- [ ] Create `src/components/library/DocumentList.jsx` (260px fixed width, scrollable)
- [ ] Create `src/components/library/DocumentListItem.jsx` with name and size
- [ ] Implement selected state styling (accent background, bold text)
- [ ] Implement hover state styling

### T4: Create Detail Panel
- [ ] Create `src/components/library/DetailPanel.jsx` as the right panel container
- [ ] Create `src/components/library/PagePreview.jsx` with placeholder UI
- [ ] Create `src/components/library/MetadataSection.jsx` with labelled properties
- [ ] Hide metadata rows when values are absent
- [ ] Create `src/components/library/ConversionOptions.jsx` with collapsible section
- [ ] Implement top-level override controls (split chapters, heading threshold, font size, image quality, page range)
- [ ] Show "(default)" label for inherited values, reset button for overridden values
- [ ] Show override count on collapsed header

### T5: Convert Button
- [ ] Add full-width convert button below conversion options
- [ ] Disable when status is `converting`
- [ ] Change to "Reconvert to EPUB" (secondary) when status is `converted`
- [ ] Wire to `startConversion()` and navigate to `/converting`

## Phase 3: Wiring

### T6: Route and Navigation
- [ ] Replace Library placeholder in `src/App.jsx` with `LibraryScreen`
- [ ] Update `ImportListRow` to pass file path via route state when navigating

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
- [x] Test `LOAD_LIBRARY` loads multiple books
- [x] Test startup loading populates library on mount
- [x] Test startup loading handles `listBooks` failure gracefully
- [x] Test `saveBookMetadata` is called after import
- [x] Test `saveBookMetadata` is skipped when `bookId` is null
- [x] Rust: test `write_book_metadata` and `read_all_book_metadata` round-trip
- [x] Rust: test `read_all_book_metadata` returns empty for nonexistent dir
- [x] Rust: test `read_all_book_metadata` skips dirs without `metadata.json`
- [x] Rust: test `read_all_book_metadata` skips invalid JSON
- [x] Rust: test `write_book_metadata` rejects invalid book IDs
- [x] Rust: test `read_all_book_metadata` reads multiple books

### T8: E2E Tests
- [ ] Test navigating from Import to Library pre-selects the document
- [ ] Test selecting different documents updates the detail panel
- [ ] Test search filters the document list
- [ ] Test conversion options overrides persist when switching documents

## Acceptance Criteria

- [x] `/library` route renders the Library screen (not a placeholder)
- [x] All imported PDFs appear in the document list
- [x] Clicking a document shows its metadata in the detail panel
- [x] Search filters the document list by name in real time
- [x] Clicking a file name in the Import list navigates to Library with that file selected
- [x] Empty state shown when no files are imported, with a working "Go to Import" link
- [x] Per-document conversion overrides can be set and show override count
- [x] Convert button starts conversion for the selected document
- [x] Library persists across app restarts (books loaded from storage on startup)
