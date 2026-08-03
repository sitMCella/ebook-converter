# 06 — Library: Data Model

## State — ImportContext Extension

### File Object (extended)

The existing file object in the `ImportContext` Map gains an `overrides` field:

```javascript
{
  // ... existing fields (path, name, size, status, metadata, bookId, etc.)
  overrides: Partial<ConversionSettings> | undefined,
}
```

### New Reducer Action

```javascript
// SET_DOCUMENT_OVERRIDES
dispatch({
  type: 'SET_DOCUMENT_OVERRIDES',
  path: string,            // file path (Map key)
  overrides: object | null, // partial settings, or null to clear all
})
```

## Per-Document Overrides Shape

A partial subset of the global settings (excluding `outputLocation`):

```javascript
{
  structure?: {
    headingLevelThreshold?: number,
  },
  images?: {
    imageQuality?: 'high' | 'medium' | 'low',
  },
  output?: {
    baseFontSize?: number,
  },
  pageHandling?: {
    splitChaptersBy?: 'heading1' | 'heading2' | 'pageBreak' | 'none',
    pageRange?: 'all' | 'custom',
    pageRangeFrom?: number | null,
    pageRangeTo?: number | null,
  },
}
```

## Effective Settings Merge

At conversion time, the effective settings for a document are:

```javascript
import { getEffectiveSettings } from '../lib/settings';

const effective = getEffectiveSettings(globalSettings, file.overrides);
```

Only keys explicitly present in `file.overrides` replace global values. Absent keys inherit the global default.

## Component Props

### DocumentList

```javascript
{
  files: ImportedFile[],     // filtered list of files
  selectedPath: string|null, // currently selected file path
  onSelect: (path: string) => void,
}
```

### DocumentListItem

```javascript
{
  file: ImportedFile,
  selected: boolean,
  onSelect: () => void,
}
```

### DetailPanel

```javascript
{
  file: ImportedFile,
}
```

### MetadataSection

```javascript
{
  file: ImportedFile,
}
```

### ConversionOptions

```javascript
{
  file: ImportedFile,
}
```

### PagePreview

```javascript
{
  file: ImportedFile,
}
```

## PDF Metadata Shape (existing, from Rust)

```javascript
{
  title: string | null,
  author: string | null,
  pageCount: number,
  pdfVersion: string,
  createdDate: string | null,
  modifiedDate: string | null,
  producer: string | null,
  fileSize: number,
}
```
