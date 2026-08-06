# 07 — Converted: Research

## Existing Infrastructure

### What's Already Available

| Capability | Source | Notes |
|---|---|---|
| Converted file data | `ImportContext` + `BookMetadata` | Files with `status === 'converted'` have `outputPath` and `conversionResult` (images, fileSize); persisted to `metadata.json` via `saveBookMetadata` and restored on startup by `LOAD_LIBRARY` |
| PDF source metadata | `ImportContext` file entries | Title, author, page count — carried over from import |
| Per-document overrides | `ImportContext` file entries | `file.overrides` — shows which settings were customised |
| File size formatting | `src/lib/format.js` | `formatFileSize()` |
| Two-panel layout pattern | Library screen (spec 06) | Same 260px list + flex detail panel layout, can reuse structural patterns |
| Save dialog | `src/lib/tauri.js` | `saveFile()` already supports saving with native dialog |
| UI components | `Button`, `StatusBadge` | Reusable across screens |
| Settings utilities | `src/lib/settings.js` | `DEFAULT_SETTINGS`, `getEffectiveSettings()` for determining override count |
| Route navigation | React Router | Used by Library and Converting screens for inter-screen linking |

### What's Missing

| Capability | Impact | Mitigation |
|---|---|---|
| ~~Shell plugin for opening files/folders~~ | ~~Resolved~~ | Shell plugin installed: `@tauri-apps/plugin-shell` (npm), `tauri-plugin-shell` (Cargo), registered in `lib.rs`, `shell:allow-open` capability granted. `openFileWithSystem()` and `openFolder()` bridge functions fully operational in Tauri mode; no-op in browser mode |
| EPUB content parsing | Cannot render content preview | Show cover image when available, or placeholder UI with book icon |
| Conversion timestamp | Cannot show exact conversion date/time | Omit field initially; add `convertedAt` to `BookMetadata` and `SET_CONVERSION_RESULT` in a future enhancement |

## Two-Panel Layout Pattern (Reuse from Library)

The Converted screen uses the same master-detail layout as the Library screen. Implementation patterns to follow:

### Layout Structure
```
Header: flex justify-between items-center, 20px bottom margin
Two panels: flex row
  Left: w-[260px] min-w-[260px] border-r overflow-y-auto
  Right: flex-1 overflow-y-auto p-6
```

### List Item Pattern
- `<button>` with `role="option"` and `aria-selected`
- Padding: 10px vertical, 14px horizontal
- Selected: `bg-[var(--bg-accent)]` background
- Hover (non-selected): `hover:bg-[var(--surface-2)]`
- Bottom border separator: `border-b border-[var(--border)]`

### Metadata Display Pattern
- Labels: 11px, uppercase, muted, 0.3px letter-spacing
- Values: 13px, primary text
- 12px vertical spacing between rows
- Hide rows with null/empty/zero values

## Shell Plugin Integration

The Tauri shell plugin enables opening files and URLs with the system default application.

### API
```javascript
import { open } from '@tauri-apps/plugin-shell';

// Open file with default app
await open('/path/to/file.epub');

// Open folder in file manager
await open('/path/to/folder/');
```

### Plugin Setup
- npm: `npm install @tauri-apps/plugin-shell`
- Cargo: `tauri-plugin-shell = "2"` in `src-tauri/Cargo.toml`
- Register: `.plugin(tauri_plugin_shell::init())` in `src-tauri/src/lib.rs`
- Capability: `"shell:allow-open"` in `src-tauri/capabilities/default.json`

### Browser Fallback
In browser mode, `openFileWithSystem` is a no-op (the EPUB file is not on the local filesystem in a browser context). For browser mode, the button is hidden or shows a tooltip explaining it requires the desktop app.

## Component Naming

To avoid name collisions with the Library screen components, Converted screen components use the `Epub` prefix:
- `EpubList` (vs `DocumentList`)
- `EpubListItem` (vs `DocumentListItem`)
- `EpubDetailPanel` (vs `DetailPanel`)
- `EpubMetadata` (vs `MetadataSection`)
- `EpubPreview` (vs `PagePreview`)

## References

- UI/UX Design spec (spec 02), Screen 4: Converted (EPUB Library + Preview)
- Spec 04: Convert PDF to EPUB (conversion result data model)
- Spec 06: Library (two-panel layout pattern, component structure)
- [Tauri Shell Plugin](https://v2.tauri.app/plugin/shell/)
