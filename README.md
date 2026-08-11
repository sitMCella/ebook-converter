# Ebook Converter

A desktop application for converting ebook files, built with React and wrapped in Tauri for native cross-platform distribution.

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for the full architecture documentation, including frontend and backend details, project structure, security model, CI/CD, code signing, and custom app icons.

## Prerequisites

### All Platforms

- **Node.js** >= 20 — [nodejs.org](https://nodejs.org/)
- **Rust** >= 1.77 — install via [rustup](https://rustup.rs/):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

### macOS

Xcode Command Line Tools:

```bash
xcode-select --install
```

### Windows

- [Microsoft Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the "Desktop development with C++" workload
- [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (pre-installed on Windows 10 version 1803+ and Windows 11)

### Linux (Debian/Ubuntu)

```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

For other distributions, see the [Tauri prerequisites guide](https://v2.tauri.app/start/prerequisites/).

## Getting Started

Clone the repository and install dependencies:

```bash
git clone git@github.com:sitMCella/ebook-converter.git
cd ebook-converter
npm install
```

## Development

### Desktop App (Tauri + Vite)

Start the app in a native window with hot-module replacement:

```bash
npm run tauri dev
```

This starts the Vite dev server and opens a Tauri window pointing at it. Edits to React components are reflected instantly via HMR.

The first run compiles the Rust backend, which takes 1-2 minutes. Subsequent runs start in seconds.

### Clearing the Cache

If frontend changes aren't reflected, first kill any running instances and clean build artifacts:

```bash
pkill -f "ebook-converter" ; pkill -f "Ebook Converter"
rm -rf dist
rm -rf src-tauri/target/debug/build
```

Then clear the Tauri WebView cache:

**macOS:**
```bash
rm -rf ~/Library/WebKit/ebook-converter
rm -rf ~/Library/Caches/ebook-converter
```

**Linux:**
```bash
rm -rf ~/.local/share/ebook-converter
rm -rf ~/.cache/ebook-converter
```

**Windows (PowerShell):**
```powershell
Remove-Item -Recurse -Force "$env:APPDATA\ebook-converter"
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\ebook-converter"
```

### Browser Only (Vite)

To work on the frontend without building the Rust backend:

```bash
npm run dev
```

Opens the app at `http://localhost:5173`. File operations fall back to browser APIs (standard file input and download links instead of native dialogs).

### Linting

```bash
npm run lint
```

## Testing

### Unit Tests

Unit tests use [Vitest](https://vitest.dev/) with jsdom and [Testing Library](https://testing-library.com/). They cover individual components, hooks, contexts, and utility functions.

```bash
npm run test
```

To run in watch mode during development:

```bash
npm run test:watch
```

### End-to-End Tests

E2E tests use [Playwright](https://playwright.dev/) against the Vite dev server. They exercise full user flows in a real browser (Chromium).

First-time setup — install the Playwright browser binaries:

```bash
npx playwright install chromium
```

Run the tests (the Vite dev server starts automatically):

```bash
npm run test:e2e
```

## Building for Release

### Local Build

Build a distributable package for the current platform:

```bash
npm run tauri build
```

This runs `vite build` for the frontend, then compiles an optimized Rust binary and packages it into a platform-specific installer.

Output locations:

| Platform | Format | Path |
|---|---|---|
| macOS | `.dmg`, `.app` | `src-tauri/target/release/bundle/dmg/` |
| Windows | `.msi`, `.exe` | `src-tauri/target/release/bundle/msi/` or `nsis/` |
| Linux | `.deb`, `.AppImage` | `src-tauri/target/release/bundle/deb/` or `appimage/` |

### Build a Specific Target

```bash
npm run tauri build -- --target universal-apple-darwin   # macOS universal binary
npm run tauri build -- --bundles deb                     # Linux .deb only
npm run tauri build -- --bundles msi                     # Windows .msi only
```

### Creating a Release

Releases are built automatically by GitHub Actions for Windows, macOS (ARM + Intel), and Linux when a version tag is pushed.

1. Update `CHANGELOG.md` — move entries from `[Unreleased]` into a new version section:

   ```markdown
   ## [0.2.0] - 2026-09-01

   ### Added
   - New feature description.
   ```

2. Update the version in `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml` to match the new version.

3. Commit and tag:

   ```bash
   git add CHANGELOG.md src-tauri/tauri.conf.json src-tauri/Cargo.toml
   git commit -m "Release v0.2.0"
   git tag v0.2.0
   git push origin main --tags
   ```

4. The workflow builds platform-specific packages and creates a **draft GitHub Release** with the binaries attached. Review the draft on the [Releases page](../../releases) and publish it when ready.

## Available Scripts

| Script | Description |
|---|---|
| `npm run dev` | Start Vite dev server (browser only) |
| `npm run build` | Build frontend for production |
| `npm run preview` | Preview the production frontend build |
| `npm run test` | Run unit tests (Vitest) |
| `npm run test:watch` | Run unit tests in watch mode |
| `npm run test:e2e` | Run end-to-end tests (Playwright) |
| `npm run lint` | Run Oxlint |
| `npm run tauri dev` | Start Tauri + Vite in development mode |
| `npm run tauri build` | Build the desktop app for the current platform |

## License

MIT — see [LICENSE](LICENSE).