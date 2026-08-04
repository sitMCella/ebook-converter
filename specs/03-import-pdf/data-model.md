# 03 — Import PDF: Data Model

## Tauri IPC Commands

### `validate_pdf`

Validates a PDF file at the given path. Reads only the header and trailer — does not parse full content.

```typescript
// Frontend call
const result: PdfValidation = await invoke('validate_pdf', { path: string });
```

```rust
// Backend handler
#[tauri::command]
fn validate_pdf(path: String) -> Result<PdfValidation, String>
```

#### `PdfValidation` (Response)

```typescript
type PdfValidation =
  | { status: "valid" }
  | { status: "encrypted" }
  | { status: "error"; message: string };
```

```rust
#[derive(Serialize)]
#[serde(tag = "status")]
enum PdfValidation {
    #[serde(rename = "valid")]
    Valid,
    #[serde(rename = "encrypted")]
    Encrypted,
    #[serde(rename = "error")]
    Error { message: String },
}
```

---

### `get_pdf_metadata`

Extracts metadata from a valid PDF. Should only be called after `validate_pdf` returns `valid`.

```typescript
// Frontend call
const metadata: PdfMetadata = await invoke('get_pdf_metadata', { path: string });
```

```rust
// Backend handler
#[tauri::command]
fn get_pdf_metadata(path: String) -> Result<PdfMetadata, String>
```

#### `PdfMetadata` (Response)

```typescript
interface PdfMetadata {
  title: string | null;
  author: string | null;
  pageCount: number;
  pdfVersion: string;          // e.g. "1.7"
  createdDate: string | null;  // ISO 8601, e.g. "1994-10-21T00:00:00Z"
  modifiedDate: string | null; // ISO 8601
  producer: string | null;
  fileSize: number;            // bytes
}
```

```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PdfMetadata {
    title: Option<String>,
    author: Option<String>,
    page_count: usize,
    pdf_version: String,
    created_date: Option<String>,
    modified_date: Option<String>,
    producer: Option<String>,
    file_size: u64,
}
```

---

## Frontend Interfaces

### `StagedFile`

Represents a single file in the staging list, waiting to be imported to the library.

```typescript
interface StagedFile {
  /** Absolute filesystem path — used as the unique key */
  path: string;

  /** Display name (filename without directory) */
  name: string;

  /** File size in bytes */
  size: number;

  /** Current status in the staging pipeline */
  status: "ready" | "error";

  /** Error message (only when status is "error") */
  errorMessage?: string;

  /** Extracted PDF metadata (null while loading or on error) */
  metadata: PdfMetadata | null;
}
```

### `ImportedFile`

Represents a file that has been imported to the library (persisted in managed storage).

```typescript
interface ImportedFile {
  /** Absolute filesystem path — used as the unique key */
  path: string;

  /** Display name (filename without directory) */
  name: string;

  /** File size in bytes */
  size: number;

  /** Current status in the pipeline */
  status: "ready" | "converting" | "converted" | "error";

  /** Error message (only when status is "error") */
  errorMessage?: string;

  /** Extracted PDF metadata (null while loading or on error) */
  metadata: PdfMetadata | null;

  /** UUID assigned during import to library */
  bookId: string | null;

  /** Path to the stored PDF copy in managed storage */
  storedPdfPath: string | null;
}
```

### `ImportState`

```typescript
interface ImportState {
  /** Files staged for import (not yet in library), keyed by path */
  stagedFiles: Map<string, StagedFile>;

  /** Library files (imported and persisted), keyed by path */
  files: Map<string, ImportedFile>;

  /** Set of paths currently checked in the UI */
  selectedPaths: Set<string>;
}
```

### `ImportAction`

Reducer actions for `ImportContext`:

```typescript
type ImportAction =
  // Staging actions
  | { type: "STAGE_FILES"; files: StagedFile[] }
  | { type: "UNSTAGE_FILES"; paths: string[] }
  | { type: "UPDATE_STAGED_STATUS"; path: string; status: StagedFile["status"]; errorMessage?: string }
  | { type: "SET_STAGED_METADATA"; path: string; metadata: PdfMetadata }
  // Library actions
  | { type: "IMPORT_TO_LIBRARY"; path: string; bookId: string; storedPdfPath: string }
  | { type: "ADD_FILES"; files: ImportedFile[] }
  | { type: "REMOVE_FILES"; paths: string[] }
  | { type: "UPDATE_STATUS"; path: string; status: ImportedFile["status"]; errorMessage?: string }
  | { type: "SET_METADATA"; path: string; metadata: PdfMetadata }
  | { type: "LOAD_LIBRARY"; books: BookMetadata[] }
  // Selection actions (operate on staged files)
  | { type: "TOGGLE_SELECTION"; path: string }
  | { type: "SELECT_ALL" }
  | { type: "DESELECT_ALL" };
```

---

## Extended Tauri Bridge

New functions added to `src/lib/tauri.js`:

```typescript
/**
 * Opens the native file dialog for selecting PDF files.
 * Returns an array of absolute file paths, or null if cancelled.
 */
async function openPdfFiles(): Promise<string[] | null>;

/**
 * Validates a PDF file at the given path.
 */
async function validatePdf(path: string): Promise<PdfValidation>;

/**
 * Extracts metadata from a valid PDF file.
 */
async function getPdfMetadata(path: string): Promise<PdfMetadata>;

/**
 * Gets the file size in bytes via the filesystem plugin.
 */
async function getFileSize(path: string): Promise<number>;

/**
 * Copies a PDF to managed storage and returns the book ID and stored path.
 */
async function importPdf(sourcePath: string): Promise<{ bookId: string; storedPdfPath: string }>;

/**
 * Saves book metadata to a JSON file in the book's storage directory.
 */
async function saveBookMetadata(metadata: BookMetadata): Promise<void>;
```

---

## Tauri Capability Additions

`src-tauri/capabilities/default.json` — add drag-and-drop permission:

```json
{
  "identifier": "default",
  "description": "Default permissions for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "dialog:allow-open",
    "dialog:allow-save",
    "fs:allow-read-file",
    "fs:allow-write-file",
    "fs:allow-stat"
  ]
}
```

---

## Cargo Dependency Additions

`src-tauri/Cargo.toml`:

```toml
[dependencies]
lopdf = "0.34"
```

---

## npm Dependency Additions

`package.json`:

```json
{
  "dependencies": {
    "react-router-dom": "^7",
    "sonner": "^2",
    "lucide-react": "^0.500"
  }
}
```

- `react-router-dom` — client-side routing for sidebar navigation.
- `sonner` — toast notifications (stacking, auto-dismiss).
- `lucide-react` — icon library (spec 02 references `upload`, `file-text`, `book`, `settings`, `cloud-upload`, `folder-open`, `transform` icons).

---

## File Size Formatting Utility

`src/lib/format.js`:

```typescript
function formatFileSize(bytes: number): string;
// < 1024         → "842 B"
// 1024–1023999   → "342 KB"   (no decimals)
// 1024000–1.07e9 → "12.4 MB"  (one decimal)
// ≥ 1.07e9       → "1.23 GB"  (two decimals)
```
