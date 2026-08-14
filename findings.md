# Conversion Quality Findings — Architecture Patterns with Python (EPUB vs PDF)

Book id: `49827d37-d502-4360-b07c-b1b23f0d4579`

**Method note:** No PDF text-extraction tool was available (no `pdftotext`, `pypdf`,
`pymupdf`, `poppler`); the PDF uses FlateDecode object streams not parseable via
`strings`. Findings come from internal-consistency analysis of the extracted EPUB
text (`OEBPS/content.xhtml`).

## 1. Running header/footer leftovers, everywhere (major, ~130+ instances)
Page footers of the form `"<page#> | <Chapter Title>"` (and front-matter variants
like `"iv | Table of Contents"`, `"x | Preface"`) are not stripped — they appear as
their own `<p>` elements mid-body, interrupting paragraphs/images throughout the
entire book, e.g.:
- `<p>6 | Chapter 1: Domain Modeling</p>` right after body text, before the next image.
- `<p>264 | Appendix E: Validation</p>` near the very end.

Notable: the codebase's own recent commit history (`b65b83b "Generalize footer
detection to handle 'page | title' format"`, `3a70881 "Strip footer iteratively to
also remove pipe when split across lines"`) claims to fix exactly this pattern, and
the EPUB's mtime is after those commits — yet the defect is still present ~130 times.

## 2. Hyphenated line-break words not rejoined (major, 403 instances)
Words broken across the PDF's line-wrap hyphenation are left split with the soft
hyphen character and a space instead of being merged, e.g.:
- `"Impera‐ tive Shell"` (should be "Imperative Shell")
- `"com‐ plexity"`, `"busi‐ nesses"`, `"SQL‐ Alchemy"`, `"appli‐ cable"`

## 3. Code blocks completely flattened into prose (critical)
Zero `<pre>` or `<code>` elements in the whole EPUB. Multi-line Python code samples
are merged into single run-on `<p>` paragraphs, losing all indentation/line breaks
and often concatenating multiple distinct code blocks together, e.g.:
> "class Batch: def __init__( self, ref: str, sku: str, qty: int, eta: Optional[date]
> ): self.reference = ref self.sku = sku self.eta = eta self.available_quantity =
> qty def allocate(self, line: OrderLine): self.available_quantity -= line.qty"

Terminal/shell examples suffer the same fate (e.g. the `grep -c test_ ...` output
table merged into one line).

## 4. Mis-split identifier (word split without hyphen)
`"available_quan tity"` — the identifier `available_quantity` is split mid-word by
a stray space.

## 5. Wrong heading level (structure defect)
`"CQRS"` is emitted as an `<h1>` (same level as chapter titles) when it's actually
a bullet/pattern-list item at the same level as the neighboring
`<h2>Domain Events</h2>` and `<h2>Message Bus</h2>` — breaking the document outline.

## 6. Two-column Index section garbled (reading-order defect)
The back-of-book index (two-column layout in the PDF) is jumbled out of order with
dropped characters at column boundaries, e.g.:
> "using Redis pub/sub channel for ntegra‐ tion, 168" (missing leading "i" of
> "integration"), and entries/page numbers interleaved incorrectly
> ("...Celery and, 124 class given handlers at runtime...").

All six issues recur consistently from front matter through Appendix E, confirming
they are systemic, not isolated.
