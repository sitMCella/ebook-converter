# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run Commands

### Frontend (browser-only, no Rust needed)
```bash
npm run dev              # Vite dev server at http://localhost:5173
npm run build            # Production build
```

### Desktop App (Tauri + Rust)
```bash
npm run tauri dev        # Native window with HMR (first run compiles Rust ~1-2 min)
npm run tauri build      # Distributable package for current platform
```

### Testing
```bash
npm run test             # Vitest unit tests (all)
npm run test:watch       # Vitest in watch mode
npx vitest run src/components/import/DropZone.test.jsx  # Single test file

npm run test:e2e         # Playwright e2e tests (auto-starts Vite)
npx playwright test --grep "triggers file dialog"       # Single e2e test by name
```

First-time Playwright setup: `npx playwright install chromium`

### Rust backend tests
```bash
cd src-tauri && cargo test
```

### Linting
```bash
npm run lint             # Oxlint
```

## Architecture

Tauri v2 desktop app: React SPA frontend rendered in an OS webview, with a Rust backend for native operations.

### Tauri Bridge Pattern
`src/lib/tauri.js` is the boundary between frontend and backend. It detects the runtime (`window.__TAURI_INTERNALS__`) and either calls Tauri IPC commands or falls back to browser APIs (file inputs, downloads). All native operations go through this module — never import `@tauri-apps/` packages directly from components.

The Rust backend exposes two IPC commands registered in `src-tauri/src/lib.rs`:
- `validate_pdf` — returns a tagged union `{status: "valid"|"encrypted"|"error"}`
- `get_pdf_metadata` — returns camelCase metadata (pageCount, pdfVersion, fileSize, etc.)

### Frontend State
Import state lives in `src/contexts/ImportContext.jsx` using `useReducer` with two Maps:
- `stagedFiles: Map<path, file>` — files in the Import screen's staging area (session-only, not persisted)
- `files: Map<path, file>` — files in the library (persisted via `metadata.json` in managed storage)
- `selectedPaths: Set<path>` — UI checkbox selections (references staged files)

Staging actions: `STAGE_FILES`, `UNSTAGE_FILES`, `UPDATE_STAGED_STATUS`, `SET_STAGED_METADATA`.
Library actions: `ADD_FILES`, `REMOVE_FILES`, `UPDATE_STATUS`, `SET_METADATA`, `LOAD_LIBRARY`.
Bridge action: `IMPORT_TO_LIBRARY` — moves a file from staging to library with bookId and storedPdfPath.

### Import Flow (Two-Phase)
`src/hooks/useImport.js` exposes two functions:
- `stageFiles(paths)` — validates PDFs and adds to staging area. Checks for duplicates against both staged files and library. Does NOT copy to storage.
- `importStagedFiles(paths)` — copies staged files to managed storage via `importPdf`, saves metadata, and dispatches `IMPORT_TO_LIBRARY` to move from staging to library.

`src/hooks/useDragDrop.js` listens to Tauri drag-drop events (no-op in browser) and calls `stageFiles`.

### Routing
React Router in `src/App.jsx`. Routes: `/import` (default, redirected from `/`), `/library`, `/converting`, `/converted`, `/settings`.

### Styling
Tailwind CSS v4 with CSS custom properties for theming in `src/index.css`. Light/dark mode via `prefers-color-scheme`. All colors reference `var(--*)` tokens — use these, not raw color values.

## Key Conventions

- JSX files use `.jsx` extension, not `.tsx` — the project does not use TypeScript on the frontend.
- Rust structs serialize to camelCase for the JS bridge (`#[serde(rename_all = "camelCase")]`).
- Tauri commands return `Result<T, String>` — errors are plain strings.
- PDF processing uses `lopdf` crate; only reads headers/metadata, never full page content.
- Vitest config must explicitly exclude `e2e/**` and `.claude/**` (overrides defaults in `vite.config.js`).
- E2e tests cannot use `Meta+key` / `Ctrl+key` shortcuts directly — Chromium intercepts them. Use `page.evaluate` with synthetic `KeyboardEvent` dispatch instead.
