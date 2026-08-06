# 04 — Convert PDF to EPUB: Research

## PDF Text Extraction in Rust

### The Problem

`lopdf` (used in spec 03 for validation and metadata) parses the PDF object tree but does **not** extract readable text from page content streams. PDF pages encode text as a series of operator/operand pairs within content streams (e.g. `BT /F1 12 Tf (Hello) Tj ET`). Extracting text requires:

1. Parsing content stream operators (`BT`, `Tf`, `Tj`, `TJ`, `Tm`, etc.).
2. Resolving font encodings to map character codes to Unicode.
3. Tracking the text matrix to determine spatial position (x, y coordinates).
4. Grouping characters into words and lines based on spacing.
5. Inferring paragraph boundaries, headings, and structure from font size/weight and vertical gaps.

### Library Comparison

| Criterion | pdf-extract | pdfium-render | mupdf (via mupdf-rs) | poppler (via poppler-rs) |
|-----------|-------------|---------------|----------------------|--------------------------|
| Text extraction | Yes (basic) | Yes (full fidelity) | Yes (full fidelity) | Yes (full fidelity) |
| Structured text (position, font info) | No (plain text only) | Yes (char-level position + font) | Yes (stext structured output) | Partial |
| Image extraction | No | Yes | Yes | Yes |
| Page rendering | No | Yes (bitmap) | Yes (bitmap) | Yes (bitmap) |
| Pure Rust | Yes | No (C++ via FFI) | No (C via FFI) | No (C via FFI) |
| Binary size impact | Small (~300 KB) | Large (~25 MB shared lib) | Medium (~5 MB shared lib) | Large (~15 MB shared lib) |
| Cross-platform | Yes | Yes (bundled lib) | Yes (bundled lib) | Linux-focused |
| Maintenance | Low activity | Active | Active | Active |
| crates.io downloads | ~300K | ~500K | ~100K | ~50K |

### Recommendation: pdf-extract for v1, pdfium-render for Future

**For this spec (v1 conversion)**: use `pdf-extract`. It is pure Rust, extracts text content from PDF pages, and has no external shared library dependency. Its output is plain text per page — sufficient for producing readable EPUB content. It does not provide character-level position or font metadata, which limits structural detection (headings, lists). For v1, heading detection will use heuristics on the extracted text (blank-line separation, short uppercase lines, etc.).

**Limitations of pdf-extract**:
- No font size/weight information → heading detection is heuristic-only.
- No image extraction → images require a separate approach (see Images section).
- No character position data → column layout detection is not possible.
- Some PDFs with complex encodings may produce garbled text.

**For future versions**: `pdfium-render` provides character-level position and font metadata, enabling precise heading detection, column layout handling, and image extraction. It should be adopted when the Library screen's page preview feature is built (which also needs a rendering engine). The conversion engine should be designed with a `PdfTextExtractor` trait so the backend can be swapped without changing the pipeline.

### pdf-extract Capabilities

```rust
use pdf_extract::extract_text;

// Extract all text from a PDF
let text = extract_text("file.pdf")?;
// Returns a single String with page breaks as form-feed characters (\x0C)

// Extract text page by page
use pdf_extract::extract_text_from_mem;
let bytes = std::fs::read("file.pdf")?;
let text = extract_text_from_mem(&bytes)?;
// Split by \x0C to get per-page text
let pages: Vec<&str> = text.split('\x0C').collect();
```

### Text-to-Structure Heuristics (v1 Approach)

Without font metadata, structure detection relies on textual patterns:

| Structure | Heuristic |
|-----------|-----------|
| Paragraph boundaries | Two or more consecutive newlines |
| Headings (level 1) | Short line (< 80 chars), preceded by 2+ blank lines, all uppercase or title case, not ending with a period |
| Headings (level 2) | Short line, preceded by 1+ blank lines, title case, not ending with a period |
| Bullet lists | Lines starting with `•`, `–`, `-`, `*` followed by a space |
| Numbered lists | Lines starting with `1.`, `2.`, `(a)`, `(1)` etc. |
| Page breaks | Form-feed character (`\x0C`) from pdf-extract |
| Blank pages | Pages with only whitespace after extraction |
| Footnotes | Lines starting with a superscript number pattern at the bottom of a page (best-effort) |

These heuristics are imperfect but produce readable output for the majority of text-heavy PDFs (technical books, novels, reports). Users can adjust structure detection settings per spec 02's Settings screen.

---

## EPUB Generation in Rust

### Library Comparison

| Criterion | epub-builder | epub-rs | Manual ZIP construction |
|-----------|-------------|---------|------------------------|
| EPUB 2 support | Yes | Read-only | Manual |
| EPUB 3 support | Yes | Read-only | Manual |
| API ergonomics | Builder pattern | Reader, not writer | Low-level |
| TOC generation | Yes (NCX + XHTML nav) | No (read-only) | Manual |
| Image embedding | Yes | No | Manual |
| CSS embedding | Yes | No | Manual |
| Metadata (title, author, etc.) | Yes | Read-only | Manual |
| Maintenance | Moderate | Low | N/A |
| crates.io downloads | ~100K | ~80K | N/A |

### Recommendation: epub-builder

`epub-builder` is the only actively-maintained Rust crate that generates EPUB files. It supports both EPUB 2 and EPUB 3, handles the complex EPUB container structure (META-INF, OPF, NCX, XHTML navigation), and provides a clean builder API.

### epub-builder Capabilities

```rust
use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipLibrary};
use epub_builder::EpubVersion;

let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;

// Set metadata
builder.metadata("title", "Design Patterns")?;
builder.metadata("author", "Gang of Four")?;
builder.epub_version(EpubVersion::V30);

// Add CSS stylesheet
builder.stylesheet(css_content.as_bytes())?;

// Add content
builder.add_content(
    EpubContent::new("content.xhtml", content_html.as_bytes())
        .title("Content")
        .reftype(ReferenceType::Text),
)?;

// Add an image
builder.add_resource("images/fig1.png", image_bytes, "image/png")?;

// Generate the EPUB
let mut output = Vec::new();
builder.generate(&mut output)?;
std::fs::write("output.epub", &output)?;
```

### EPUB Structure Generated

```
output.epub (ZIP archive)
├── mimetype                          # "application/epub+zip" (uncompressed)
├── META-INF/
│   └── container.xml                 # Points to content.opf
├── OEBPS/
│   ├── content.opf                   # Package document (metadata, manifest, spine)
│   ├── toc.ncx                       # NCX navigation (EPUB 2)
│   ├── nav.xhtml                     # XHTML navigation (EPUB 3)
│   ├── stylesheet.css                # Embedded CSS
│   ├── content.xhtml                 # All structured content in a single file
│   └── images/
│       ├── fig1.png
│       └── fig2.jpg
```

---

## Image Extraction from PDFs

### The Challenge

`pdf-extract` does not extract images. PDF images are stored as XObject streams within the page content, encoded in various formats (JPEG, JPEG2000, CCITT fax, raw bitmap with optional filters like FlateDecode).

### Approaches

| Approach | Complexity | Quality | Dependencies |
|----------|-----------|---------|-------------|
| lopdf XObject parsing | Medium | Good for JPEG | None (already a dependency) |
| pdfium-render | Low | Excellent | pdfium shared library (~25 MB) |
| External CLI (mutool, pdftoppm) | Low | Excellent | System dependency |

### Recommendation: lopdf XObject Parsing for v1

`lopdf` can access XObject image streams directly from the PDF object tree. For JPEG images (the most common format in PDFs), the stream bytes can be written directly to a `.jpg` file. For other formats (FlateDecode-compressed raw bitmaps), decoding and re-encoding to PNG/JPEG is needed using the `image` crate.

```rust
use lopdf::Document;

let doc = Document::load("file.pdf")?;
for (page_num, page_id) in doc.get_pages() {
    let resources = doc.get_page_resources(page_id);
    // Iterate XObject entries, filter for Image subtypes
    // Extract stream bytes, detect encoding, save as image file
}
```

This approach keeps the dependency footprint small (only adds the `image` crate) and handles the majority of embedded PDF images. Complex image formats (JPEG2000, JBIG2) may not be supported in v1 — these are logged as warnings and skipped.

---

## Conversion Pipeline Architecture

### Async Command with Progress Events

Tauri IPC commands are synchronous by default (blocking the command thread). Long-running conversion must not freeze the UI. Tauri provides two mechanisms:

1. **Async commands**: annotate with `async` + return a `Result`. Runs on the async runtime without blocking other commands.
2. **Event emission**: the Rust backend emits named events to the frontend via `app_handle.emit("event-name", payload)`. The frontend listens with `listen("event-name", callback)`.

```rust
#[tauri::command]
async fn convert_pdf(
    app: tauri::AppHandle,
    path: String,
    options: ConversionOptions,
) -> Result<ConversionResult, String> {
    // Emit progress events during conversion
    app.emit("conversion-progress", ProgressPayload {
        path: path.clone(),
        stage: "extracting_text".into(),
        percent: 10,
        message: "Extracting text from pages 1–50...".into(),
    }).ok();

    // ... do work ...

    Ok(ConversionResult { output_path, images })
}
```

### Cancellation

Tauri does not provide built-in command cancellation. The standard pattern is:
- Store a `CancellationToken` (an `Arc<AtomicBool>`) keyed by job ID.
- The conversion loop checks the token periodically.
- A separate `cancel_conversion` command sets the token to `true`.

```rust
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::collections::HashMap;
use tauri::State;
use std::sync::Mutex;

struct ConversionState {
    cancel_tokens: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

#[tauri::command]
fn cancel_conversion(
    state: State<'_, ConversionState>,
    path: String,
) -> Result<(), String> {
    if let Some(token) = state.cancel_tokens.lock().unwrap().get(&path) {
        token.store(true, Ordering::Relaxed);
    }
    Ok(())
}
```

### Queue Management

The frontend manages the conversion queue. Files are converted one at a time (sequential queue). The frontend:
1. Picks the next `ready` file from the queue.
2. Calls `convert_pdf` for that file.
3. Listens for progress events.
4. On completion or error, moves to the next file.

Parallel conversion is deferred — PDF text extraction is CPU-intensive and would compete for resources on most user machines.

---

## EPUB CSS for Readability

The generated EPUB includes a default CSS stylesheet that follows the readability-first principle from spec 02. Settings from the global/per-document configuration are injected into the CSS.

```css
body {
    font-family: serif;           /* or user setting */
    font-size: 1em;               /* relative, overridable by reader */
    line-height: 1.5;             /* user setting */
    margin: 1em;                  /* user setting */
    text-align: justify;
    orphans: 2;
    widows: 2;
}

h1 { font-size: 1.6em; margin-top: 2em; margin-bottom: 0.5em; page-break-before: always; }
h2 { font-size: 1.3em; margin-top: 1.5em; margin-bottom: 0.4em; }
h3 { font-size: 1.1em; margin-top: 1.2em; margin-bottom: 0.3em; }

p { margin-top: 0.3em; margin-bottom: 0.3em; text-indent: 1.5em; }
p:first-of-type { text-indent: 0; }

ul, ol { margin-left: 1.5em; }
li { margin-bottom: 0.2em; }

img { max-width: 100%; height: auto; }
```

---

## Conversion Performance Expectations

| PDF Size | Pages | Expected Conversion Time | Bottleneck |
|----------|-------|-------------------------|------------|
| < 5 MB | < 100 | 1–5 seconds | Text extraction |
| 5–20 MB | 100–500 | 5–30 seconds | Text extraction + image processing |
| 20–100 MB | 500–2000 | 30–120 seconds | Image extraction and re-encoding |
| > 100 MB | 2000+ | 2–5 minutes | All phases |

Text extraction is CPU-bound. Image extraction and re-encoding is both CPU- and I/O-bound. EPUB generation (ZIP assembly) is fast (< 1 second for most files).

---

## References

- [pdf-extract crate](https://docs.rs/pdf-extract/)
- [epub-builder crate](https://docs.rs/epub-builder/)
- [lopdf image XObject handling](https://docs.rs/lopdf/latest/lopdf/struct.Document.html)
- [image crate (encoding/decoding)](https://docs.rs/image/)
- [Tauri v2 Events](https://v2.tauri.app/develop/calling-rust/#event-system)
- [EPUB 3.3 Specification](https://www.w3.org/TR/epub-33/)
- [EPUB 2.0.1 Specification](http://idpf.org/epub/201)
