# 07 — Converted: Tasks

## Phase 1: Components

### T1: Create Converted Screen Shell
- [x] Create `src/components/converted/ConvertedScreen.jsx` with header and two-panel layout
- [x] Add search input in header (right-aligned, 180px, placeholder "Search converted...")
- [x] Implement search filtering (case-insensitive substring on EPUB file name)
- [x] Filter ImportContext files to only those with `status === 'converted'`
- [x] Read initial selected EPUB from React Router location state
- [x] Auto-select first EPUB when none is selected
- [x] Show empty state when no files are converted
- [x] Add "Open folder" button in header (visible in Tauri mode only)

### T2: Create EPUB List
- [x] Create `src/components/converted/EpubList.jsx` (260px fixed width, scrollable)
- [x] Create `src/components/converted/EpubListItem.jsx` with EPUB name, size
- [x] Implement selected state styling (accent background, bold text)
- [x] Implement hover state styling
- [x] Derive EPUB file name from `outputPath` or source PDF name

### T3: Create Detail Panel
- [x] Create `src/components/converted/EpubDetailPanel.jsx` as the right panel container
- [x] Create `src/components/converted/EpubPreview.jsx` with placeholder UI (Book icon + chapter count)
- [x] Create `src/components/converted/EpubMetadata.jsx` with labelled properties (Source, EPUB size, Chapters, Images, Settings used)
- [x] Hide metadata rows when values are absent or zero
- [x] Create `src/components/converted/TableOfContents.jsx` with collapsible placeholder

### T4: Action Buttons
- [x] Add "Open in reader" primary button with ExternalLink icon
- [x] Add "Save as..." secondary button with Download icon
- [x] Add "Reconvert" secondary button with RefreshCw icon
- [x] Wire "Reconvert" to navigate to `/library` with source file path in route state

## Phase 2: Tauri Bridge

### T5: Add Shell Bridge Functions
- [x] Add `openFileWithSystem(path)` to `src/lib/tauri.js`
- [x] Add `openFolder(path)` to `src/lib/tauri.js`
- [x] Both return no-op in browser mode
- [x] Wire "Open in reader" button to `openFileWithSystem()`
- [x] Wire "Open folder" header button to `openFolder()`
- [x] Wire "Save as..." button to existing `saveFile()` (read EPUB bytes from disk first)

### T6: Shell Plugin Setup
- [x] Install `@tauri-apps/plugin-shell` (npm) and `tauri-plugin-shell` (Cargo)
- [x] Register plugin in `src-tauri/src/lib.rs`
- [x] Add `shell:allow-open` to capabilities

## Phase 3: Persistence

### T6b: Persist Conversion Results to Disk
- [x] Add `output_path`, `chapters`, `images`, `epub_file_size` optional fields to Rust `BookMetadata` struct (with `#[serde(default)]` for backward compatibility)
- [x] Call `saveBookMetadata` in `useConversion` after successful conversion with `status: 'converted'` and result data
- [x] Restore `outputPath` and `conversionResult` in `LOAD_LIBRARY` reducer for books with `status === 'converted'`

## Phase 4: Wiring

### T7: Route and Navigation
- [x] Replace Converted placeholder in `src/App.jsx` with `ConvertedScreen`
- [x] Update `CompletedList` in Converting screen to pass file path via route state when navigating to `/converted`
- [x] Verify sidebar "Converted" link activates the correct route

## Phase 5: Tests

### T8: Unit Tests
- [x] Test ConvertedScreen empty state renders correctly
- [x] Test ConvertedScreen renders EPUB list and detail panel with converted files
- [x] Test only files with status `converted` appear in the list
- [x] Test search filtering reduces EPUB list
- [x] Test no-results message when search matches nothing
- [x] Test EpubListItem selected and hover states
- [x] Test EpubMetadata shows available fields, hides absent ones
- [x] Test TableOfContents collapse/expand placeholder
- [x] Test action buttons render correctly
- [x] Test "Reconvert" button navigates to Library with correct route state
- [x] Test auto-selection of first EPUB when none is selected
- [x] Test pre-selection from React Router location state (via EPUB name derivation)

### T9: E2E Tests
- [x] Test navigating from Converting to Converted pre-selects the EPUB
- [x] Test selecting different EPUBs updates the detail panel
- [x] Test search filters the EPUB list
- [x] Test empty state shown when no conversions exist
- [x] Test "Go to Import" button in empty state navigates correctly
- [x] Test sidebar navigation to Converted screen

## Acceptance Criteria

- [x] `/converted` route renders the Converted screen (not a placeholder)
- [x] All files with status `converted` appear in the EPUB list
- [x] Clicking an EPUB shows its metadata in the detail panel
- [x] Search filters the EPUB list by name in real time
- [x] Empty state shown when no files are converted, with a working "Go to Import" link
- [x] Action buttons (Open in reader, Save as, Reconvert) render correctly
- [x] "Reconvert" navigates to Library with the source PDF selected
- [x] EPUB preview shows a placeholder with chapter count
- [x] Design matches Library screen patterns (spacing, typography, colours)
- [x] Converted EPUB list persists across app restarts
