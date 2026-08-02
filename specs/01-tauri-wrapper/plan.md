# 01 — Tauri Wrapper: Plan

## Architectural Approach

Tauri v2 wraps the existing Vite dev server (in development) or the static `dist/` build output (in production) inside a native webview. The Rust backend handles filesystem operations and exposes them to the frontend via Tauri's IPC command system.

```
┌─────────────────────────────────────┐
│         Native OS Window            │
│  ┌───────────────────────────────┐  │
│  │        Webview (OS)           │  │
│  │  ┌─────────────────────────┐  │  │
│  │  │   React + Vite App      │  │  │
│  │  │   (existing frontend)   │  │  │
│  │  └─────────────────────────┘  │  │
│  └───────────────────────────────┘  │
│         Tauri Rust Core             │
│  ┌───────────────────────────────┐  │
│  │  Commands: open_file,         │  │
│  │  save_file, file_dialog       │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

## Key Decisions

### D1: Tauri v2 (not v1)

Tauri v2 is the current stable release. It uses a plugin-based architecture for capabilities like file dialogs and filesystem access, which aligns with the security-scoped access model in FR-2/FR-3.

### D2: Rust Backend for File I/O Only

The Rust side stays minimal — its only job is bridging native file dialogs and filesystem read/write to the frontend. Ebook conversion logic (if any) lives in the frontend or is delegated to a WASM library in a future spec.

### D3: Vite Integration via `devUrl` / `frontendDist`

Tauri's config points to:
- `devUrl: "http://localhost:5173"` in development (Vite's dev server)
- `frontendDist: "../dist"` in production (Vite's build output)

No changes needed to `vite.config.js`.

### D4: Directory Structure

Tauri files live in a `src-tauri/` directory at the project root, following Tauri's convention:

```
ebook-converter/
├── src/                  # Existing React frontend
├── src-tauri/            # New Tauri backend
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   ├── icons/
│   └── src/
│       ├── main.rs
│       └── lib.rs
├── package.json          # Updated with Tauri scripts
├── vite.config.js        # Unchanged
└── index.html            # Unchanged
```

### D5: npm Scripts

Add Tauri-specific scripts to `package.json`:

- `tauri dev` — starts Vite + Tauri together in dev mode
- `tauri build` — builds the frontend then packages the native app

These are added via `@tauri-apps/cli` as a dev dependency.

## Integration Points

### Frontend → Tauri (IPC)

The frontend invokes Tauri commands using `@tauri-apps/api`:

```js
import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
```

### Tauri → Frontend (Events)

Tauri can emit events to the frontend for progress updates during file operations.

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Rust toolchain unfamiliar to team | Rust side is < 50 lines; the `tauri init` scaffold covers most of it |
| Webview rendering differences | Test on all three OS targets; Tauri uses the OS webview (WebKit on macOS, WebView2 on Windows, WebKitGTK on Linux) |
| Large binary size | Tauri binaries are inherently small (< 10 MB); monitor with `cargo bloat` if needed |
| Vite 8 compatibility | Tauri v2 is Vite-agnostic — it just needs a URL or a directory of static files |
