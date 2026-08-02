# 03 — Import PDF: Research

## PDF Parsing Libraries for Rust

### Library Comparison

| Criterion | lopdf | pdf-rs | pdfium (via pdfium-render) |
|-----------|-------|--------|----------------------------|
| Metadata extraction | Yes (document info dictionary) | Yes | Yes |
| Page count | Yes | Yes | Yes |
| Password detection | Yes (encryption dict check) | Partial | Yes |
| PDF version parsing | Yes (header) | Yes | Yes |
| Page rendering | No | No | Yes (full rendering engine) |
| Binary size impact | Small (~200 KB) | Small (~300 KB) | Large (~25 MB shared lib) |
| Pure Rust | Yes | Yes | No (C++ via FFI) |
| Maintenance | Active, widely used | Less active | Active, Google-backed |
| crates.io downloads | ~1.5M | ~200K | ~500K |

### Recommendation: lopdf for Import, pdfium for Rendering Later

**For this spec (import/validation/metadata)**: use `lopdf`. It is pure Rust, lightweight, and handles everything needed — reading the document info dictionary, detecting encryption, counting pages, and parsing the PDF version header. No external shared libraries required.

**For future specs (page rendering)**: `pdfium-render` will be needed. It wraps Google's PDFium engine (the same renderer used in Chrome) and can produce page images. This is a separate, heavier dependency that should be added when the Library screen preview feature is built, not in the import spec.

### lopdf Capabilities

```rust
use lopdf::Document;

// Load and validate
let doc = Document::load("file.pdf")?; // Errors on corruption

// Detect encryption
let is_encrypted = doc.is_encrypted();

// Page count
let page_count = doc.get_pages().len();

// PDF version
let version = format!("{}.{}", doc.version.0, doc.version.1);

// Metadata from document info dictionary
if let Ok(info) = doc.trailer.get(b"Info") {
    // Extract Title, Author, CreationDate, ModDate, Producer
}
```

### Password-Protected PDF Detection

lopdf can detect encrypted PDFs via the encryption dictionary in the trailer:
- If `doc.is_encrypted()` returns `true` and the document cannot be loaded without a password, the file is password-protected.
- Some PDFs have an empty owner password (permissions-restricted but readable) — `lopdf::Document::load` succeeds for these. Only user-password-encrypted PDFs that fail to load are flagged.

**Approach**: attempt `Document::load()`. If it returns an error indicating encryption, report "password-protected". If it returns a different error, report "corrupted/unreadable". If it succeeds and `is_encrypted()` is true but content is accessible, treat as a valid import with a note.

## Drag-and-Drop in Tauri v2

### Tauri v2 Drag-and-Drop Plugin

Tauri v2 provides built-in drag-and-drop support via the `tauri::DragDropEvent`:

```rust
// In setup or via the window event listener
app.listen("tauri://drag-drop", |event| {
    // event.payload() contains file paths
});
```

On the frontend, Tauri v2 emits drag-drop events that can be listened to:

```javascript
import { getCurrentWindow } from '@tauri-apps/api/window';

const unlisten = await getCurrentWindow().onDragDropEvent((event) => {
  if (event.payload.type === 'over') {
    // Visual feedback: drag is hovering
  } else if (event.payload.type === 'drop') {
    // event.payload.paths contains file paths
  } else if (event.payload.type === 'leave') {
    // Drag left the window
  }
});
```

### Alternative: HTML5 Drag-and-Drop

Standard HTML5 drag-and-drop (`ondragover`, `ondrop`) also works in the Tauri webview, but:
- File paths are not available in the HTML5 API (only `File` objects with content)
- Tauri's native API provides full filesystem paths, which are needed for the backend to read the PDF

**Decision**: use Tauri's native `onDragDropEvent` for file path access. Use HTML5 events only for visual feedback on the drop zone element (since Tauri's event is window-level, not element-level).

### Hybrid Approach

Combine both APIs:
1. **Tauri `onDragDropEvent`** — captures file paths when files are dropped anywhere on the window.
2. **HTML5 drag events on the drop zone** — provides element-level visual feedback (border colour, background tint, scale).
3. The drop handler filters paths to `.pdf` extensions before passing them to the backend.

## File Dialog Configuration

Tauri's dialog plugin supports multi-select:

```javascript
import { open } from '@tauri-apps/plugin-dialog';

const paths = await open({
  multiple: true,
  filters: [{ name: 'PDF Files', extensions: ['pdf'] }],
});
// paths is string[] when multiple: true, or string | null when false
```

The existing `src/lib/tauri.js` only supports single file selection (`multiple: false`). It needs to be extended for multi-file import.

## File Size Formatting

Per spec 02, file sizes follow these rules:
- `< 1 KB` → "842 B"
- `1 KB – 999 KB` → "342 KB" (no decimals)
- `1 MB – 999 MB` → "12.4 MB" (one decimal)
- `1 GB+` → "1.23 GB" (two decimals)

This is a frontend utility function. File size in bytes can be obtained either from the Rust backend (via `std::fs::metadata`) or on the frontend via the Tauri filesystem plugin's `stat()`.

## Toast Notification Libraries

For non-blocking toast notifications (duplicate file, errors), options:
- **react-hot-toast** — lightweight (~5 KB), simple API, supports stacking and auto-dismiss.
- **sonner** — modern, visually polished, stacking support, Tailwind-friendly.
- **Custom implementation** — a simple toast component using Tailwind and React state.

**Decision**: use `sonner` — it has built-in stacking (spec 02 requires "max 3 visible, newest on top"), auto-dismiss, manual dismiss, and integrates well with Tailwind CSS theming. If dependency minimization is preferred, a custom implementation is straightforward for the limited toast requirements.

## References

- [lopdf crate documentation](https://docs.rs/lopdf/)
- [Tauri v2 Drag & Drop](https://v2.tauri.app/reference/webview/drag-and-drop/)
- [Tauri v2 Dialog Plugin](https://v2.tauri.app/plugin/dialog/)
- [sonner — toast library](https://sonner.emilkowal.dev/)
