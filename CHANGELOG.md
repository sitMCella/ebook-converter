# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
