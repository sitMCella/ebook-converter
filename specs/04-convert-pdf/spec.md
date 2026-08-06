# 04 — Convert PDF to EPUB

## Goal

Convert imported PDF files into well-structured EPUB ebooks, with real-time progress feedback, a conversion queue, and per-document configuration — implementing the Converting screen (screen 3) and the core conversion pipeline from the Rust backend through to frontend state.

## Background

The import pipeline (spec 03) brings PDF files into the application, validates them, and extracts metadata. The UI/UX design (spec 02) defines the Converting screen layout: an active/queued conversion list with progress bars, a real-time conversion log, and a completed section. The Settings screen (spec 02) defines conversion parameters (structure detection, images, output format, page handling). This spec covers everything needed to take a `Ready` file from the import list and produce an EPUB file on disk.

## Functional Requirements

### FR-1: Single-File Conversion

A user can convert a single PDF to EPUB from the Library detail panel by clicking "Convert to EPUB". The conversion uses the effective settings (global defaults merged with any per-document overrides). The resulting EPUB is written to the configured output folder.

### FR-2: Batch Conversion

A user can select multiple files in the import list and click "Convert selected" to queue them all for conversion. Files are converted sequentially (one at a time). The queue is displayed on the Converting screen.

### FR-3: Conversion Pipeline

The Rust backend performs conversion in the following stages:

1. **Text extraction** — Extract text content from each PDF page. Split by page boundaries.
2. **Structure detection** — Identify headings, paragraphs, lists, and footnotes using text heuristics. Configurable via settings.
3. **Image extraction** — Extract embedded images from the PDF. Optionally resize and re-encode. Configurable via settings.
4. **EPUB assembly** — Generate the EPUB file with metadata, structured content, images, CSS, and navigation. Write to the output folder.

### FR-4: Progress Reporting

During conversion, the Rust backend emits progress events to the frontend. Each event includes:
- The file path being converted.
- The current stage name.
- A progress percentage (0–100).
- A human-readable log message.

The frontend displays progress on the Converting screen:
- A progress bar for the active file.
- Log messages in the conversion log panel.
- Percentage display next to the file name.

### FR-5: Conversion Log

The Converting screen includes a log panel showing real-time output from the conversion engine. Log entries include:
- `Extracting text from pages N–M...`
- `Detecting headings and structure...`
- `Extracting images (X of Y)...`
- `Generating EPUB structure...`
- `Writing EPUB file...`
- `Conversion complete.`
- `Error: <message>` (in danger colour on failure).

The log auto-scrolls to the latest entry.

### FR-6: Cancellation

The user can cancel active and queued conversions:
- **Cancel all** button in the Converting screen header. Shows a confirmation dialog: "Cancel N remaining conversion(s)? Files already converted are not affected."
- Cancelling stops the active conversion mid-process and removes all queued items. Partially-written output files are cleaned up.

### FR-7: Conversion Completion

When a file finishes converting:
- Its status changes from `Converting` to `Converted` in the import list.
- It moves from the active/queued section to the completed section on the Converting screen.
- The next queued file begins conversion automatically.

When all conversions complete:
- The header changes to "Conversion complete".
- The "Cancel all" button is hidden (no active work remains).

### FR-8: Error Handling

If conversion fails for a file:
- Its status changes to `Error` in the import list.
- The Converting screen shows an `Error` badge on that file's row.
- The conversion log shows the error message in danger colour.
- An expandable error detail is available inline on the row.
- Remaining queued files continue converting — one failure does not stop the queue.

### FR-9: Output File Naming

The output EPUB file is named after the source PDF file, with the extension changed:
- `Design patterns.pdf` → `Design patterns.epub`
- If a file with that name already exists in the output folder, a numeric suffix is appended: `Design patterns (1).epub`.

### FR-10: Settings Integration

Conversion uses the effective settings for each document, computed by merging per-document overrides on top of global defaults (see spec 02 "Settings Architecture"). The conversion command receives the full resolved settings as a parameter.

Applicable setting groups:
- **Structure detection**: detect headings, detect footnotes, heading level threshold, paragraph detection, list detection.
- **Images**: extract images, image quality, max image width, convert to WebP.
- **Output format**: EPUB version, embed fonts, font family, base font size, line height, margins.
- **Page handling**: skip blank pages, page range.
- **Output location**: default output folder (global only).

## Non-Functional Requirements

### NFR-1: Conversion Performance

Text extraction and EPUB generation for a 100-page PDF (< 5 MB, text-heavy) should complete in under 10 seconds on a modern machine. The UI must remain fully responsive during conversion — all Rust work runs asynchronously off the main thread.

### NFR-2: Memory Efficiency

The conversion pipeline processes pages in batches rather than loading the entire PDF content into memory. Peak memory usage should stay under 500 MB for PDFs up to 100 MB.

### NFR-3: Output Quality

Generated EPUBs must:
- Be valid EPUB 2 or EPUB 3 files (pass basic structural validation).
- Render correctly in major e-reader apps (Apple Books, Calibre, Kobo, Kindle via conversion).
- Preserve the reading order of the source PDF.
- Use semantic HTML (`<h1>`–`<h6>`, `<p>`, `<ul>`, `<ol>`, `<img>`) in the content XHTML file.

### NFR-4: Disk Space

The application checks available disk space before starting conversion. If insufficient space is estimated (output size > available space), a toast notification is shown: "Not enough disk space to save the EPUB. Free some space and try again."

### NFR-5: Accessibility

The Converting screen is keyboard navigable. The progress bar has an `aria-valuenow` and `aria-label`. Log entries are announced to screen readers via an `aria-live` region. The "Cancel all" confirmation dialog traps focus and closes on Escape.

## Out of Scope

- EPUB preview / reading (Converted screen, future spec).
- Per-document conversion options UI in Library detail panel (Library screen, future spec).
- Settings screen UI (future spec — this spec reads settings from a config file or defaults).
- Font embedding (setting exists but implementation is deferred — always uses system/reader fonts in v1).
- Footnote detection (setting exists but implementation is deferred to a future version with font-aware extraction).
- Advanced column layout / table detection (requires font-position-aware extraction).
- Parallel conversion (CPU contention makes sequential the better default).
- Conversion history / reconversion tracking.
