# 06 — Library: Data Model

## State — ImportContext Extension

### File Object (extended)

The existing file object in the `ImportContext` `files` Map gains an `overrides` field:

```javascript
{
  // ... existing fields (path, name, size, status, metadata, bookId, storedPdfPath, etc.)
  overrides: Partial<ConversionSettings> | undefined,
}
```

### New Reducer Actions

```javascript
// SET_DOCUMENT_OVERRIDES
dispatch({
  type: 'SET_DOCUMENT_OVERRIDES',
  path: string,            // file path (Map key)
  overrides: object | null, // partial settings, or null to clear all
})

// LOAD_LIBRARY — bulk-load persisted books on startup
dispatch({
  type: 'LOAD_LIBRARY',
  books: BookMetadata[],   // array from list_books command
})
```

The `LOAD_LIBRARY` action populates the `files` Map from persisted metadata. It does not overwrite entries that already exist (e.g. files imported during the current session before the async load completes).

## Relationship to Staging

The library (`state.files`) is populated in two ways:

1. **Import from staging** — the `IMPORT_TO_LIBRARY` action moves a file from `state.stagedFiles` to `state.files`, adding `bookId` and `storedPdfPath`.
2. **Load on startup** — the `LOAD_LIBRARY` action populates `state.files` from persisted metadata on disk.

The staging area (`state.stagedFiles`) never feeds directly into the library screen — only files that have been explicitly imported appear here.

## Per-Document Overrides Shape

A partial subset of the global settings (excluding `outputLocation`):

```javascript
{
  structure?: {
    headingLevelThreshold?: number,
  },
  images?: {
    imageQuality?: 'high' | 'medium' | 'low',
  },
  output?: {
    baseFontSize?: number,
  },
  pageHandling?: {
    splitChaptersBy?: 'heading1' | 'heading2' | 'pageBreak' | 'none',
    pageRange?: 'all' | 'custom',
    pageRangeFrom?: number | null,
    pageRangeTo?: number | null,
  },
}
```

## Effective Settings Merge

At conversion time, the effective settings for a document are:

```javascript
import { getEffectiveSettings } from '../lib/settings';

const effective = getEffectiveSettings(globalSettings, file.overrides);
```

Only keys explicitly present in `file.overrides` replace global values. Absent keys inherit the global default.

## Component Props

### DocumentList

```javascript
{
  files: ImportedFile[],     // filtered list of files
  selectedPath: string|null, // currently selected file path
  onSelect: (path: string) => void,
}
```

### DocumentListItem

```javascript
{
  file: ImportedFile,
  selected: boolean,
  onSelect: () => void,
}
```

### DetailPanel

```javascript
{
  file: ImportedFile,
}
```

### MetadataSection

```javascript
{
  file: ImportedFile,
}
```

### ConversionOptions

```javascript
{
  file: ImportedFile,
}
```

### PagePreview

```javascript
{
  file: ImportedFile,
}
```

## PDF Metadata Shape (existing, from Rust)

```javascript
{
  title: string | null,
  author: string | null,
  pageCount: number,
  pdfVersion: string,
  createdDate: string | null,
  modifiedDate: string | null,
  producer: string | null,
  fileSize: number,
}
```

## BookMetadata (persisted, Rust struct)

Stored as `metadata.json` in each book's directory (`<app_data>/books/<uuid>/metadata.json`). Serialized with camelCase field names.

```rust
pub struct BookMetadata {
    pub book_id: String,
    pub stored_pdf_path: String,
    pub original_path: String,
    pub original_name: String,
    pub file_size: u64,
    pub title: Option<String>,
    pub author: Option<String>,
    pub page_count: u32,
    pub pdf_version: Option<String>,
    pub created_date: Option<String>,
    pub modified_date: Option<String>,
    pub producer: Option<String>,
    pub status: String,
}
```

## Tauri Bridge Functions (persistence)

| Function | Rust Command | Description |
|---|---|---|
| `saveBookMetadata(metadata)` | `save_book_metadata` | Writes `metadata.json` to the book's directory |
| `listBooks()` | `list_books` | Scans all book directories and returns their metadata |
