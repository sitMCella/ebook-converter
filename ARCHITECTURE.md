# Architecture

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

## Frontend

| Technology | Version | Purpose |
|---|---|---|
| React | 19 | UI components |
| Vite | 8 | Build tooling and dev server with HMR |
| Tailwind CSS | 4 | Utility-first styling |
| Oxlint | 1.x | Linting |

The frontend is a standard React SPA. It runs identically in a browser (for development without Tauri) and inside the Tauri webview (for the desktop app). The bridge module at `src/lib/tauri.js` detects the runtime environment and provides native file dialogs when running in Tauri or falls back to browser APIs otherwise.

## Backend (Tauri)

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

## Project Structure

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

## Security Model

Tauri v2 uses capability-based permissions defined in `src-tauri/capabilities/default.json`. The app is scoped to:

- Opening and saving files through native dialogs only
- Reading and writing only the files the user explicitly selects
- No blanket filesystem access, no remote code loading

The Content Security Policy in production restricts sources to `'self'` with inline styles allowed for Tailwind.

## CI/CD (GitHub Actions)

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

## Code Signing

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
