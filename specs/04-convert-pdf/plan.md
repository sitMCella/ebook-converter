# 04 — Convert PDF to EPUB: Plan

## Architectural Approach

The conversion feature spans three layers: a Rust conversion engine (text extraction, structure detection, image extraction, EPUB assembly), a Tauri IPC bridge (async command + progress events), and a React frontend (conversion queue, progress UI, Converting screen).

```
┌──────────────────────────────────────────────────────┐
│                      Frontend                         │
│                                                       │
│  ┌──────────┐  ┌──────────────┐  ┌────────────────┐  │
│  │Converting │  │ Conversion   │  │  Tauri Bridge  │  │
│  │ Screen    │──│ Context      │──│  (IPC + events)│  │
│  │ (React)   │  │ + Import Ctx │  │                │  │
│  └──────────┘  └──────────────┘  └───────┬────────┘  │
│                                           │           │
├───────────────────────────────────────────┼───────────┤
│              Tauri IPC + Events           │           │
├───────────────────────────────────────────┼───────────┤
│                                           │           │
│  ┌────────────────────────────────────────▼────────┐  │
│  │                Rust Backend                      │  │
│  │  ┌───────────────────────────────────────────┐   │  │
│  │  │  convert_pdf command (async)              │   │  │
│  │  │  ┌─────────────┐  ┌──────────────────┐    │   │  │
│  │  │  │ PDF Reader   │  │ EPUB Builder     │    │   │  │
│  │  │  │ (pdf-extract │  │ (epub-builder)   │    │   │  │
│  │  │  │  + lopdf)    │  │                  │    │   │  │
│  │  │  └─────────────┘  └──────────────────┘    │   │  │
│  │  │  ┌─────────────┐  ┌──────────────────┐    │   │  │
│  │  │  │ Structure    │  │ Image Extractor  │    │   │  │
│  │  │  │ Detector     │  │ (lopdf + image)  │    │   │  │
│  │  │  └─────────────┘  └──────────────────┘    │   │  │
│  │  └───────────────────────────────────────────┘   │  │
│  │  ┌───────────────────────────────────────────┐   │  │
│  │  │  cancel_conversion command                │   │  │
│  │  │  ConversionState (cancel tokens)          │   │  │
│  │  └───────────────────────────────────────────┘   │  │
│  └──────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────┘
```

## Key Decisions

### D1: pdf-extract for Text Extraction

Use the `pdf-extract` crate for extracting text from PDF pages. It is pure Rust (no external shared library), which simplifies cross-platform builds and keeps the binary small. Its output is plain text per page — sufficient for v1 where structure detection uses heuristics. The conversion module is designed with a trait boundary (`PdfTextExtractor`) so a future upgrade to `pdfium-render` (for font-aware extraction) can be swapped in without changing the pipeline.

### D2: epub-builder for EPUB Generation

Use the `epub-builder` crate for generating EPUB files. It handles the complex EPUB container format (META-INF, OPF, NCX/nav, XHTML content) and supports both EPUB 2 and EPUB 3. The builder pattern API maps cleanly to the conversion pipeline's output.

### D3: lopdf for Image Extraction

Reuse the existing `lopdf` dependency (already used in spec 03) to access XObject image streams from the PDF object tree. JPEG images are extracted as-is; other formats are decoded and re-encoded using the `image` crate. This avoids adding a large FFI dependency solely for image extraction.

### D4: Async Command with Event-Based Progress

The `convert_pdf` command is `async` to avoid blocking the Tauri command thread. Progress is reported via Tauri events (`app.emit("conversion-progress", payload)`) rather than return values. This decouples progress display from command completion and allows the frontend to update the UI in real time.

### D5: Frontend Queue Management

The conversion queue is managed on the frontend, not the backend. The frontend calls `convert_pdf` one file at a time, waiting for each to complete before starting the next. This keeps the backend stateless (no queue state in Rust) and gives the frontend full control over queue ordering, cancellation, and UI updates.

### D6: Separate ConversionContext

Conversion queue state (queue order, active file, completed files, log entries) lives in a new `ConversionContext` separate from `ImportContext`. Import state (file status, metadata, selection) is updated from the conversion hook via the existing `ImportContext` dispatch. This separation keeps each context focused and avoids overloading `ImportContext` with queue management.

### D7: Conversion Settings from Defaults

For v1, the Settings screen UI is not yet built (future spec). The conversion pipeline reads settings from a JSON file in the app data directory. If the file does not exist, hardcoded defaults are used. Per-document overrides are stored in `ImportContext` and merged on the frontend before calling `convert_pdf`. This allows the conversion pipeline to work immediately while the Settings UI is built later.

### D8: Sequential Conversion

Files are converted one at a time. PDF text extraction is CPU-intensive, and running multiple conversions in parallel would degrade performance on most user machines. The sequential approach also simplifies progress reporting and cancellation.

## Directory Structure Changes

```
src/
├── components/
│   ├── conversion/
│   │   ├── ConvertingScreen.jsx     # Converting screen container (screen 3)
│   │   ├── ConversionQueue.jsx      # Active + queued file list
│   │   ├── ConversionQueueRow.jsx   # Single file row with progress
│   │   ├── ConversionLog.jsx        # Real-time log panel
│   │   ├── CompletedList.jsx        # Completed conversion list
│   │   └── ProgressBar.jsx          # Reusable progress bar component
│   └── ...existing...
├── contexts/
│   ├── ImportContext.jsx             # Extended with new actions
│   └── ConversionContext.jsx         # New: queue + log state
├── hooks/
│   ├── useConversion.js              # Conversion orchestration hook
│   └── ...existing...
├── lib/
│   ├── tauri.js                      # Extended with conversion functions
│   ├── settings.js                   # Settings loading + defaults + merge
│   └── ...existing...
└── App.jsx                           # Updated: /converting route, ConversionContext provider

src-tauri/src/
├── main.rs                           # Unchanged
├── lib.rs                            # Updated: register conversion commands + state
├── pdf.rs                            # Existing: validation + metadata
└── conversion/
    ├── mod.rs                        # Conversion module root, IPC commands
    ├── pipeline.rs                   # Orchestrates the conversion stages
    ├── text_extractor.rs             # PDF text extraction (pdf-extract)
    ├── structure_detector.rs         # Heading, paragraph, list detection
    ├── image_extractor.rs            # Image extraction from PDF (lopdf + image)
    ├── epub_generator.rs             # EPUB assembly (epub-builder)
    └── css.rs                        # EPUB stylesheet generation
```

## Integration Points

### Frontend → Tauri IPC

```javascript
// Start conversion
const result = await convertPdfToEpub(path, effectiveSettings);

// Cancel active conversion
await cancelConversion(path);
```

### Tauri → Frontend (Events)

```javascript
// Listen for progress updates
const unlisten = await onConversionProgress((progress) => {
  // progress: { path, stage, percent, message }
  dispatch({ type: 'SET_CONVERSION_PROGRESS', path, percent, stage });
  queueDispatch({ type: 'ADD_LOG_ENTRY', entry: { message, level: 'info' } });
});
```

### Settings Merge

```javascript
// In useConversion.js, before calling convert_pdf
const globalSettings = await loadSettings();
const documentOverrides = importState.files.get(path)?.overrides ?? {};
const effectiveSettings = deepMerge(globalSettings, documentOverrides);
```

## Conversion Pipeline Stages

```
┌─────────────┐     ┌──────────────────┐     ┌──────────────────┐
│ 1. Extract   │────▸│ 2. Detect        │────▸│ 3. Extract       │
│    Text      │     │    Structure     │     │    Images        │
│    (0-40%)   │     │    (40-55%)      │     │    (55-75%)      │
└─────────────┘     └──────────────────┘     └──────────────────┘
                                                      │
                    ┌──────────────────┐     ┌───────▾──────────┐
                    │ 5. Write         │◂───│ 4. Generate      │
                    │    File          │     │    EPUB          │
                    │    (95-100%)     │     │    (75-95%)      │
                    └──────────────────┘     └──────────────────┘
```

Each stage checks the cancellation token before proceeding. If cancelled, partial output is cleaned up and an error is returned.

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| pdf-extract fails on some PDFs | Wrap extraction in a catch; report error per-file, continue queue |
| Image extraction produces garbled output for rare encodings | Skip unsupported image formats with a warning log entry; document limitations |
| Large PDFs exhaust memory | Process pages in batches of 50; stream images to temp files rather than holding in memory |
| epub-builder produces invalid EPUBs | Include basic structural validation in tests; test output in Apple Books and Calibre |
| Conversion takes too long for large files | Progress events keep the user informed; cancellation provides an escape hatch |
| Output folder does not exist or is not writable | Check before starting conversion; show toast with guidance to update Settings |
| Settings file is malformed | Fall back to defaults for any unparseable field; log a warning |
