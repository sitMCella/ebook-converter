# 07 — Converted: Plan

## Architectural Approach

The Converted screen consumes `ImportContext` state, filtering for files with status `converted`. It mirrors the Library screen's master-detail layout. Conversion results are persisted to the book's `metadata.json` on disk (via extended `BookMetadata` fields) so that converted files survive app restarts. Two new Tauri bridge functions are needed for the action buttons: opening a file with the system default app and opening a folder in the file manager.

```
┌──────────────────────────────────────────────────────┐
│  ConvertedScreen                                     │
│  ┌──────────────────────────────────┬──────────────┐ │
│  │  "Converted EPUBs"              │ [Open folder] │ │
│  └──────────────────────────────────┴──────────────┘ │
│                                                      │
│  ┌───────────────────────┬──────────────────────────┐│
│  │  EpubList (260px)     │  EpubDetailPanel (flex-1)││
│  │    EpubListItem       │    EpubPreview           ││
│  │    EpubListItem       │    EpubMetadata          ││
│  │                       │    [Action buttons]      ││
│  └───────────────────────┴──────────────────────────┘│
│                                                      │
│  OR: EmptyConverted (when no files converted)        │
└──────────────────────────────────────────────────────┘
```

## Key Decisions

### D1: State Source — ImportContext (Filtered) with Disk Persistence

The Converted screen reads from the same `ImportContext` that Import and Library use. Converted files are identified by `status === 'converted'` and have `outputPath` and `conversionResult` fields. No separate "converted state" is needed.

Conversion results are persisted to disk: after a successful conversion, `useConversion` calls `saveBookMetadata` to write the `'converted'` status along with `outputPath`, `images`, and `epubFileSize` to the book's `metadata.json`. On startup, the `LOAD_LIBRARY` reducer restores these fields from the loaded `BookMetadata`, ensuring converted files appear after restart. The Rust `BookMetadata` struct uses `#[serde(default)]` on the new optional fields for backward compatibility with older metadata files.

### D2: Selection — Component State

The selected EPUB path is component-local state in ConvertedScreen, initialised from React Router location state (when navigating from the Converting screen). Search query is also component-local. This mirrors the Library screen's approach.

### D3: EPUB Preview — Cover Image

The preview section shows the EPUB cover image when available, extracted via the `readEpubPreview` bridge function. When no cover image is available, a placeholder with a Book icon is shown. Full EPUB content rendering is deferred.

### D4: Tauri Bridge — Open File and Open Folder

Two new bridge functions are added to `src/lib/tauri.js`:
- `openFileWithSystem(path)` — uses the Tauri shell plugin's `open()` to launch the file with the OS default app.
- `openFolder(path)` — uses the Tauri shell plugin's `open()` to open the containing folder in the file manager.

Both fall back to no-ops in browser mode (with the exception of "Open in reader" which triggers a download in browser mode).

### D5: Component Structure

Components are co-located in `src/components/converted/`. The screen follows the same page-level pattern as LibraryScreen: header row with title + actions, then content below.

### D6: File Name Derivation

The EPUB file on disk is named after the source PDF by the Rust backend: `storage::get_epub_output_path` extracts the PDF file stem and produces `books/<uuid>/<stem>.epub` (e.g., `Design patterns.pdf` → `Design patterns.epub`). On the frontend, EPUB file names are derived from the `outputPath` on the file object (extracting the basename). If `outputPath` is not available, the source PDF name is used with `.epub` appended.

## Integration Points

### Converting Screen → Converted

`CompletedList` in the Converting screen navigates to `/converted` with the file path via React Router state: `navigate('/converted', { state: { selectedPath: file.path } })`. ConvertedScreen reads this on mount to pre-select the EPUB.

### Converted → Library (Reconvert)

The "Reconvert" button navigates to `/library` with the source file path via React Router state, pre-selecting the source PDF in the Library detail panel.

### Tauri Bridge

| Function | Tauri Plugin | Description |
|---|---|---|
| `openFileWithSystem(path)` | `shell` plugin `open()` | Opens EPUB in default reader |
| `openFolder(path)` | `shell` plugin `open()` | Opens output folder in file manager |
| `saveFile(data, name, filters)` | `dialog` + `fs` plugins | Existing — save a copy |

### Settings

The Converted metadata section shows "Default" or "N overrides" for the "Settings used" field. This reads from `file.overrides` — if null/empty, it shows "Default"; otherwise it counts the override keys.

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| No converted files to show during development | Use the existing conversion pipeline to produce test files; unit tests seed converted file entries via dispatch |
| Shell plugin unavailable | Shell plugin is installed (`@tauri-apps/plugin-shell` npm, `tauri-plugin-shell` Cargo, registered in `lib.rs`, `shell:allow-open` capability granted); in browser mode, functions are no-ops |
| EPUB preview complexity | Cover image display when available; placeholder UI with book icon otherwise |
| Conversion result data missing | Handle null/undefined `conversionResult` gracefully; show available fields only |
| Converted files lost on restart | Conversion results persisted to `metadata.json` via `saveBookMetadata`; `LOAD_LIBRARY` restores `outputPath` and `conversionResult` for books with `status === 'converted'` |
| Old metadata.json without conversion fields | `BookMetadata` uses `#[serde(default)]` on new optional fields; old files deserialize safely with `None` values |
