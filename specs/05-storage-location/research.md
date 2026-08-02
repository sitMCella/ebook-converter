# 05 — Storage Location: Research

## Platform-Specific App Data Directories

### How Tauri Resolves App Data

Tauri v2 provides `AppHandle::path().app_data_dir()` which resolves to a platform-specific directory based on the app identifier. The identifier `sitmcella.ebook-converter` (set in `src-tauri/tauri.conf.json`) produces:

| Platform | API | Resolved Path |
|----------|-----|---------------|
| macOS | `NSSearchPathForDirectoriesInDomains` | `~/Library/Application Support/sitmcella.ebook-converter/` |
| Windows | `SHGetFolderPath(FOLDERID_RoamingAppData)` | `C:\Users\<user>\AppData\Roaming\sitmcella.ebook-converter\` |
| Linux | `$XDG_CONFIG_HOME` (fallback `~/.config`) | `~/.config/sitmcella.ebook-converter/` |

### `appDataDir` vs `appLocalDataDir`

Tauri exposes two related paths:

| Path | macOS | Windows | Linux |
|------|-------|---------|-------|
| `appDataDir` | `~/Library/Application Support/<id>/` | `AppData\Roaming\<id>\` | `~/.config/<id>/` |
| `appLocalDataDir` | Same as above | `AppData\Local\<id>\` | Same as above |

**Decision**: Use `appDataDir`. The distinction only matters on Windows, where `Roaming` syncs across machines in enterprise Active Directory environments and `Local` does not. For an ebook converter, the book files are user data that should follow the user if they roam between machines. The storage size concern (syncing large PDFs) is mitigated by the fact that most enterprise environments cap roaming profile sizes or exclude large files.

### Existing Usage in the Codebase

The application already uses `appDataDir` for settings persistence (`src/lib/settings.js`, lines 42-46). Adding book storage to the same root keeps all app-managed data co-located.

## Per-Book Directory Structure

### Alternatives Considered

#### Option A: Flat Structure

```
books/
  <uuid-1>.pdf
  <uuid-1>.epub
  <uuid-2>.pdf
```

**Pros**: Simpler directory structure, fewer filesystem operations.
**Cons**: Requires naming convention to associate source and output. No clean way to add per-book metadata files. Deletion requires finding all files matching a UUID prefix.

#### Option B: Per-Book Directories (Chosen)

```
books/
  <uuid-1>/
    source.pdf
    output.epub
  <uuid-2>/
    source.pdf
```

**Pros**: Clean lifecycle management. Extensible (add metadata.json, cover.jpg, conversion.log later). Single `remove_dir_all` for deletion. No name collision possible.
**Cons**: More directories to manage. Opaque names when browsing manually.

#### Option C: Human-Readable Directory Names

```
books/
  design-patterns-gamma/
    source.pdf
    output.epub
```

**Pros**: Human-readable when browsing the folder.
**Cons**: Requires sanitising filenames for all three platforms (different reserved characters on Windows vs Unix). Length limits (255 chars on most filesystems, 260 char total path on Windows without long path support). Collision risk from similar titles. Renaming directories when metadata changes.

**Decision**: Option B. The UUID approach is the most robust and requires no platform-specific filename sanitisation.

## UUID v4 Generation

### Crate: `uuid`

The `uuid` crate (v1.x) is the standard Rust library for UUID generation. The `v4` feature enables random UUID generation.

```toml
uuid = { version = "1", features = ["v4"] }
```

```rust
use uuid::Uuid;
let id = Uuid::new_v4().to_string();
// e.g., "550e8400-e29b-41d4-a716-446655440000"
```

**Crate stats**: 150M+ downloads, actively maintained, no transitive dependencies beyond `getrandom`. Already used transitively by Tauri itself.

### Path Traversal Prevention

A crafted `book_id` like `../../etc/passwd` could escape the `books/` directory. Validating the book ID as a UUID v4 string before constructing paths prevents this:

```rust
fn validate_book_id(book_id: &str) -> Result<(), String> {
    uuid::Uuid::parse_str(book_id)
        .map_err(|_| format!("Invalid book ID: {}", book_id))?;
    Ok(())
}
```

UUID strings only contain `[0-9a-f-]`, which are safe for all filesystems and cannot form path traversal sequences.

## File Copy Considerations

### `std::fs::copy` Behaviour

- On macOS/Linux: uses `sendfile` or equivalent for kernel-level copy (no userspace buffering).
- On Windows: uses `CopyFileExW` with progress callback support.
- Returns `Result<u64>` with the number of bytes copied.
- Preserves file permissions on Unix; copies security descriptors on Windows.
- Does not follow symlinks on the source — copies the symlink target.

### Large File Handling

For a 100 MB PDF, `std::fs::copy` typically completes in under 1 second on modern SSDs. The Tauri `#[tauri::command]` with `async` runs on the Tokio thread pool, so the UI remains responsive. No progress callback is needed for the copy operation itself — the file appears in the import list immediately, and the copy is a sub-second background operation.

### Failure Cleanup

If the copy fails (disk full, permission denied, source vanished), the freshly created book directory should be cleaned up:

```rust
async fn import_pdf(app: tauri::AppHandle, source_path: String) -> Result<StoredBook, String> {
    let (book_id, book_dir) = create_book_dir(&app)?;
    let dest = book_dir.join("source.pdf");

    if let Err(e) = std::fs::copy(&source_path, &dest) {
        let _ = std::fs::remove_dir_all(&book_dir);
        return Err(format!("Failed to copy PDF: {}", e));
    }

    Ok(StoredBook {
        book_id,
        stored_pdf_path: dest.to_string_lossy().to_string(),
    })
}
```

## Impact on Existing Features

### Conversion Pipeline

The `resolve_output_path` function in `pipeline.rs` currently derives the EPUB filename from the PDF stem and handles collisions with numbering (`file (1).epub`). With per-book directories, this logic is bypassed — the output is always `output.epub` in the book directory. The existing function is retained for backward compatibility but is only used when `book_id` is `None`.

### Settings

The `outputLocation.defaultFolder` setting (`~/Documents/Ebooks`) is no longer used when `bookId` is set in conversion options. The setting is retained in the schema for backward compatibility — existing `settings.json` files will not break — but the UI control is removed.

### Import State

The `ImportContext` file entries gain `bookId` and `storedPdfPath` fields. These are optional — the entry is created without them (via `ADD_FILES`), and they are populated after the `import_pdf` call succeeds (via `SET_STORAGE_INFO`). This two-step approach ensures the file appears in the UI immediately while the copy runs in the background.

## References

- [Tauri v2 Path API](https://v2.tauri.app/reference/javascript/api/namespacepath/)
- [Tauri v2 AppHandle paths](https://docs.rs/tauri/latest/tauri/struct.AppHandle.html)
- [uuid crate](https://crates.io/crates/uuid)
- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/latest/)
- [Windows Known Folders](https://learn.microsoft.com/en-us/windows/win32/shell/known-folders)
