# 01 — Tauri Wrapper: Data Model

## Tauri IPC Commands

Commands exposed by the Rust backend to the frontend via `invoke()`.

### `read_file`

Read raw bytes from a file path (returned from a dialog selection).

```typescript
// Frontend call
const contents: Uint8Array = await invoke('read_file', { path: string });
```

```rust
// Backend handler
#[tauri::command]
fn read_file(path: String) -> Result<Vec<u8>, String>
```

### `write_file`

Write bytes to a file path (returned from a save dialog selection).

```typescript
// Frontend call
await invoke('write_file', { path: string, contents: Uint8Array });
```

```rust
// Backend handler
#[tauri::command]
fn write_file(path: String, contents: Vec<u8>) -> Result<(), String>
```

## Frontend Interfaces

### `TauriBridge`

Abstraction layer in `src/lib/tauri.js` so the app works in both browser and Tauri contexts.

```typescript
interface FileResult {
  path: string;
  name: string;
  contents: Uint8Array;
}

interface TauriBridge {
  /** Opens a native file picker and returns the selected file's contents. */
  openFile(filters?: FileFilter[]): Promise<FileResult | null>;

  /** Opens a native save dialog and writes the provided data. */
  saveFile(data: Uint8Array, defaultName?: string, filters?: FileFilter[]): Promise<string | null>;

  /** Whether the app is running inside Tauri (vs. a browser). */
  isTauri: boolean;
}

interface FileFilter {
  name: string;        // e.g. "EPUB Files"
  extensions: string[]; // e.g. ["epub"]
}
```

## Tauri Configuration Schema

Key fields in `src-tauri/tauri.conf.json`:

```json
{
  "$schema": "https://raw.githubusercontent.com/tauri-apps/tauri/dev/crates/tauri-cli/config.schema.json",
  "identifier": "com.ebook-converter.app",
  "productName": "Ebook Converter",
  "version": "0.1.0",
  "build": {
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "Ebook Converter",
        "width": 1024,
        "height": 768,
        "minWidth": 640,
        "minHeight": 480,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": "default-src 'self'; style-src 'self' 'unsafe-inline'"
    }
  }
}
```

## Capability Permissions

`src-tauri/capabilities/default.json`:

```json
{
  "identifier": "default",
  "description": "Default permissions for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "dialog:allow-open",
    "dialog:allow-save",
    "fs:allow-read-file",
    "fs:allow-write-file"
  ]
}
```

## Cargo Dependencies

`src-tauri/Cargo.toml` additions:

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

## npm Dependencies

Added to the project's `package.json`:

```json
{
  "devDependencies": {
    "@tauri-apps/cli": "^2"
  },
  "dependencies": {
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-dialog": "^2",
    "@tauri-apps/plugin-fs": "^2"
  }
}
```
