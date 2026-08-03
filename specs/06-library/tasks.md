# 06 — Library: Tasks

## Phase 1: State Changes

### T1: Add Per-Document Overrides to ImportContext
- [ ] Add `SET_DOCUMENT_OVERRIDES` action to the import reducer
- [ ] Store `overrides` as a partial settings object on the file entry
- [ ] Verify existing reducer actions are unaffected

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
- [ ] Create `src/components/library/DocumentListItem.jsx` with name, size, status badge
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
- [ ] Test LibraryScreen empty state renders correctly
- [ ] Test LibraryScreen renders document list and detail panel
- [ ] Test search filtering reduces document list
- [ ] Test DocumentListItem selected and hover states
- [ ] Test MetadataSection shows available fields, hides absent ones
- [ ] Test ConversionOptions collapse/expand and override count
- [ ] Test convert button states (ready, converting, converted)

### T8: E2E Tests
- [ ] Test navigating from Import to Library pre-selects the document
- [ ] Test selecting different documents updates the detail panel
- [ ] Test search filters the document list
- [ ] Test conversion options overrides persist when switching documents

## Acceptance Criteria

- [ ] `/library` route renders the Library screen (not a placeholder)
- [ ] All imported PDFs appear in the document list
- [ ] Clicking a document shows its metadata in the detail panel
- [ ] Search filters the document list by name in real time
- [ ] Clicking a file name in the Import list navigates to Library with that file selected
- [ ] Empty state shown when no files are imported, with a working "Go to Import" link
- [ ] Per-document conversion overrides can be set and show override count
- [ ] Convert button starts conversion for the selected document
