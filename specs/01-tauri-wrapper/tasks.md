# 01 — Tauri Wrapper: Tasks

## Phase 1: Scaffolding

### T1: Install Prerequisites
- [ ] Install Rust toolchain via `rustup` (if not already installed)
- [ ] Verify `cargo` and `rustc` are available
- [ ] On Linux: install system dependencies (`libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, etc.)
- [ ] On Windows: verify WebView2 runtime is available

### T2: Initialize Tauri in the Project
- [ ] Install `@tauri-apps/cli` as a dev dependency
- [ ] Run `npx tauri init` to scaffold `src-tauri/`
- [ ] Configure `tauri.conf.json`:
  - Set `identifier` to `com.ebook-converter.app`
  - Set `productName` to `Ebook Converter`
  - Set `devUrl` to `http://localhost:5173`
  - Set `frontendDist` to `../dist`
  - Set window title, default size (1024x768), and minimum size (640x480)

### T3: Add npm Scripts
- [ ] Add `"tauri": "tauri"` to `package.json` scripts
- [ ] Verify `npm run tauri dev` starts both Vite and the Tauri window
- [ ] Verify HMR works inside the Tauri webview

## Phase 2: Tauri Plugins & Capabilities

### T4: Add File Dialog Plugin
- [ ] Install `@tauri-apps/plugin-dialog` (npm) and `tauri-plugin-dialog` (Cargo)
- [ ] Register the plugin in `src-tauri/src/lib.rs`
- [ ] Add dialog capability to `src-tauri/capabilities/default.json`

### T5: Add Filesystem Plugin
- [ ] Install `@tauri-apps/plugin-fs` (npm) and `tauri-plugin-fs` (Cargo)
- [ ] Register the plugin in `src-tauri/src/lib.rs`
- [ ] Configure scoped filesystem access in capabilities (user-selected paths only)

### T6: Security Configuration
- [ ] Review and tighten CSP in `tauri.conf.json` for production
- [ ] Ensure no `dangerousRemoteDomainIpcAccess` is enabled
- [ ] Verify filesystem scope is limited to dialog-selected paths

## Phase 3: Frontend Integration

### T7: Create Tauri Bridge Module
- [ ] Create `src/lib/tauri.js` — a module that wraps Tauri API calls
- [ ] Implement `openFileDialog()` — opens native file picker, returns file path + contents
- [ ] Implement `saveFileDialog(data, defaultName)` — opens native save dialog, writes file
- [ ] Add environment detection: use Tauri APIs when running in Tauri, fall back to browser APIs otherwise

### T8: Wire File Dialogs into the UI
- [ ] Replace or augment the existing file input with a button that calls `openFileDialog()`
- [ ] Wire the export/save action to call `saveFileDialog()`
- [ ] Handle errors (user cancels dialog, write permission denied)

## Phase 4: Application Identity

### T9: App Icons
- [ ] Create or source a 1024x1024 PNG app icon
- [ ] Run `npx tauri icon <path-to-icon>` to generate all required icon sizes
- [ ] Verify icons appear in the dock/taskbar and window title bar

### T10: Window Configuration
- [ ] Set default window size and minimum size in `tauri.conf.json`
- [ ] Configure window title
- [ ] Test window resize behavior and responsive layout

## Phase 5: Build & Distribution

### T11: Production Build
- [ ] Run `npm run tauri build` and verify it produces a working binary
- [ ] Test the built binary on macOS (`.dmg`)
- [ ] Test on Windows (`.msi`/`.exe`) via CI or cross-compilation
- [ ] Test on Linux (`.AppImage`, `.deb`) via CI or VM

### T12: CI Pipeline (Optional)
- [ ] Add GitHub Actions workflow for building on all three platforms
- [ ] Use `tauri-apps/tauri-action` for automated builds
- [ ] Upload build artifacts to GitHub Releases

## Acceptance Criteria

- [ ] `npm run tauri dev` opens a native window rendering the React app with HMR
- [ ] The native file open dialog can select a file and its contents reach the React app
- [ ] The native file save dialog writes a file to the chosen location
- [ ] `npm run tauri build` produces a working installer for the current platform
- [ ] The production binary starts in under 2 seconds
- [ ] The binary size is under 15 MB (excluding OS webview)
