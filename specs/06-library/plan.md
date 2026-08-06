# 06 — Library: Plan

## Architectural Approach

The Library screen consumes `ImportContext` state and persists book metadata to disk so the library survives app restarts. Two backend commands (`save_book_metadata`, `list_books`) handle persistence, and the `ImportProvider` loads persisted books on mount.

The library reads from `state.files` — the library Map that is separate from `state.stagedFiles` (the staging area). Files enter the library via the `IMPORT_TO_LIBRARY` action (triggered by "Import to library" on the Import screen) or via `LOAD_LIBRARY` on startup.

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

### D1: State Source — ImportContext with Staging/Library Separation

The Library reads from the `files` Map in `ImportContext` — the same Map that persists across app restarts. This is distinct from the `stagedFiles` Map used by the Import screen. No separate "library state" is needed since the `files` Map already contains metadata, status, storage info, and per-document overrides.

On startup, persisted book metadata is loaded from disk via the `list_books` Rust command and dispatched as a `LOAD_LIBRARY` action into the `files` Map.

### D2: Per-Document Overrides — Stored in ImportContext

Per-document conversion overrides are added as an `overrides` field on the file object in the ImportContext `files` Map. A new `SET_DOCUMENT_OVERRIDES` reducer action stores partial settings that merge on top of global defaults at conversion time. This aligns with the `getEffectiveSettings()` function already in `src/lib/settings.js`.

### D3: Selection — Component State

The selected document path is component-local state in LibraryScreen, initialised from React Router location state (when navigating from the Import list). Search query is also component-local.

### D4: Cover Page Preview — Extracted Image

The page preview section displays the cover image extracted from the first page of the stored PDF. A new Tauri IPC command `get_pdf_cover` reuses the existing `extract_cover_image` function from the conversion pipeline's `image_extractor` module. It extracts the largest embedded XObject image from page 1, encodes it as a base64 data URI (`data:image/jpeg;base64,...`), and returns it to the frontend.

The `PagePreview` component follows the same async loading pattern as `EpubPreview`: a `useEffect` triggers on the file's `storedPdfPath`, shows a loading spinner while fetching, then displays the image or a "No cover image available" fallback. This approach avoids adding a full PDF page rendering library (like pdfium or mupdf) while providing a useful preview for PDFs that have an embedded cover image on their first page.

### D5: Conversion Entry Point

Conversion is initiated from the Library screen's detail panel via the "Convert to EPUB" button. This replaces the old "Convert selected" button on the Import screen. Conversion is single-file: click a file in the library, then click "Convert to EPUB". The button calls `startConversion([file.path])` and navigates to `/converting`.

### D6: Component Structure

Components are co-located in `src/components/library/`. The screen follows the same page-level pattern as ImportScreen: header row with title + actions, then content below.

## Integration Points

### Import Screen → Library

Files move from staging to library when the user clicks "Import to library" on the Import screen. The `IMPORT_TO_LIBRARY` reducer action moves the file from `stagedFiles` to `files`, adding `bookId` and `storedPdfPath`. The file then appears in the library's document list.

### Library → Conversion

The "Convert to EPUB" button uses the existing `useConversion` hook's `startConversion()` and navigates to `/converting`.

### Settings Merge

`getEffectiveSettings(globalSettings, file.overrides)` from `src/lib/settings.js` computes the effective settings for conversion. The ConversionOptions component reads global defaults via `loadSettings()` and displays them alongside per-document overrides.

### Storage → Library (Persistence)

The import flow in `useImport.js` calls `importPdf()` and `saveBookMetadata()` during the "Import to library" action. This copies the PDF to managed storage and writes a `metadata.json` file to the book's storage directory. On startup, `ImportProvider` calls `listBooks()` in a `useEffect` to load all persisted books into the `files` Map via the `LOAD_LIBRARY` reducer action.

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Large file list performance | Document list uses fixed-height items; metadata display is for the selected file only |
| Settings screen not yet implemented | ConversionOptions reads defaults from `DEFAULT_SETTINGS`; global settings changes will work automatically once the Settings screen is built |
| PDF has no embedded cover image | Fallback placeholder shown ("No cover image available") — no broken functionality |
| Metadata file corruption | `read_all_book_metadata()` silently skips dirs with missing or malformed `metadata.json` |
| Startup load race condition | `LOAD_LIBRARY` does not overwrite files already in the Map, so books imported during the current session before the async load completes are preserved |
