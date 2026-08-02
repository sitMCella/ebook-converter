# 03 — Import PDF: Plan

## Architectural Approach

The import feature spans the Rust backend (PDF validation and metadata extraction) and the React frontend (Import screen UI, state management, drag-and-drop). Communication happens via Tauri IPC commands.

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
│  │  └─────────────────┘  └──────────────────┘  │  │
│  └─────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────┘
```

## Key Decisions

### D1: lopdf for PDF Parsing

Use the `lopdf` crate for validation and metadata extraction. It is pure Rust, lightweight, and covers all import-phase requirements. Page rendering (needed later for the Library screen) will use `pdfium-render` in a future spec.

### D2: Two Separate IPC Commands

Split the backend into two commands rather than one monolithic call:

1. **`validate_pdf`** — takes a file path, returns a validation result (`valid`, `encrypted`, or `error` with message). Fast — only reads enough of the file to determine validity.
2. **`get_pdf_metadata`** — takes a file path, returns extracted metadata. Only called for valid files.

This separation allows the frontend to show the `Ready` status quickly (after validation) and load metadata asynchronously without blocking the import list update.

### D3: Frontend State Management with React Context

The import list is a shared state needed by the Import screen and potentially the Library and Converting screens. Use a React Context + `useReducer` pattern:

- `ImportContext` provides the import list and dispatch actions.
- Actions: `ADD_FILES`, `REMOVE_FILES`, `UPDATE_STATUS`, `SET_METADATA`.
- State shape: `Map<string, ImportedFile>` keyed by absolute file path.

No external state library (Redux, Zustand) — the state is simple enough for React's built-in tools, and spec 02 states the list is session-only (not persisted).

### D4: Hybrid Drag-and-Drop

Combine Tauri's native `onDragDropEvent` (for file paths) with HTML5 drag events (for element-level visual feedback on the drop zone):

- Tauri's window-level event captures dropped file paths.
- HTML5 `dragenter`/`dragleave`/`dragover` on the drop zone element control the visual states (border colour, background tint, scale).
- On drop, filter to `.pdf` extensions before processing.

### D5: Extend Existing Tauri Bridge

Extend `src/lib/tauri.js` with new functions rather than creating a separate module:

- `openPdfFiles()` — opens the native dialog with multi-select and PDF filter.
- `validatePdf(path)` — calls the `validate_pdf` IPC command.
- `getPdfMetadata(path)` — calls the `get_pdf_metadata` IPC command.
- `getFileSize(path)` — gets file size via the filesystem plugin's `stat()`.

### D6: Application Shell and Routing

The Import screen is the first screen in the sidebar navigation defined in spec 02. This spec introduces the application shell (sidebar + main content area) and client-side routing with `react-router-dom`:

- `/` or `/import` — Import screen (this spec)
- `/library` — Library screen (future spec, placeholder)
- `/converted` — Converted screen (future spec, placeholder)
- `/settings` — Settings screen (future spec, placeholder)

The sidebar navigation and layout structure follow spec 02 exactly.

### D7: Toast Notifications with sonner

Use the `sonner` library for toast notifications. It supports stacking (max 3), auto-dismiss, manual dismiss, and Tailwind CSS theming — all required by spec 02.

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
│   │   ├── ImportList.jsx       # Import list table
│   │   ├── ImportListRow.jsx    # Single file row
│   │   ├── BatchActions.jsx     # Remove/Convert selected buttons
│   │   └── StatusBadge.jsx      # Status pill component
│   └── ui/
│       ├── Button.jsx           # Reusable button (primary/secondary)
│       ├── Checkbox.jsx         # Styled checkbox
│       └── ConfirmDialog.jsx    # Confirmation modal
├── contexts/
│   └── ImportContext.jsx        # Import list state + provider
├── hooks/
│   ├── useDragDrop.js           # Drag-and-drop hook (Tauri + HTML5)
│   └── useImport.js             # Import orchestration hook
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
// Import PDF files
const paths = await openPdfFiles();

// Validate each file
const validation = await invoke('validate_pdf', { path });
// Returns: { status: "valid" } | { status: "encrypted" } | { status: "error", message: "..." }

// Get metadata for valid files
const metadata = await invoke('get_pdf_metadata', { path });
// Returns: { title?, author?, pageCount, pdfVersion, createdDate?, modifiedDate?, producer? }
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
| Import list grows very large | Frontend only holds metadata in memory (~1 KB per file); no file content is stored in React state |
