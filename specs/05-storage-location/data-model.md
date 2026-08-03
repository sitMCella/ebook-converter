# 05 — Storage Location: Data Model

## Tauri IPC Commands

### `import_pdf`

Copy a PDF file into managed storage under a new book directory.

```typescript
// Frontend call
const result: StoredBook = await invoke('import_pdf', { sourcePath: string });
```

```rust
// Backend handler
#[tauri::command]
async fn import_pdf(app: tauri::AppHandle, source_path: String) -> Result<StoredBook, String>
```

### `delete_book`

Delete a book directory and all its contents (source PDF and converted EPUB).

```typescript
// Frontend call
await invoke('delete_book', { bookId: string });
```

```rust
// Backend handler
#[tauri::command]
fn delete_book(app: tauri::AppHandle, book_id: String) -> Result<(), String>
```

### `get_books_dir`

Return the absolute path to the `books/` directory. Used by the "Open folder" button on the Converted screen.

```typescript
// Frontend call
const booksDir: string = await invoke('get_books_dir');
```

```rust
// Backend handler
#[tauri::command]
fn get_books_dir(app: tauri::AppHandle) -> Result<String, String>
```

### `convert_pdf` (Modified)

The existing `convert_pdf` command is unchanged in signature. The `ConversionOptions` struct gains an optional `bookId` field.

## Rust Structs

### `StoredBook`

Returned by the `import_pdf` command.

```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredBook {
    pub book_id: String,
    pub stored_pdf_path: String,
}
```

### `ConversionOptions` (Modified)

The existing struct gains one optional field.

```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionOptions {
    pub structure: StructureOptions,
    pub images: ImageOptions,
    pub output: OutputOptions,
    pub page_handling: PageHandlingOptions,
    pub output_folder: String,
    pub book_id: Option<String>,  // NEW — when set, output goes to books/<uuid>/output.epub
}
```

## Frontend Interfaces

### `importPdf` (New — `src/lib/tauri.js`)

```typescript
async function importPdf(sourcePath: string): Promise<{
  bookId: string;
  storedPdfPath: string;
}>
```

Browser fallback returns `{ bookId: null, storedPdfPath: sourcePath }`.

### `deleteBook` (New — `src/lib/tauri.js`)

```typescript
async function deleteBook(bookId: string): Promise<void>
```

Browser fallback is a no-op.

### `getBooksDir` (New — `src/lib/tauri.js`)

```typescript
async function getBooksDir(): Promise<string>
```

Browser fallback returns an empty string.

### File Object Shape (Modified — `ImportContext`)

The file entries in the `ImportContext` `Map` gain two new fields:

```typescript
interface ImportedFile {
  path: string;              // original filesystem path (Map key)
  name: string;              // display name (filename only)
  size: number;
  status: 'ready' | 'converting' | 'converted' | 'error';
  errorMessage?: string;
  metadata: PdfMetadata | null;
  bookId?: string;           // NEW — UUID of the book directory
  storedPdfPath?: string;    // NEW — path to the copy in managed storage
}
```

### New Reducer Action — `SET_STORAGE_INFO`

```typescript
dispatch({
  type: 'SET_STORAGE_INFO',
  path: string,
  bookId: string,
  storedPdfPath: string,
});
```

Updates the file entry with storage information after a successful `import_pdf` call.

## Settings Changes

### `DEFAULT_SETTINGS` (Modified — `src/lib/settings.js`)

The `outputLocation` group is retained in the defaults for backward compatibility but is no longer exposed in the UI:

```javascript
outputLocation: {
  defaultFolder: '~/Documents/Ebooks',  // retained, but unused when bookId is set
}
```

### `settingsToConversionOptions` (Modified)

When a `bookId` is available, it is included in the options. The `outputFolder` field becomes a fallback for cases where `bookId` is not set.

```javascript
export function settingsToConversionOptions(settings, { outputFolder, bookId } = {}) {
  return {
    structure: settings.structure,
    images: settings.images,
    output: settings.output,
    pageHandling: settings.pageHandling,
    outputFolder: outputFolder || settings.outputLocation?.defaultFolder || '~/Documents/Ebooks',
    bookId: bookId || null,
  };
}
```

## Cargo Dependencies

### New Dependency

```toml
[dependencies]
uuid = { version = "1", features = ["v4"] }
```

## Filesystem Layout

```
<app_data_dir>/                              # Tauri app data dir
├── settings.json                            # User settings (existing)
└── books/                                   # NEW — managed book storage
    ├── 550e8400-e29b-41d4-a716-446655440000/
    │   ├── Design patterns.pdf              # Copied on import (original name preserved)
    │   └── Design patterns.epub             # Written on conversion (name derived from PDF)
    ├── 6ba7b810-9dad-11d1-80b4-00c04fd430c8/
    │   └── My report.pdf                    # Not yet converted
    └── ...
```
