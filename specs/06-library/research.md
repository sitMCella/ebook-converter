# 06 — Library: Research

## Existing Infrastructure

### What's Already Available

| Capability | Source | Notes |
|---|---|---|
| File list with metadata | `ImportContext` | All imported PDFs with status, size, and metadata |
| PDF metadata extraction | `get_pdf_metadata` Rust command | Title, author, page count, PDF version, dates, producer, file size |
| File validation status | `validate_pdf` Rust command | Valid, encrypted, or error |
| Managed storage | `storage.rs` | Each book stored at `<app_data>/books/<uuid>/` |
| Settings system | `src/lib/settings.js` | `DEFAULT_SETTINGS`, `loadSettings()`, `getEffectiveSettings()`, `mergeSettings()` |
| Conversion pipeline | `useConversion` hook | Queue management, progress tracking, result handling |
| UI components | `Button`, `StatusBadge`, `ConfirmDialog`, `Checkbox` | Reusable across screens |
| Format utilities | `src/lib/format.js` | `formatFileSize()` |

### What's Missing

| Capability | Impact | Mitigation |
|---|---|---|
| PDF page rendering | Cannot show page preview | Show placeholder with file icon and page count |
| Persistent library state | File list is session-only | State lives in ImportContext; persistence is a future feature |
| Settings screen | Global settings cannot be changed via UI | Use `DEFAULT_SETTINGS` as the baseline; settings.js is already wired |

## Two-Panel Layout Pattern

The Library uses a master-detail layout common in document management applications. This pattern is already established in the UI/UX design spec (Screen 2) and is mirrored by the Converted screen (Screen 4).

### Layout Implementation

CSS Flexbox with:
- Left panel: `w-[260px] min-w-[260px]` (fixed width, scrollable)
- Right panel: `flex-1 overflow-y-auto` (fills remaining width, scrollable)
- Separator: `border-r border-[var(--border)]` between panels

Both panels scroll independently, which is critical when the document list is long and the detail panel has extensive metadata.

## Per-Document Override UI Pattern

The spec defines three visual states for each overridable setting:

1. **Inherited**: Shows global value + "(default)" in muted text. Control is interactive.
2. **Overridden**: Shows custom value in primary text. Reset (×) button visible.
3. **Collapsed indicator**: Header shows "· N custom" in accent colour when overrides exist.

This pattern uses the `mergeSettings()` function from `settings.js` to compare document overrides against global defaults.

## References

- UI/UX Design spec, Screen 2: Library (Source Document Preview + Details)
- UI/UX Design spec, Settings Architecture — Per-Document Overrides
- Spec 03: Import PDF (file import flow)
- Spec 05: Storage Location (managed book storage)
