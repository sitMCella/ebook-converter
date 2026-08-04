# 05 — Storage Location

## Goal

Store imported PDF files and converted EPUB files in a managed, platform-specific application data directory, organised into per-book subdirectories. Each book gets a UUID-named folder containing its source PDF and converted EPUB, enabling clean lifecycle management (import, convert, delete) without name collisions or orphaned files.

## Background

Currently the application does not copy imported PDFs — it references them by their original filesystem path, held in session-only React state (`ImportContext`). Converted EPUBs are written to a user-configured output folder (default `~/Documents/Ebooks`). This means:

- Imported files are lost on app restart (session-only state).
- Moving or deleting the original PDF breaks the reference.
- Converted EPUBs live outside the app's control, with collision-avoidance numbering for duplicate names.

This feature introduces managed storage so the application owns copies of both source and output files, enabling persistent library, reconversion, and export.

## Storage Locations

The storage root is the platform-specific application data directory, resolved by Tauri's `AppHandle::path().app_data_dir()`. The Tauri app identifier is `sitmcella.ebook-converter`, which produces:

| Platform | Storage root |
|----------|-------------|
| macOS | `~/Library/Application Support/sitmcella.ebook-converter/` |
| Windows | `C:\Users\<user>\AppData\Roaming\sitmcella.ebook-converter\` |
| Linux | `~/.config/sitmcella.ebook-converter/` |

## Directory Structure

Each book is stored in its own UUID-named subdirectory under a `books/` folder within the storage root:

```
<app_data_dir>/
  settings.json               (existing — user settings)
  books/
    <uuid-1>/
      Design patterns.pdf     (copy of the imported PDF, original name preserved)
      Design patterns.epub    (converted EPUB, created on conversion)
      metadata.json           (book metadata, persisted on import)
    <uuid-2>/
      My report.pdf
      metadata.json
    ...
```

### Per-Book Directory Rationale

Per-book subdirectories are chosen over a flat structure for these reasons:

1. **No name collisions.** Two PDFs named `report.pdf` each get their own UUID directory. No need for collision-avoidance numbering.
2. **Clean deletion.** Removing a book and all its artifacts is a single `remove_dir_all` call. In a flat structure, tracking which files belong to which book requires additional bookkeeping.
3. **Extensibility.** Future additions (cover images, conversion logs, multiple output formats, per-book metadata JSON) fit naturally into the per-book directory without polluting a shared namespace.
4. **Predictable output path.** The conversion pipeline writes to `<book_dir>/<stem>.epub` — no collision detection needed since each book has its own directory.

**Trade-off**: browsing the storage folder manually shows opaque UUIDs instead of human-readable titles. This is acceptable because users interact through the app UI, not the filesystem.

## Functional Requirements

### FR-1: Copy PDF on Import to Library

When a user clicks "Import to library" on the Import screen for staged files, the application copies each file into managed storage:

1. Generate a new UUID v4 as the book identifier.
2. Create the directory `<app_data_dir>/books/<uuid>/`.
3. Copy the source PDF into the directory, preserving the original filename (e.g., `Design patterns.pdf`).
4. Save metadata to `<uuid>/metadata.json`.
5. Return the book ID and stored path to the frontend.

The copy happens in the Rust backend (not via the frontend filesystem plugin) to avoid scope restrictions. This is triggered by the "Import to library" action, NOT during the initial staging/validation phase.

### FR-2: Store EPUB in Book Directory

When a PDF is converted to EPUB, the output file is written to `<book_dir>/<stem>.epub` (where `<stem>` is the PDF filename without extension, e.g., `Design patterns.pdf` → `Design patterns.epub`) instead of the user-configured output folder. The conversion pipeline uses the book ID and the PDF path to resolve the output path.

### FR-3: Delete Book from Storage

When a user removes a book from the library, the application deletes the entire book directory (`<app_data_dir>/books/<uuid>/`), removing both the source PDF, any converted EPUB, and the metadata file.

Note: removing a file from the Import screen's staging list does NOT delete from storage. Staging removal is non-destructive — it simply removes the file from the staging area. Only library-level deletion triggers storage cleanup.

### FR-4: Output Location Setting Removed

The "Output location" setting group in the Settings screen (Screen 5) is removed. The `outputLocation.defaultFolder` field is no longer used — all output goes to the per-book directory within managed storage. The "Open folder" button on the Converted screen (Screen 4) opens the `books/` directory in the OS file manager.

### FR-5: Export EPUB

Users can still save a copy of a converted EPUB to any location using the existing "Save as..." button on the Converted screen (Screen 4). This uses the Tauri native save dialog and copies the file from managed storage to the user-chosen location.

## Non-Functional Requirements

### NFR-1: Copy Performance

PDF file copying should not block the UI. The `import_pdf` Rust command runs on Tauri's async runtime. For large files (100+ MB), the file appears in the library immediately with status `Ready`; the copy completes in the background.

### NFR-2: Disk Space

Storing both source PDF and converted EPUB approximately doubles the storage compared to keeping only the EPUB. This is the expected behaviour — users can delete books from the app to reclaim space.

### NFR-3: Path Validation

Book IDs passed from the frontend are validated as UUID v4 format before being used to construct filesystem paths. This prevents path traversal attacks.

## UI Changes

### Settings Screen (Screen 5)

Per the UI/UX design spec (02), the "Output location" setting group is removed from the right column. The setting group previously contained:

| Setting | Control | Default |
|---|---|---|
| Default output folder | Button: "Choose" | `~/Documents/Ebooks` |

This group is no longer displayed. The two-column settings layout is rebalanced: the "Page handling" group remains in the right column.

### Converted Screen (Screen 4)

The "Open folder" button in the header opens the `<app_data_dir>/books/` directory in the OS file manager, rather than the previous output folder.

### Import Screen (Screen 1)

The import screen operates as a staging area. Files are validated and previewed in the staging list. The PDF copy to managed storage happens only when the user clicks "Import to library", at which point files move from staging to the library. Removing staged files is non-destructive — no storage operations occur.

## Out of Scope

- Migration from an existing output folder — there is no prior managed storage to migrate from.
- Configurable storage location — the app data directory is fixed per platform.
- Deduplication of identical PDFs imported from different paths.
