# 01 — Tauri Wrapper

## Goal

Package the existing React + Vite ebook-converter web application as a native desktop application using Tauri v2, targeting macOS, Windows, and Linux.

## Background

The ebook-converter is currently a browser-based SPA built with React 19, Vite 8, and Tailwind CSS 4. Wrapping it with Tauri will give it native window chrome, filesystem access for reading/writing ebook files, and a distributable installer — while keeping the existing frontend code largely unchanged.

## Functional Requirements

### FR-1: Native Desktop Window

The application runs inside a native OS window with standard title bar, minimize/maximize/close controls, and proper taskbar/dock integration. The window title is "Ebook Converter".

### FR-2: File System Access

The application can open ebook files from the local filesystem via a native file picker dialog. Supported input: at minimum `.epub` files. The user selects files through a Tauri-provided dialog, not a browser `<input type="file">`.

### FR-3: File Output

Converted ebook files are saved to the local filesystem via a native save dialog. The user chooses the output location and filename.

### FR-4: Cross-Platform Builds

The build pipeline produces distributable packages for:
- **macOS**: `.dmg` installer
- **Windows**: `.msi` or `.exe` installer
- **Linux**: `.AppImage` and `.deb`

### FR-5: Auto-Update (Deferred)

Out of scope for the initial release. The spec acknowledges this as a future enhancement.

### FR-6: Development Workflow

Developers can run the application in development mode with hot-module replacement (HMR) from Vite, rendered inside the Tauri webview — preserving the existing `npm run dev` ergonomics.

## Non-Functional Requirements

### NFR-1: Bundle Size

The application binary (excluding OS-provided webview) should remain under 15 MB.

### NFR-2: Startup Time

Cold start to interactive UI in under 2 seconds on a modern machine.

### NFR-3: Security

Follow Tauri's security best practices:
- Disable the `dangerousRemoteDomainIpcAccess` option.
- Scope filesystem access to user-selected paths only (no blanket read/write).
- CSP configured to prevent loading remote scripts in production builds.

### NFR-4: Existing Frontend Untouched

The React/Vite frontend source (`src/`, `index.html`, `vite.config.js`) requires zero or minimal changes. Tauri integrates alongside, not inside, the existing build.

## Out of Scope

- Backend server or database
- Cloud sync or user accounts
- Auto-update mechanism (FR-5 deferred)
- Mobile builds (Tauri mobile is experimental)
- Tray icon / menu bar mode
