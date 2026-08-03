# 07 — Converted: Plan

## Architectural Approach

The Converted screen consumes `ImportContext` state, filtering for files with status `converted`. It mirrors the Library screen's master-detail layout. No new backend commands are required for the initial listing — all data is already in the file Map from the conversion pipeline. Two new Tauri bridge functions are needed for the action buttons: opening a file with the system default app and opening a folder in the file manager.

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
│  │                       │    TableOfContents       ││
│  │                       │    [Action buttons]      ││
│  └───────────────────────┴──────────────────────────┘│
│                                                      │
│  OR: EmptyConverted (when no files converted)        │
└──────────────────────────────────────────────────────┘
```

## Key Decisions

### D1: State Source — ImportContext (Filtered)

The Converted screen reads from the same `ImportContext` that Import and Library use. Converted files are identified by `status === 'converted'` and have `outputPath` and `conversionResult` fields. No separate "converted state" is needed.

### D2: Selection — Component State

The selected EPUB path is component-local state in ConvertedScreen, initialised from React Router location state (when navigating from the Converting screen). Search query is also component-local. This mirrors the Library screen's approach.

### D3: EPUB Preview — Placeholder

EPUB chapter rendering requires parsing EPUB XHTML content and rendering it in a sandboxed iframe/container. This is complex and deferred. The preview section shows a placeholder with a Book icon and chapter count. This parallels the Library's deferred page preview.

### D4: Table of Contents — Placeholder

Parsing the EPUB's NCX or nav document to build a clickable TOC requires either a Rust command to extract the TOC structure or frontend EPUB parsing. Deferred. The collapsible section shows a placeholder message when expanded.

### D5: Tauri Bridge — Open File and Open Folder

Two new bridge functions are added to `src/lib/tauri.js`:
- `openFileWithSystem(path)` — uses the Tauri shell plugin's `open()` to launch the file with the OS default app.
- `openFolder(path)` — uses the Tauri shell plugin's `open()` to open the containing folder in the file manager.

Both fall back to no-ops in browser mode (with the exception of "Open in reader" which triggers a download in browser mode).

### D6: Component Structure

Components are co-located in `src/components/converted/`. The screen follows the same page-level pattern as LibraryScreen: header row with title + actions, then content below.

### D7: File Name Derivation

EPUB file names are derived from the `outputPath` on the file object (extracting the basename). If `outputPath` is not available, the source PDF name is used with `.epub` appended.

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
| Shell plugin not yet installed | `openFileWithSystem` and `openFolder` return no-op in browser mode; feature degrades gracefully |
| EPUB preview complexity | Clear placeholder UI with book icon and chapter count; no broken functionality |
| TOC parsing complexity | Collapsible section with placeholder message; expandable in a future spec |
| Conversion result data missing | Handle null/undefined `conversionResult` gracefully; show available fields only |
