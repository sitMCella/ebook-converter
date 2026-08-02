# 04 — Convert PDF to EPUB: Tasks

## Phase 1: Rust Backend — Conversion Engine

### T1: Add Conversion Dependencies
- [ ] Add `pdf-extract = "0.7"` to `src-tauri/Cargo.toml`
- [ ] Add `epub-builder = "0.7"` to `src-tauri/Cargo.toml`
- [ ] Add `image = { version = "0.25", default-features = false, features = ["jpeg", "png", "webp"] }` to `src-tauri/Cargo.toml`
- [ ] Run `cargo check` in `src-tauri/` to verify all crates resolve

### T2: Conversion Data Types
- [ ] Create `src-tauri/src/conversion/mod.rs` — module root, re-exports
- [ ] Define `ConversionOptions` struct with nested `StructureOptions`, `ImageOptions`, `OutputOptions`, `PageHandlingOptions` — all `Deserialize` with `camelCase`
- [ ] Define `ConversionResult` struct (`output_path`, `chapters`, `images`, `file_size`) — `Serialize` with `camelCase`
- [ ] Define `ConversionProgress` struct (`path`, `stage`, `percent`, `message`) — `Serialize` with `camelCase`
- [ ] Define `ConversionState` struct with `cancel_tokens: Mutex<HashMap<String, Arc<AtomicBool>>>`

### T3: Text Extraction Module
- [ ] Create `src-tauri/src/conversion/text_extractor.rs`
- [ ] Implement `extract_text(path: &str, page_range: Option<(u32, u32)>) -> Result<Vec<String>, String>`:
  - Use `pdf_extract::extract_text` to get full text
  - Split by form-feed (`\x0C`) to get per-page text
  - Apply page range filtering if `pageRange` is `custom`
  - Skip blank pages if `skipBlankPages` is true
- [ ] Handle extraction errors gracefully — return a descriptive error message

### T4: Structure Detection Module
- [ ] Create `src-tauri/src/conversion/structure_detector.rs`
- [ ] Define `StructuredContent` enum: `Heading { level, text }`, `Paragraph { text }`, `ListItem { text, ordered }`, `BlankLine`
- [ ] Implement `detect_structure(pages: &[String], options: &StructureOptions) -> Vec<StructuredContent>`:
  - Merge text across pages into a continuous stream
  - Detect paragraph boundaries (double newlines)
  - Detect headings: short lines (< 80 chars), preceded by blank lines, uppercase/title case, not ending with period — up to `headingLevelThreshold`
  - Detect bullet lists: lines starting with `•`, `–`, `-`, `*` + space
  - Detect numbered lists: lines starting with `1.`, `(a)`, `(1)` patterns
  - Respect on/off toggles from options (detectHeadings, paragraphDetection, listDetection)

### T5: Image Extraction Module
- [ ] Create `src-tauri/src/conversion/image_extractor.rs`
- [ ] Implement `extract_images(path: &str, options: &ImageOptions) -> Result<Vec<ExtractedImage>, String>`:
  - Load the PDF with `lopdf::Document::load`
  - Iterate pages → resources → XObject entries with `/Subtype /Image`
  - For JPEG-encoded streams (`/DCTDecode` filter): extract raw bytes as `.jpg`
  - For FlateDecode-compressed streams: decode with `lopdf`, then encode to PNG/JPEG using `image` crate
  - Resize images exceeding `maxImageWidth` using `image::imageops::resize`
  - If `convertToWebP` is true, encode as WebP instead of JPEG/PNG
  - Apply quality setting (high: 90, medium: 75, low: 50 for JPEG/WebP)
- [ ] Define `ExtractedImage` struct: `{ id: String, data: Vec<u8>, mime_type: String, width: u32, height: u32 }`
- [ ] Skip unsupported image formats (JPEG2000, JBIG2) with a warning log

### T6: Chapter Splitter Module
- [ ] Create `src-tauri/src/conversion/chapter_splitter.rs`
- [ ] Define `Chapter` struct: `{ title: String, content: Vec<StructuredContent>, images: Vec<String> }`
- [ ] Implement `split_chapters(content: Vec<StructuredContent>, options: &PageHandlingOptions) -> Vec<Chapter>`:
  - `heading1`: split at every `Heading { level: 1 }` — the heading becomes the chapter title
  - `heading2`: split at every `Heading { level: 1 | 2 }`
  - `pageBreak`: split at page boundaries preserved from text extraction
  - `none`: all content in a single chapter
  - If no headings found and strategy is heading-based, fall back to a single chapter
  - Assign a default title ("Chapter N") when a chapter has no heading

### T7: EPUB Generator Module
- [ ] Create `src-tauri/src/conversion/epub_generator.rs`
- [ ] Create `src-tauri/src/conversion/css.rs`
- [ ] Implement `generate_css(options: &OutputOptions) -> String`:
  - Generate CSS based on font family, font size, line height, margins settings
  - Include heading styles, paragraph styles, list styles, image styles
- [ ] Implement `generate_epub(chapters: &[Chapter], images: &[ExtractedImage], metadata: &PdfMetadata, options: &ConversionOptions, output_path: &str) -> Result<ConversionResult, String>`:
  - Create `EpubBuilder` with `ZipLibrary`
  - Set EPUB version from `options.output.epubVersion`
  - Set metadata (title from PDF metadata or filename, author)
  - Add generated CSS stylesheet
  - For each chapter: convert `StructuredContent` to XHTML, add as `EpubContent` with title
  - For each image: add as resource with correct MIME type
  - Generate the EPUB to a `Vec<u8>`
  - Write to `output_path`
  - Return `ConversionResult` with output stats
- [ ] Implement `content_to_xhtml(content: &[StructuredContent]) -> String`:
  - `Heading` → `<h1>`–`<h6>`
  - `Paragraph` → `<p>` with text escaping
  - `ListItem { ordered: false }` → `<ul><li>`
  - `ListItem { ordered: true }` → `<ol><li>`
  - Image references → `<img src="..." alt="" />`

### T8: Conversion Pipeline
- [ ] Create `src-tauri/src/conversion/pipeline.rs`
- [ ] Implement `run_conversion(app: &AppHandle, path: &str, options: &ConversionOptions, cancel_token: Arc<AtomicBool>) -> Result<ConversionResult, String>`:
  - Emit progress at each stage transition
  - Stage 1 (0–40%): call `extract_text`, emit page-level progress
  - Stage 2 (40–50%): call `detect_structure`
  - Stage 3 (50–70%): call `extract_images` if enabled, emit per-image progress
  - Stage 4 (70–80%): call `split_chapters`
  - Stage 5 (80–95%): call `generate_epub`
  - Stage 6 (95–100%): write file, emit completion
  - Check `cancel_token` between stages — if cancelled, clean up and return error
  - Resolve output file path (handle existing file name collision)
- [ ] Implement output path resolution: base name from PDF, `.epub` extension, numeric suffix if exists

### T9: IPC Commands
- [ ] Add `convert_pdf` async command in `src-tauri/src/conversion/mod.rs`:
  - Register cancel token in `ConversionState`
  - Call `run_conversion`
  - Clean up cancel token on completion
  - Return `ConversionResult` or error
- [ ] Add `cancel_conversion` command:
  - Look up cancel token by path in `ConversionState`
  - Set the atomic bool to `true`
- [ ] Register both commands and `ConversionState` in `src-tauri/src/lib.rs`

### T10: Rust Unit Tests
- [ ] Add test PDFs to `src-tauri/tests/fixtures/` (text-only, with images, multi-page)
- [ ] Test `extract_text` returns per-page text for a known PDF
- [ ] Test `detect_structure` identifies headings, paragraphs, and lists from sample text
- [ ] Test `split_chapters` with heading1 strategy produces correct chapter boundaries
- [ ] Test `generate_epub` produces a valid EPUB file (check ZIP structure, verify mimetype, content.opf exists)
- [ ] Test `run_conversion` end-to-end with a simple PDF → verify EPUB output exists and has expected chapter count
- [ ] Test cancellation — start conversion, cancel immediately, verify partial output is cleaned up
- [ ] Test output path resolution — existing file produces `(1)` suffix

## Phase 2: Frontend — Tauri Bridge Extensions

### T11: Extend Tauri Bridge
- [ ] Add `convertPdfToEpub(path, options)` to `src/lib/tauri.js`:
  - Calls `invoke('convert_pdf', { path, options })`
  - Browser fallback: returns a rejected promise with "Conversion requires the desktop app"
- [ ] Add `cancelConversion(path)` to `src/lib/tauri.js`:
  - Calls `invoke('cancel_conversion', { path })`
- [ ] Add `onConversionProgress(callback)` to `src/lib/tauri.js`:
  - Uses `listen('conversion-progress', callback)` from `@tauri-apps/api/event`
  - Returns the unlisten function
  - Browser fallback: returns a no-op unlisten function

### T12: Settings Module
- [ ] Create `src/lib/settings.js`:
  - `DEFAULT_SETTINGS` object with all default values per spec 02
  - `loadSettings()` — reads `settings.json` from app data dir via Tauri FS plugin; returns defaults if file missing
  - `saveSettings(settings)` — writes to `settings.json`
  - `mergeSettings(global, overrides)` — deep merge, overrides take precedence
  - `getEffectiveSettings(globalSettings, documentOverrides)` — convenience wrapper
- [ ] Browser fallback: `loadSettings()` returns defaults; `saveSettings()` is a no-op

## Phase 3: Frontend — State Management

### T13: Extend ImportContext
- [ ] Add `SET_CONVERSION_PROGRESS` action to the reducer in `src/contexts/ImportContext.jsx`:
  - Updates `conversionProgress` and `conversionStage` fields on the file
- [ ] Add `SET_CONVERSION_RESULT` action:
  - Sets `outputPath`, `conversionResult` fields
  - Sets status to `converted`
- [ ] Ensure `UPDATE_STATUS` clears conversion fields when setting status back to `ready` or `error`

### T14: Create ConversionContext
- [ ] Create `src/contexts/ConversionContext.jsx`:
  - State: `queue`, `activeFile`, `completedFiles`, `logEntries`, `isComplete`
  - Actions: `ENQUEUE_FILES`, `START_NEXT`, `COMPLETE_ACTIVE`, `FAIL_ACTIVE`, `CANCEL_ALL`, `ADD_LOG_ENTRY`, `CLEAR_LOG`
  - Provider component
- [ ] Add `ConversionContext.Provider` to `App.jsx`

### T15: Conversion Orchestration Hook
- [ ] Create `src/hooks/useConversion.js`:
  - `startConversion(paths)`:
    1. Load effective settings for each file (global + per-document overrides)
    2. Dispatch `ENQUEUE_FILES` to ConversionContext
    3. Set status to `converting` for the first file via ImportContext dispatch
    4. Call `convertPdfToEpub(path, settings)` for the first file
    5. On completion: dispatch `COMPLETE_ACTIVE`, `SET_CONVERSION_RESULT`, move to next file
    6. On error: dispatch `FAIL_ACTIVE`, `UPDATE_STATUS` with error, move to next file
    7. When queue empty: set `isComplete` to true
  - `cancelAll()`:
    1. Call `cancelConversion(activeFile)` if active
    2. Dispatch `CANCEL_ALL`
    3. Reset statuses of queued files back to `ready`
  - Set up progress event listener on mount, clean up on unmount
  - Progress handler: dispatch `SET_CONVERSION_PROGRESS` + `ADD_LOG_ENTRY`
- [ ] Returns `{ startConversion, cancelAll, isConverting }`

## Phase 4: Frontend — Converting Screen UI

### T16: ProgressBar Component
- [ ] Create `src/components/conversion/ProgressBar.jsx`:
  - 4 px height, rounded (2 px radius)
  - Accent colour fill, border colour track
  - `aria-valuenow`, `aria-valuemin="0"`, `aria-valuemax="100"`, `aria-label`
  - Accepts `percent` prop (0–100)

### T17: ConversionQueueRow Component
- [ ] Create `src/components/conversion/ConversionQueueRow.jsx`:
  - PDF file icon
  - File name (weight 500)
  - Progress percentage or "Queued" (muted text, right-aligned)
  - Status badge: `Converting` (warning) or `Pending` (accent)
  - ProgressBar below the row (only for the active file)

### T18: ConversionQueue Component
- [ ] Create `src/components/conversion/ConversionQueue.jsx`:
  - Renders active file row with progress bar
  - Renders queued file rows with "Queued" label and "Pending" badge
  - Uses ConversionContext to determine queue state

### T19: ConversionLog Component
- [ ] Create `src/components/conversion/ConversionLog.jsx`:
  - Bordered panel with info icon + "Conversion log" header (weight 500)
  - Monospace text (12 px, muted colour) for log entries
  - Error entries in danger text colour
  - Auto-scrolls to the latest entry (via `scrollIntoView` on new entries)
  - `aria-live="polite"` region for screen reader announcements
  - Uses ConversionContext `logEntries`

### T20: CompletedList Component
- [ ] Create `src/components/conversion/CompletedList.jsx`:
  - "Completed" muted label header
  - Each row: check icon (success colour), file name, `Converted` status badge
  - Clicking a completed row navigates to `/converted`
  - Uses ConversionContext `completedFiles`

### T21: ConvertingScreen Assembly
- [ ] Create `src/components/conversion/ConvertingScreen.jsx`:
  - Header: "Converting" title (h3) or "Conversion complete" when done
  - "Cancel all" secondary button (right-aligned) — visible when active/queued files exist
    - Shows ConfirmDialog: "Cancel N remaining conversion(s)? Files already converted are not affected."
    - On confirm: calls `cancelAll()` from useConversion
  - "View converted" primary button — visible when `isComplete` is true, navigates to `/converted`
  - ConversionQueue section
  - ConversionLog section
  - CompletedList section

### T22: Routing and Navigation
- [ ] Add `/converting` route in `src/App.jsx` pointing to `ConvertingScreen`
- [ ] Update `BatchActions.jsx`: "Convert selected" button calls `startConversion(selectedPaths)` from useConversion and navigates to `/converting`
- [ ] Add "Converting" option to sidebar navigation (or reuse "Converted" with a dynamic label based on active state)

## Phase 5: Wiring and Integration

### T23: Wire BatchActions to Conversion
- [ ] Update `src/components/import/BatchActions.jsx`:
  - On "Convert selected" click: collect selected paths with status `ready`
  - Call `startConversion(paths)` from useConversion hook
  - Navigate to `/converting`
- [ ] Update ImportContext: set status to `converting` for all queued files

### T24: Output Folder Handling
- [ ] On conversion start, verify the output folder exists via Tauri FS plugin
- [ ] If the folder does not exist, show toast: "The output folder no longer exists. Please choose a new one in Settings." and abort
- [ ] Create the output folder if it does not exist (using `fs:allow-mkdir` capability)
- [ ] Default output folder: resolve `~/Documents/Ebooks` to the platform-specific absolute path

### T25: Disk Space Check
- [ ] Before starting conversion, estimate output size (rough heuristic: input PDF size × 0.3 for text-heavy, × 0.8 for image-heavy)
- [ ] If estimated output exceeds available disk space, show danger toast and abort

## Phase 6: Testing

### T26: Frontend Unit Tests
- [ ] Test `ConversionContext` reducer: ENQUEUE_FILES, START_NEXT, COMPLETE_ACTIVE, FAIL_ACTIVE, CANCEL_ALL, ADD_LOG_ENTRY
- [ ] Test `ImportContext` extended reducer: SET_CONVERSION_PROGRESS, SET_CONVERSION_RESULT
- [ ] Test `mergeSettings` correctly deep-merges overrides
- [ ] Test ProgressBar renders correct width and aria attributes
- [ ] Test ConversionQueueRow displays correct status for active vs queued files
- [ ] Test ConversionLog auto-scrolls on new entries
- [ ] Test CompletedList renders completed files with correct badges

### T27: E2E Tests
- [ ] Test full conversion flow: import a PDF → select → click "Convert selected" → verify Converting screen shows progress → verify completion
- [ ] Test cancellation: start conversion → click "Cancel all" → verify confirmation dialog → confirm → verify queue cleared
- [ ] Test error handling: import a corrupted PDF → attempt conversion → verify error badge and log entry
- [ ] Test batch conversion: import 3 PDFs → convert all → verify sequential processing and completion

### T28: Integration Tests (Rust)
- [ ] Test `convert_pdf` IPC command end-to-end with a test PDF
- [ ] Verify the output EPUB file is a valid ZIP with correct mimetype
- [ ] Verify the EPUB contains expected chapters and metadata
- [ ] Test conversion with different option combinations (skip images, custom page range, heading2 split)
- [ ] Test cancellation mid-conversion

## Acceptance Criteria

- [ ] Clicking "Convert selected" on the import screen queues selected `Ready` files and navigates to the Converting screen
- [ ] The Converting screen shows the active file with a progress bar and percentage
- [ ] Queued files display as "Pending" below the active file
- [ ] The conversion log shows real-time messages from the backend during conversion
- [ ] On completion, files move to the "Completed" section with a `Converted` badge
- [ ] When all conversions finish, the header shows "Conversion complete" and a "View converted" button appears
- [ ] "Cancel all" stops the active conversion and clears queued files after confirmation
- [ ] A conversion error for one file does not stop the remaining queue
- [ ] Error files show an `Error` badge with an expandable error detail
- [ ] The generated EPUB file exists at the configured output folder with the correct name
- [ ] The EPUB contains chapters split by the configured strategy with a navigable table of contents
- [ ] Images are extracted and embedded (when the setting is enabled) at the configured quality and max width
- [ ] The EPUB CSS reflects the configured font family, font size, line height, and margins
- [ ] The status badge on the import list updates in real time during conversion (Ready → Converting → Converted)
- [ ] The Converting screen is keyboard navigable with proper ARIA attributes
- [ ] Conversion works correctly in both light and dark modes
