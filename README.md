# Ebook Converter

A desktop application for converting ebook files, built with React and wrapped in Tauri for native cross-platform distribution.

## Architecture

The application follows a two-layer architecture: a React frontend rendered inside a native OS window provided by Tauri v2.

```
┌──────────────────────────────────────────┐
│            Native OS Window              │
│  ┌────────────────────────────────────┐  │
│  │         OS Webview                 │  │
│  │  ┌──────────────────────────────┐  │  │
│  │  │     React Frontend           │  │  │
│  │  │     (Vite + Tailwind CSS)    │  │  │
│  │  └──────────────────────────────┘  │  │
│  └────────────────────────────────────┘  │
│           Tauri Rust Backend             │
│  ┌────────────────────────────────────┐  │
│  │  Plugins: dialog, fs, log          │  │
│  └────────────────────────────────────┘  │
└──────────────────────────────────────────┘
```

### Frontend

| Technology | Version | Purpose |
|---|---|---|
| React | 19 | UI components |
| Vite | 8 | Build tooling and dev server with HMR |
| Tailwind CSS | 4 | Utility-first styling |
| Oxlint | 1.x | Linting |

The frontend is a standard React SPA. It runs identically in a browser (for development without Tauri) and inside the Tauri webview (for the desktop app). The bridge module at `src/lib/tauri.js` detects the runtime environment and provides native file dialogs when running in Tauri or falls back to browser APIs otherwise.

### Backend (Tauri)

The Rust backend is intentionally minimal. It registers three Tauri plugins and contains no custom application logic:

- **dialog** — native open/save file picker dialogs
- **fs** — scoped filesystem read/write (only user-selected paths)
- **log** — structured logging in development builds

Tauri uses the OS-provided webview engine, keeping the binary small:

| OS | Webview Engine |
|---|---|
| macOS | WebKit (WKWebView) |
| Windows | WebView2 (Chromium-based, ships with Windows 10/11) |
| Linux | WebKitGTK |

### Project Structure

```
ebook-converter/
├── src/                          # React frontend
│   ├── main.jsx                  # App entry point
│   ├── App.jsx                   # Root component
│   ├── index.css                 # Global styles (Tailwind)
│   └── lib/
│       └── tauri.js              # Tauri/browser bridge (file I/O)
├── src-tauri/                    # Tauri native backend
│   ├── Cargo.toml                # Rust dependencies
│   ├── tauri.conf.json           # Tauri app configuration
│   ├── capabilities/
│   │   └── default.json          # Permission scopes
│   ├── icons/                    # App icons (all platforms)
│   └── src/
│       ├── main.rs               # Native entry point
│       └── lib.rs                # Plugin registration
├── public/                       # Static assets
├── index.html                    # HTML shell
├── vite.config.js                # Vite configuration
└── package.json                  # Node dependencies and scripts
```

### Security Model

Tauri v2 uses capability-based permissions defined in `src-tauri/capabilities/default.json`. The app is scoped to:

- Opening and saving files through native dialogs only
- Reading and writing only the files the user explicitly selects
- No blanket filesystem access, no remote code loading

The Content Security Policy in production restricts sources to `'self'` with inline styles allowed for Tailwind.

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
git clone <repository-url>
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

### CI/CD (GitHub Actions)

For automated cross-platform builds, use the [tauri-apps/tauri-action](https://github.com/tauri-apps/tauri-action) GitHub Action. A minimal workflow:

```yaml
name: Build
on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - platform: macos-latest
          - platform: ubuntu-22.04
          - platform: windows-latest
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 22
      - uses: dtolnay/rust-toolchain@stable
      - run: npm install
      - uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### Code Signing

Production distribution requires code signing:

- **macOS** — an Apple Developer certificate and notarization. Set `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, and `APPLE_TEAM_ID` in your CI environment.
- **Windows** — an EV code signing certificate improves SmartScreen trust. See the [Tauri signing guide](https://v2.tauri.app/distribute/sign/windows/).

For local development and testing, unsigned builds work fine.

## Custom App Icons

Replace the default Tauri icons with your own:

1. Prepare a 1024x1024 PNG source image
2. Generate all required sizes:
   ```bash
   npx tauri icon path/to/icon-1024x1024.png
   ```
   This overwrites the files in `src-tauri/icons/`.

## Available Scripts

| Script | Description |
|---|---|
| `npm run dev` | Start Vite dev server (browser only) |
| `npm run build` | Build frontend for production |
| `npm run preview` | Preview the production frontend build |
| `npm run lint` | Run Oxlint |
| `npm run tauri dev` | Start Tauri + Vite in development mode |
| `npm run tauri build` | Build the desktop app for the current platform |

## License

MIT — see [LICENSE](LICENSE).