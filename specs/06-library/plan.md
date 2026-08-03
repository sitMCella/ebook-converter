# 06 — Library: Plan

## Architectural Approach

The Library screen consumes `ImportContext` state and persists book metadata to disk so the library survives app restarts. Two backend commands (`save_book_metadata`, `list_books`) handle persistence, and the `ImportProvider` loads persisted books on mount.

```
┌──────────────────────────────────────────────────────┐
│  LibraryScreen                                       │
│  ┌──────────────────────────────────┬──────────────┐ │
│  │  "Library"                       │ SearchInput  │ │
│  └──────────────────────────────────┴──────────────┘ │
│                                                      │
│  ┌───────────────────────┬──────────────────────────┐│
│  │  DocumentList (260px) │  DetailPanel (flex-1)    ││
│  │    DocumentListItem   │    PagePreview           ││
│  │    DocumentListItem   │    MetadataSection       ││
│  │    DocumentListItem   │    ConversionOptions     ││
│  │                       │    [Convert button]      ││
│  └───────────────────────┴──────────────────────────┘│
│                                                      │
│  OR: EmptyLibrary (when no files imported)           │
└──────────────────────────────────────────────────────┘
```

## Key Decisions

### D1: State Source — ImportContext with Persistence

The Library reads from the same `ImportContext` that the Import screen uses. No separate "library state" is needed since imported files _are_ the library. The file Map already contains metadata, status, and storage info. On startup, persisted book metadata is loaded from disk via the `list_books` Rust command and dispatched as a `LOAD_LIBRARY` action.

### D2: Per-Document Overrides — Stored in ImportContext

Per-document conversion overrides are added as an `overrides` field on the file object in the ImportContext. A new `SET_DOCUMENT_OVERRIDES` reducer action stores partial settings that merge on top of global defaults at conversion time. This aligns with the `getEffectiveSettings()` function already in `src/lib/settings.js`.

### D3: Selection — Component State

The selected document path is component-local state in LibraryScreen, initialised from React Router location state (when navigating from the Import list). Search query is also component-local.

### D4: Page Preview — Placeholder

PDF page rendering requires a backend command (`render_pdf_page`) that does not exist yet. The preview section shows a placeholder with the file icon and page count. This is explicitly deferred (spec FR-4).

### D5: Component Structure

Components are co-located in `src/components/library/`. The screen follows the same page-level pattern as ImportScreen: header row with title + actions, then content below.

## Integration Points

### Import Screen → Library

`ImportListRow` passes the file path via React Router state: `navigate('/library', { state: { selectedPath: file.path } })`. LibraryScreen reads this on mount to pre-select the document.

### Library → Conversion

The "Convert to EPUB" button uses the existing `useConversion` hook's `startConversion()` and navigates to `/converting`.

### Settings Merge

`getEffectiveSettings(globalSettings, file.overrides)` from `src/lib/settings.js` computes the effective settings for conversion. The ConversionOptions component reads global defaults via `loadSettings()` and displays them alongside per-document overrides.

### Storage → Library (Persistence)

The import flow in `useImport.js` calls `saveBookMetadata()` after successfully importing a PDF and extracting its metadata. This writes a `metadata.json` file to the book's storage directory (`<app_data>/books/<uuid>/metadata.json`). On startup, `ImportProvider` calls `listBooks()` in a `useEffect` to load all persisted books into the file Map via the `LOAD_LIBRARY` reducer action. The load is resilient: missing or malformed metadata files are silently skipped.

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Large file list performance | Document list uses fixed-height items; metadata display is for the selected file only |
| Settings screen not yet implemented | ConversionOptions reads defaults from `DEFAULT_SETTINGS`; global settings changes will work automatically once the Settings screen is built |
| Page preview not available | Clear placeholder UI; no broken functionality |
| Metadata file corruption | `read_all_book_metadata()` silently skips dirs with missing or malformed `metadata.json` |
| Startup load race condition | `LOAD_LIBRARY` does not overwrite files already in the Map, so books imported during the current session before the async load completes are preserved |
