# 05 — Storage Location: Plan

## Architectural Approach

The storage system lives in the Rust backend as a new `storage` module. The frontend calls a new IPC command (`import_pdf`) to copy files into managed storage and receives a book ID. The conversion pipeline is updated to resolve EPUB output paths from the book ID instead of a user-configured folder.

```
┌─────────────────────────────────────────────────────────────┐
│  Frontend (React)                                           │
│                                                             │
│  useImport.js ──► importPdf(sourcePath)                     │
│       │               │                                     │
│       │               ▼                                     │
│       │          tauri.js ──► invoke('import_pdf', {...})    │
│       │               │                                     │
│       │               ▼                                     │
│       │          { bookId, storedPdfPath }                   │
│       │                                                     │
│  useConversion.js ──► convertPdfToEpub(storedPdfPath, opts) │
│       │               │                                     │
│       │               ▼                                     │
│       │          opts.bookId → pipeline resolves output path │
├─────────────────────────────────────────────────────────────┤
│  Rust Backend                                               │
│                                                             │
│  storage.rs                                                 │
│    get_books_dir(app)      → <app_data_dir>/books/          │
│    get_book_dir(app, id)   → <app_data_dir>/books/<uuid>/   │
│    create_book_dir(app)    → books/<uuid>/                  │
│    import_pdf(app, path)   → copy + return StoredBook       │
│    get_epub_output_path(app, book_id) → books/<uuid>/output │
│    delete_book(app, book_id) → remove_dir_all               │
│                                                             │
│  conversion/pipeline.rs                                     │
│    run_conversion(app, path, options, cancel_token)          │
│      └─ when book_id is set, output → books/<uuid>/output   │
│      └─ when book_id is absent, fallback to output_folder   │
└─────────────────────────────────────────────────────────────┘
```

## Key Decisions

### D1: UUID v4 for Book Directories

Book directories are named with UUID v4 strings (e.g., `550e8400-e29b-41d4-a716-446655440000`). This avoids:
- Filesystem-unsafe characters from titles or filenames.
- Length limits on various platforms.
- Collisions from sanitised names.

The human-readable name comes from PDF metadata, not the directory name.

### D2: Original Filenames Inside Book Directories

Each book directory preserves the original filename for both the imported PDF and the converted EPUB. `copy_pdf_to_storage` extracts the filename from the source path (e.g., `Design patterns.pdf`). `get_epub_output_path` derives the EPUB name from the PDF path by replacing the extension (e.g., `Design patterns.epub`). Since each book has its own UUID directory, there are no name collisions regardless of the filename.

### D3: Copy in Rust, Not Frontend

The PDF copy is done in Rust via `std::fs::copy`. The frontend filesystem plugin has scope restrictions (only user-selected paths); the Rust backend has unrestricted filesystem access. This also keeps the copy operation on Tauri's async thread pool.

### D4: Optional book_id in ConversionOptions

The `ConversionOptions` struct gains an optional `book_id: Option<String>` field. When present, the pipeline resolves the output path from storage. When absent, it falls back to the existing `output_folder` logic. This preserves backward compatibility during the transition.

### D5: Remove Output Location Setting

The `outputLocation.defaultFolder` setting is removed from the UI since all output now goes to managed storage. The setting field is retained in `DEFAULT_SETTINGS` for backward compatibility with existing `settings.json` files, but the UI no longer displays or updates it. The `settingsToConversionOptions` function ignores the field when `bookId` is provided.

## Integration Points

### Frontend → Rust (New IPC Commands)

```js
// Import a PDF into managed storage
const { bookId, storedPdfPath } = await invoke('import_pdf', { sourcePath });

// Delete a book from managed storage
await invoke('delete_book', { bookId });

// Get the books directory path
const booksDir = await invoke('get_books_dir');

// Get a specific book's directory path (for "Open folder" button)
const bookDir = await invoke('get_book_dir', { bookId });
```

### Modified IPC Command

The existing `convert_pdf` command receives `bookId` inside `ConversionOptions`. No new command is needed for conversion.

### Rust → Frontend (No New Events)

No new events are needed. The existing `conversion-progress` event continues to work unchanged.

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Large PDF copy slows import | Copy runs on Tauri's async runtime; file appears in UI immediately with `Ready` status |
| Disk full during copy | Rust returns an error string; frontend shows error toast |
| Path traversal via crafted bookId | Validate bookId as UUID v4 format before constructing paths |
| Orphaned book directories (import fails mid-copy) | Clean up the directory on copy failure in the `import_pdf` command |
| Existing settings.json has outputLocation | Field is ignored when bookId is set; retained for schema compatibility |
