# 04 — Convert PDF to EPUB: Data Model

## Tauri IPC Commands

### `convert_pdf`

Converts a PDF file to EPUB. Runs asynchronously. Emits `conversion-progress` events during processing.

```typescript
// Frontend call
const result: ConversionResult = await invoke('convert_pdf', {
  path: string,
  options: ConversionOptions,
});
```

```rust
// Backend handler
#[tauri::command]
async fn convert_pdf(
    app: tauri::AppHandle,
    state: State<'_, ConversionState>,
    path: String,
    options: ConversionOptions,
) -> Result<ConversionResult, String>
```

#### `ConversionOptions` (Request)

```typescript
interface ConversionOptions {
  structure: {
    detectHeadings: boolean;
    detectToc: boolean;
    detectFootnotes: boolean;
    headingLevelThreshold: number;
    paragraphDetection: boolean;
    listDetection: boolean;
  };
  images: {
    extractImages: boolean;
    imageQuality: "high" | "medium" | "low";
    maxImageWidth: number;
    convertToWebP: boolean;
  };
  output: {
    epubVersion: "epub2" | "epub3";
    embedFonts: boolean;
    fontFamily: "default" | "serif" | "sans-serif" | "monospace";
    baseFontSize: number;
    lineHeight: number;
    margins: number;
  };
  pageHandling: {
    skipBlankPages: boolean;
    pageRange: "all" | "custom";
    pageRangeFrom: number | null;
    pageRangeTo: number | null;
    splitChaptersBy: "heading1" | "heading2" | "pageBreak" | "none";
  };
  outputFolder: string;
}
```

```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConversionOptions {
    structure: StructureOptions,
    images: ImageOptions,
    output: OutputOptions,
    page_handling: PageHandlingOptions,
    output_folder: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StructureOptions {
    detect_headings: bool,
    detect_toc: bool,
    detect_footnotes: bool,
    heading_level_threshold: u8,
    paragraph_detection: bool,
    list_detection: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImageOptions {
    extract_images: bool,
    image_quality: String,       // "high" | "medium" | "low"
    max_image_width: u32,
    convert_to_webp: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OutputOptions {
    epub_version: String,        // "epub2" | "epub3"
    embed_fonts: bool,
    font_family: String,         // "default" | "serif" | "sans-serif" | "monospace"
    base_font_size: u8,
    line_height: f32,
    margins: f32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PageHandlingOptions {
    skip_blank_pages: bool,
    page_range: String,          // "all" | "custom"
    page_range_from: Option<u32>,
    page_range_to: Option<u32>,
    split_chapters_by: String,   // "heading1" | "heading2" | "pageBreak" | "none"
}
```

#### `ConversionResult` (Response)

```typescript
interface ConversionResult {
  outputPath: string;
  chapters: number;
  images: number;
  fileSize: number;
}
```

```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConversionResult {
    output_path: String,
    chapters: usize,
    images: usize,
    file_size: u64,
}
```

---

### `cancel_conversion`

Requests cancellation of an active conversion. The conversion will stop at the next safe checkpoint and clean up partial output.

```typescript
// Frontend call
await invoke('cancel_conversion', { path: string });
```

```rust
// Backend handler
#[tauri::command]
fn cancel_conversion(
    state: State<'_, ConversionState>,
    path: String,
) -> Result<(), String>
```

---

## Tauri Events

### `conversion-progress` (Backend → Frontend)

Emitted during conversion to report progress. The frontend listens for this event to update the UI.

```typescript
// Frontend listener
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen<ConversionProgress>('conversion-progress', (event) => {
  const { path, stage, percent, message } = event.payload;
  // Update UI
});
```

#### `ConversionProgress` (Event Payload)

```typescript
interface ConversionProgress {
  path: string;
  stage: ConversionStage;
  percent: number;
  message: string;
}

type ConversionStage =
  | "extracting_text"
  | "detecting_structure"
  | "extracting_images"
  | "splitting_chapters"
  | "generating_toc"
  | "assembling_epub"
  | "writing_file"
  | "complete"
  | "error";
```

```rust
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ConversionProgress {
    path: String,
    stage: String,
    percent: u8,
    message: String,
}
```

---

## Backend State

### `ConversionState`

Managed state in the Tauri app for tracking active conversions and supporting cancellation.

```rust
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::collections::HashMap;

struct ConversionState {
    cancel_tokens: Mutex<HashMap<String, Arc<AtomicBool>>>,
}
```

Registered in `src-tauri/src/lib.rs`:

```rust
.manage(ConversionState {
    cancel_tokens: Mutex::new(HashMap::new()),
})
```

---

## Frontend Interfaces

### Extended `ImportedFile`

The existing `ImportedFile` shape (from spec 03) is extended with conversion-related fields:

```typescript
interface ImportedFile {
  // Existing fields (spec 03)
  path: string;
  name: string;
  size: number;
  status: "ready" | "converting" | "converted" | "error";
  errorMessage?: string;
  metadata: PdfMetadata | null;

  // New fields (this spec)
  conversionProgress?: number;      // 0–100, present when status is "converting"
  conversionStage?: ConversionStage; // Current pipeline stage
  outputPath?: string;              // Absolute path to generated EPUB, present when status is "converted"
  conversionResult?: ConversionResult; // Chapters, images, file size of output
}
```

### New Reducer Actions

Added to `ImportContext.jsx`:

```typescript
type ImportAction =
  // Existing actions (spec 03)
  | { type: "ADD_FILES"; files: ImportedFile[] }
  | { type: "REMOVE_FILES"; paths: string[] }
  | { type: "UPDATE_STATUS"; path: string; status: ImportedFile["status"]; errorMessage?: string }
  | { type: "SET_METADATA"; path: string; metadata: PdfMetadata }
  | { type: "TOGGLE_SELECTION"; path: string }
  | { type: "SELECT_ALL" }
  | { type: "DESELECT_ALL" }

  // New actions (this spec)
  | { type: "SET_CONVERSION_PROGRESS"; path: string; percent: number; stage: ConversionStage }
  | { type: "SET_CONVERSION_RESULT"; path: string; outputPath: string; result: ConversionResult };
```

#### `SET_CONVERSION_PROGRESS`

Updates the `conversionProgress` and `conversionStage` fields on a file. Only applies when status is `converting`.

#### `SET_CONVERSION_RESULT`

Sets the `outputPath` and `conversionResult` fields on a file. Automatically sets status to `converted`.

---

### Conversion Queue State

A new `ConversionContext` manages the conversion queue separately from the import list:

```typescript
interface ConversionQueueState {
  /** Ordered list of file paths waiting to be converted */
  queue: string[];

  /** Path of the file currently being converted, or null */
  activeFile: string | null;

  /** Paths of files that completed conversion in this session */
  completedFiles: string[];

  /** Log entries for the current conversion */
  logEntries: ConversionLogEntry[];

  /** Whether the overall batch is complete */
  isComplete: boolean;
}

interface ConversionLogEntry {
  timestamp: number;
  message: string;
  level: "info" | "error";
}

type ConversionQueueAction =
  | { type: "ENQUEUE_FILES"; paths: string[] }
  | { type: "START_NEXT" }
  | { type: "COMPLETE_ACTIVE"; path: string }
  | { type: "FAIL_ACTIVE"; path: string }
  | { type: "CANCEL_ALL" }
  | { type: "ADD_LOG_ENTRY"; entry: ConversionLogEntry }
  | { type: "CLEAR_LOG" };
```

---

## Extended Tauri Bridge

New functions added to `src/lib/tauri.js`:

```typescript
/**
 * Converts a PDF to EPUB. Returns a promise that resolves when conversion completes.
 * Progress is reported via the 'conversion-progress' event, not via this promise.
 */
async function convertPdfToEpub(
  path: string,
  options: ConversionOptions
): Promise<ConversionResult>;

/**
 * Cancels an active conversion for the given file path.
 */
async function cancelConversion(path: string): Promise<void>;

/**
 * Listens for conversion progress events.
 * Returns an unlisten function.
 */
async function onConversionProgress(
  callback: (progress: ConversionProgress) => void
): Promise<() => void>;
```

---

## Tauri Capability Additions

`src-tauri/capabilities/default.json` — add write access for EPUB output:

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
    "fs:allow-stat",
    "fs:allow-mkdir",
    "fs:allow-exists"
  ]
}
```

---

## Cargo Dependency Additions

`src-tauri/Cargo.toml`:

```toml
[dependencies]
pdf-extract = "0.7"
epub-builder = "0.7"
image = { version = "0.25", default-features = false, features = ["jpeg", "png", "webp"] }
```

- `pdf-extract` — text extraction from PDF pages.
- `epub-builder` — EPUB file generation (EPUB 2 and 3).
- `image` — image decoding, resizing, and re-encoding for extracted PDF images.

---

## Settings File Schema

Settings are stored as JSON in the Tauri app data directory. The frontend reads/writes this file via the Tauri filesystem plugin.

**File path**: `{app_data_dir}/settings.json`

```typescript
interface PersistedSettings {
  structure: {
    detectHeadings: boolean;           // default: true
    detectToc: boolean;                // default: true
    detectFootnotes: boolean;          // default: false
    headingLevelThreshold: number;     // default: 3
    paragraphDetection: boolean;       // default: true
    listDetection: boolean;            // default: true
  };
  images: {
    extractImages: boolean;            // default: true
    imageQuality: "high" | "medium" | "low"; // default: "medium"
    maxImageWidth: number;             // default: 800
    convertToWebP: boolean;            // default: false
  };
  output: {
    epubVersion: "epub2" | "epub3";    // default: "epub3"
    embedFonts: boolean;               // default: false
    fontFamily: "default" | "serif" | "sans-serif" | "monospace"; // default: "default"
    baseFontSize: number;              // default: 12
    lineHeight: number;                // default: 1.5
    margins: number;                   // default: 1.0
  };
  pageHandling: {
    skipBlankPages: boolean;           // default: true
    pageRange: "all" | "custom";       // default: "all"
    pageRangeFrom: number | null;      // default: null
    pageRangeTo: number | null;        // default: null
    splitChaptersBy: "heading1" | "heading2" | "pageBreak" | "none"; // default: "heading1"
  };
  outputLocation: {
    defaultFolder: string;             // default: "~/Documents/Ebooks"
  };
}
```

When the file does not exist, all defaults are used. Missing keys are filled with defaults (forward-compatible).

---

## Output File Path Resolution

```typescript
function resolveOutputPath(
  pdfPath: string,
  outputFolder: string
): string {
  // 1. Extract filename without extension: "Design patterns.pdf" → "Design patterns"
  // 2. Append ".epub": "Design patterns.epub"
  // 3. Join with outputFolder: "/Users/me/Documents/Ebooks/Design patterns.epub"
  // 4. If file exists, append " (N)": "Design patterns (1).epub"
}
```

This resolution happens on the Rust side to avoid race conditions between the check and the write.
