# 01 — Tauri Wrapper: Research

## Framework Comparison

### Tauri v2 vs Electron

| Criterion | Tauri v2 | Electron |
|-----------|----------|----------|
| Binary size | ~3-10 MB | ~150+ MB |
| RAM usage | ~30-50 MB | ~100-300 MB |
| Backend language | Rust | Node.js |
| Webview | OS-native (WebKit/WebView2/WebKitGTK) | Bundled Chromium |
| Security model | Capability-scoped permissions | Full Node.js access by default |
| Ecosystem maturity | Newer, growing | Mature, large ecosystem |
| Auto-update | Built-in plugin | electron-updater |
| Cross-compilation | Via CI (GitHub Actions) | electron-builder |

**Decision**: Tauri wins on bundle size, resource usage, and security — all critical for a utility app like an ebook converter. The trade-off is a less mature ecosystem and Rust on the backend, but the Rust surface area is minimal (file I/O bridging only).

### Tauri v2 vs Tauri v1

Tauri v2 (stable since October 2024) is the recommended version:
- Plugin-based architecture (dialog, fs, shell, etc. are separate plugins)
- Capability-based security model replaces the v1 allowlist
- Better multi-window support
- Mobile support (experimental, out of scope here)

## Tauri v2 Plugin Ecosystem

### Required Plugins

| Plugin | npm Package | Cargo Crate | Purpose |
|--------|-------------|-------------|---------|
| Dialog | `@tauri-apps/plugin-dialog` | `tauri-plugin-dialog` | Native open/save file dialogs |
| Filesystem | `@tauri-apps/plugin-fs` | `tauri-plugin-fs` | Read/write files on disk |

### Optional (Future)

| Plugin | Purpose |
|--------|---------|
| `tauri-plugin-updater` | Auto-update from GitHub Releases |
| `tauri-plugin-shell` | Invoke external CLI tools (e.g., Calibre) |
| `tauri-plugin-notification` | Desktop notifications for long conversions |

## Webview Compatibility

Tauri uses the OS-provided webview, not a bundled browser engine:

| OS | Webview Engine | Notes |
|----|---------------|-------|
| macOS | WebKit (WKWebView) | Ships with macOS; always up to date |
| Windows | WebView2 (Chromium-based) | Ships with Windows 10/11; Tauri bundles a bootstrapper for older systems |
| Linux | WebKitGTK | Must be installed as a system dependency; version varies by distro |

**Risk**: CSS/JS features may behave differently across webview engines. Tailwind CSS 4 and React 19 are well-supported on all three. Test specifically on Linux WebKitGTK for edge cases.

## Vite Integration

Tauri v2 is framework-agnostic. Integration with Vite requires two config values:

```json
{
  "build": {
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist"
  },
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  }
}
```

- In dev: Tauri starts the Vite dev server automatically, then opens a webview pointing at it. HMR works as normal.
- In build: Tauri runs `npm run build` first, then packages the `dist/` output into the native binary.

No changes to `vite.config.js` are required. The only consideration is that `vite.config.js` should not set a `base` path other than `/` (the current config uses the default, which is correct).

## Security Model

Tauri v2 uses a capability-based permission system defined in JSON files under `src-tauri/capabilities/`:

```json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "dialog:default",
    "fs:allow-read-file",
    "fs:allow-write-file"
  ]
}
```

Filesystem access is scoped — the app can only read/write paths the user explicitly selects via the dialog. No blanket filesystem access is granted.

## Build & Distribution

### macOS
- Produces `.dmg` and `.app` bundle
- Code signing requires an Apple Developer certificate (optional for local builds, required for distribution)
- Notarization needed for Gatekeeper approval

### Windows
- Produces `.msi` and/or `.exe` (NSIS) installer
- Code signing with an EV certificate is recommended for SmartScreen trust
- WebView2 bootstrapper is bundled automatically

### Linux
- Produces `.deb` and `.AppImage`
- `.AppImage` is portable; `.deb` is for Debian/Ubuntu
- System dependencies (WebKitGTK, etc.) must be listed in the package metadata

### CI/CD

The `tauri-apps/tauri-action` GitHub Action builds for all three platforms in a matrix:

```yaml
strategy:
  matrix:
    include:
      - platform: macos-latest
      - platform: ubuntu-22.04
      - platform: windows-latest
```

## References

- [Tauri v2 Documentation](https://v2.tauri.app/)
- [Tauri v2 Plugin Directory](https://v2.tauri.app/plugin/)
- [Tauri + Vite Guide](https://v2.tauri.app/start/frontend/vite/)
- [tauri-apps/tauri-action (GitHub Actions)](https://github.com/tauri-apps/tauri-action)
