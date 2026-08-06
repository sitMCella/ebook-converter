# 07 — Converted: Data Model

## State — ImportContext (Read-Only)

The Converted screen reads from the existing `ImportContext` state. No new reducer actions are needed.

### Persistence

Converted file data is persisted to disk via `BookMetadata` in the Rust backend. When a conversion completes, `useConversion` calls `saveBookMetadata` to update the book's `metadata.json` with `status: 'converted'` and the conversion result fields (`outputPath`, `images`, `epubFileSize`). On app startup, the `LOAD_LIBRARY` reducer action restores `outputPath` and `conversionResult` from the loaded metadata for books with `status === 'converted'`.

#### BookMetadata — Conversion Fields

The Rust `BookMetadata` struct includes these optional fields for conversion results (backward-compatible via `#[serde(default)]`):

```rust
pub output_path: Option<String>,
pub images: Option<usize>,
pub epub_file_size: Option<u64>,
```

### Converted File Shape

Files with `status === 'converted'` have the following fields (set by `SET_CONVERSION_RESULT` action in spec 04, and restored from disk by `LOAD_LIBRARY` on startup):

```javascript
{
  // Existing fields (spec 03 + 04)
  path: string,              // Original PDF path (Map key)
  name: string,              // Original PDF file name
  size: number,              // Original PDF file size
  status: 'converted',
  metadata: {                // PDF metadata (from import)
    title: string | null,
    author: string | null,
    pageCount: number,
    pdfVersion: string,
    createdDate: string | null,
    modifiedDate: string | null,
    producer: string | null,
    fileSize: number,
  },
  bookId: string | null,
  storedPdfPath: string | null,

  // Conversion result fields (spec 04)
  outputPath: string,        // Absolute path to generated EPUB
  conversionResult: {
    outputPath: string,
    images: number,           // Number of extracted images
    fileSize: number,         // EPUB file size in bytes
  },

  // Per-document overrides (spec 06)
  overrides: object | undefined,
}
```

## Component Props

### ConvertedScreen

No props — reads from ImportContext and React Router.

### EpubList

```javascript
{
  files: ConvertedFile[],      // Filtered list of converted files
  selectedPath: string | null, // Currently selected file path
  onSelect: (path: string) => void,
}
```

### EpubListItem

```javascript
{
  file: ConvertedFile,
  selected: boolean,
  onSelect: () => void,
}
```

### EpubDetailPanel

```javascript
{
  file: ConvertedFile,
}
```

### EpubPreview

```javascript
{
  file: ConvertedFile,
}
```

### EpubMetadata

```javascript
{
  file: ConvertedFile,
}
```

## Derived Data

### EPUB File Name

The Rust backend names the EPUB file after the source PDF via `storage::get_epub_output_path`, which derives the stem from the PDF path (e.g., `Design patterns.pdf` → `books/<uuid>/Design patterns.epub`). If the PDF path cannot be parsed, it falls back to `output.epub`.

On the frontend, the display name is extracted from `file.outputPath`:

```javascript
function getEpubName(file) {
  if (file.outputPath) {
    return file.outputPath.split(/[\\/]/).pop();
  }
  return file.name.replace(/\.pdf$/i, '.epub');
}
```

### EPUB File Size

From `file.conversionResult.fileSize`. Formatted via `formatFileSize()` from `src/lib/format.js`.

### Conversion Date

Not currently stored on the file object. For the initial implementation, this field is omitted from the metadata display. A future enhancement could store a `convertedAt` timestamp in the `SET_CONVERSION_RESULT` action.

### Settings Used Label

```javascript
function getSettingsLabel(file) {
  if (!file.overrides) return 'Default';
  const count = countOverrides(file.overrides);
  if (count === 0) return 'Default';
  return `${count} override${count !== 1 ? 's' : ''}`;
}

function countOverrides(overrides) {
  let count = 0;
  for (const group of Object.values(overrides)) {
    if (typeof group === 'object' && group !== null) {
      count += Object.keys(group).length;
    }
  }
  return count;
}
```

### Output Folder Path

Extracted from `file.outputPath` by removing the filename:

```javascript
function getOutputFolder(file) {
  if (!file.outputPath) return null;
  const parts = file.outputPath.split(/[\\/]/);
  parts.pop();
  return parts.join('/');
}
```

## Tauri Bridge Extensions

New functions added to `src/lib/tauri.js`:

### `openFileWithSystem(path)`

Opens a file with the OS default application (e.g., EPUB reader).

```javascript
async function openFileWithSystem(path) {
  if (isTauri) {
    const { open } = await import('@tauri-apps/plugin-shell');
    return open(path);
  }
}
```

### `openFolder(path)`

Opens a folder in the OS file manager.

```javascript
async function openFolder(path) {
  if (isTauri) {
    const { open } = await import('@tauri-apps/plugin-shell');
    return open(path);
  }
}
```

### Tauri Plugin Dependency

The shell plugin is needed:
- npm: `@tauri-apps/plugin-shell`
- Cargo: `tauri-plugin-shell`
- Capability: `shell:allow-open`

If the shell plugin is not yet installed, these functions are no-ops. The feature degrades gracefully — action buttons are hidden or disabled in browser mode.

## Tauri Capability Additions

`src-tauri/capabilities/default.json` — add shell open permission:

```json
{
  "permissions": [
    "shell:allow-open"
  ]
}
```
