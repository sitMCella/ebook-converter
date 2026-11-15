# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2026-11-15

### Added

- `Include table of contents` page-handling option (off by default) to keep the PDF's table of contents pages in the EPUB, configurable in Settings and per book in the Library.
- `Keep notes` page-handling option (off by default) that keeps notes printed at the bottom of PDF pages, rendered as styled note blocks right after the paragraph that cites them.
- `Keep chapter labels` page-handling option (on by default) that keeps chapter labels ("CHAPTER 1", "Part II", ...) on the same page as the chapter title, or removes them.
- Output Directory section in README documenting where imported PDFs and converted EPUBs are stored.

### Changed

- Text extraction migrated to MuPDF's structured text page API (blocks, lines and characters with bounding box and font info).
- Front matter before the table of contents (half-title, copyright, back-cover blurb) is no longer included in the EPUB text; the table of contents is detected from PDF bookmarks, falling back to text detection.
- Heading detection now based on actual font size instead of capitalization.
- Images are placed next to their captions instead of at the end of their PDF page.
- Code blocks are left-aligned and never hyphenated.

### Fixed

- Footers leaking into body text mid-page, not only at page boundaries.
- Hyphenated line wraps left split instead of rejoined.
- Multi-line code samples flattened into run-on prose paragraphs.
- Inline code identifiers split mid-word across a line wrap.
- Body-sized ALL-CAPS labels misclassified as chapter headings.
- Chapter labels stranded at the bottom of the previous page.
- Cover art split into several tiles now rasterized as a whole page instead of using a single tile.
- Cover images repeated in the EPUB body.
- Cover preview blocked by CSP for `data:` image URLs.

## [0.1.1] - 2026-08-13

### Added

- Image extraction and embedding in EPUB output using MuPDF.
- Footnote detection and EPUB endnotes rendering.
- Per-page progress reporting during image extraction.
- `keepPageBreaks` option in EPUB generator.
- Documentation section in README linking to GitHub Wiki.

### Changed

- Replaced `pdf-extract` with MuPDF for text extraction.
- Switched image extraction from manual `lopdf` decoding to MuPDF.
- Extracted images now scaled using CTM to preserve original display size, with percentage-based sizing.
- macOS app icons now use squircle mask for rounded Dock appearance.

### Fixed

- Cover extraction falls back to full-page rasterization for PDFs with nested XObjects.
- Duplicate log entries caused by async `useEffect` cleanup race in StrictMode.
- Footer detection generalized to handle "page | title" format and pipe separators split across lines.
- Recurring decorative images no longer included in EPUB output.
- Small icon images in source PDF no longer incorrectly constrained.
- macOS aarch64 build failure from mupdf-sys Tesseract dependency.

## [0.1.0] - 2026-08-11

### Added

- PDF import with validation and metadata extraction.
- Library management with persistent storage.
- Drag-and-drop file import (Tauri desktop).
- Cover image extraction from PDF pages using MuPDF.
- Progress bar during PDF import.
- Light and dark theme support.
