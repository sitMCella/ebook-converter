# 03 — Import PDF: Plan

## Architectural Approach

The import feature spans the Rust backend (PDF validation and metadata extraction) and the React frontend (Import screen UI, state management, drag-and-drop). Communication happens via Tauri IPC commands.

The Import screen operates as a **staging area**: files are validated and previewed before being explicitly imported to the library. The staging state (`stagedFiles` Map) is separate from the library state (`files` Map), ensuring the import list shows only pending files, not the full library.

```
┌──────────────────────────────────────────────────┐
│                    Frontend                       │
│                                                   │
│  ┌──────────┐  ┌────────────┐  ┌──────────────┐  │
│  │  Import   │  │  Import    │  │  Tauri       │  │
│  │  Screen   │──│  Store     │──│  Bridge      │  │
│  │  (React)  │  │  (state)   │  │  (IPC calls) │  │
│  └──────────┘  └────────────┘  └──────┬───────┘  │
│                                        │          │
│  State: stagedFiles (staging)          │          │
│         files (library)                │          │
│         selectedPaths (UI selection)   │          │
│                                        │          │
├────────────────────────────────────────┼──────────┤
│                 Tauri IPC              │          │
├────────────────────────────────────────┼──────────┤
│                                        │          │
│  ┌─────────────────────────────────────▼───────┐  │
│  │              Rust Backend                    │  │
│  │  ┌─────────────────┐  ┌──────────────────┐  │  │
│  │  │  IPC Commands    │  │  PDF Service     │  │  │
│  │  │  validate_pdf    │──│  (lopdf)         │  │  │
│  │  │  get_pdf_metadata│  │                  │  │  │
│  │  │  import_pdf      │  │  Storage Service │  │  │
│  │  │  save_metadata   │  │                  │  │  │
│  │  └─────────────────┘  └──────────────────┘  │  │
│  └─────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────┘
```

## Key Decisions

### D1: Two-Phase Import (Staging → Library)

Files go through two phases:

1. **Staging** — user drops/browses files, they appear in the staging list. The backend validates the PDF and extracts metadata. No storage copy or persistence happens yet.
2. **Import to library** — the user clicks "Import to library". The backend copies the PDF to managed storage, assigns a book ID, and saves metadata. The file moves from `stagedFiles` to `files` in the React state.

This separation ensures the import list shows only pending files, not the full library. Removing a staged file is non-destructive (just an unstage), while removing a library file deletes it from storage.

### D2: lopdf for PDF Parsing

Use the `lopdf` crate for validation and metadata extraction. It is pure Rust, lightweight, and covers all import-phase requirements. Page rendering (needed later for the Library screen) will use `pdfium-render` in a future spec.

### D3: Two Separate IPC Commands for Validation

Split the backend into two commands rather than one monolithic call:

1. **`validate_pdf`** — takes a file path, returns a validation result (`valid`, `encrypted`, or `error` with message). Fast — only reads enough of the file to determine validity.
2. **`get_pdf_metadata`** — takes a file path, returns extracted metadata. Only called for valid files.

This separation allows the frontend to show the `Ready` status quickly (after validation) and load metadata asynchronously without blocking the staging list update.

### D4: Frontend State Management with React Context

The staging and library state share a single `ImportContext` with `useReducer`:

- `stagedFiles: Map<string, StagedFile>` — files in the staging area (import screen)
- `files: Map<string, ImportedFile>` — files in the library (library screen)
- `selectedPaths: Set<string>` — UI checkbox state (references staged files)

Staging actions: `STAGE_FILES`, `UNSTAGE_FILES`, `UPDATE_STAGED_STATUS`, `SET_STAGED_METADATA`.
Library actions: `IMPORT_TO_LIBRARY`, `ADD_FILES`, `REMOVE_FILES`, `UPDATE_STATUS`, `SET_METADATA`, `LOAD_LIBRARY`.

### D5: Hybrid Drag-and-Drop

Combine Tauri's native `onDragDropEvent` (for file paths) with HTML5 drag events (for element-level visual feedback on the drop zone):

- Tauri's window-level event captures dropped file paths.
- HTML5 `dragenter`/`dragleave`/`dragover` on the drop zone element control the visual states (border colour, background tint, scale).
- On drop, filter to `.pdf` extensions before processing.

### D6: Extend Existing Tauri Bridge

Extend `src/lib/tauri.js` with new functions rather than creating a separate module:

- `openPdfFiles()` — opens the native dialog with multi-select and PDF filter.
- `validatePdf(path)` — calls the `validate_pdf` IPC command.
- `getPdfMetadata(path)` — calls the `get_pdf_metadata` IPC command.
- `getFileSize(path)` — gets file size via the filesystem plugin's `stat()`.
- `importPdf(sourcePath)` — copies PDF to managed storage, returns book ID and stored path.
- `saveBookMetadata(metadata)` — persists book metadata to the book's storage directory.

### D7: Application Shell and Routing

The Import screen is the first screen in the sidebar navigation defined in spec 02. This spec introduces the application shell (sidebar + main content area) and client-side routing with `react-router-dom`:

- `/` or `/import` — Import screen (this spec)
- `/library` — Library screen (spec 06)
- `/converted` — Converted screen (spec 07)
- `/settings` — Settings screen (spec 08)

The sidebar navigation and layout structure follow spec 02 exactly.

### D8: Toast Notifications with sonner

Use the `sonner` library for toast notifications. It supports stacking (max 3), auto-dismiss, and Tailwind CSS theming — all required by spec 02.

## Directory Structure Changes

```
src/
├── components/
│   ├── layout/
│   │   ├── AppShell.jsx         # Sidebar + main content layout
│   │   ├── Sidebar.jsx          # Navigation sidebar
│   │   └── StatusBar.jsx        # Bottom status bar (hidden initially)
│   ├── import/
│   │   ├── ImportScreen.jsx     # Import screen container
│   │   ├── DropZone.jsx         # Drag-and-drop zone
│   │   ├── ImportList.jsx       # Staging list
│   │   ├── ImportListRow.jsx    # Single staged file row
│   │   ├── BatchActions.jsx     # Remove/Import to library buttons
│   │   └── StatusBadge.jsx      # Status pill component
│   └── ui/
│       ├── Button.jsx           # Reusable button (primary/secondary)
│       ├── Checkbox.jsx         # Styled checkbox
│       └── ConfirmDialog.jsx    # Confirmation modal
├── contexts/
│   └── ImportContext.jsx        # Staging + library state + provider
├── hooks/
│   ├── useDragDrop.js           # Drag-and-drop hook (Tauri + HTML5)
│   └── useImport.js             # Staging + import orchestration hook
├── lib/
│   ├── tauri.js                 # Extended with PDF-specific functions
│   └── format.js                # File size formatting utility
└── App.jsx                      # Updated with routing + AppShell

src-tauri/src/
├── main.rs                      # Unchanged
├── lib.rs                       # Updated: register PDF commands
└── pdf.rs                       # New: PDF validation + metadata
```

## Integration Points

### Frontend → Tauri IPC

```javascript
// Stage PDF files (validation + metadata)
const validation = await invoke('validate_pdf', { path });
// Returns: { status: "valid" } | { status: "encrypted" } | { status: "error", message: "..." }

const metadata = await invoke('get_pdf_metadata', { path });
// Returns: { title?, author?, pageCount, pdfVersion, createdDate?, modifiedDate?, producer? }

// Import to library (storage + persistence)
const { bookId, storedPdfPath } = await importPdf(sourcePath);
await saveBookMetadata({ bookId, storedPdfPath, ... });
```

### Tauri → Frontend (Events)

Drag-and-drop events flow via Tauri's window event listener:

```javascript
import { getCurrentWindow } from '@tauri-apps/api/window';

getCurrentWindow().onDragDropEvent((event) => {
  // event.payload.type: 'over' | 'drop' | 'leave'
  // event.payload.paths: string[] (on 'drop')
});
```

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| lopdf fails on some PDFs | lopdf is mature and handles most PDF variants; add a fallback error message for edge cases |
| Large PDF blocks the IPC thread | lopdf metadata extraction reads only the header/trailer, not full content; Tauri IPC runs commands off the main thread by default |
| Drag-and-drop conflicts between Tauri and HTML5 APIs | The hybrid approach separates concerns: Tauri for paths, HTML5 for visual feedback |
| Staging list grows very large | Frontend only holds metadata in memory (~1 KB per file); no file content is stored in React state |
| Confusing import page with library | Staging vs. library separation ensures the import list only shows pending files |
