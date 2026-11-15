use super::PageHandlingOptions;
use mupdf::text_page::TextBlockType;
use mupdf::{TextCharFlags, TextPage, TextPageFlags};

/// Wraps a detected code block in the extracted page text. Private-use-area code
/// points, so they can never collide with real PDF text content.
pub(crate) const CODE_BLOCK_START: char = '\u{E000}';
pub(crate) const CODE_BLOCK_END: char = '\u{E001}';

/// Prefixes a detected heading line, immediately followed by an ASCII digit
/// ('1'-'6') for its level, e.g. "\u{E002}1Domain Modeling".
pub(crate) const HEADING_MARK: char = '\u{E002}';

/// Starts a kept note line (Keep notes on), placed at the end of its page's
/// text: `NOTE_START` id `NOTE_FIELD_SEP` label `NOTE_FIELD_SEP` note text.
/// `id` is unique across the document; `label` is the printed mark ("1").
pub(crate) const NOTE_START: char = '\u{E003}';

/// Wraps a reference mark pointing at a kept note:
/// `NOTE_REF_START` id `NOTE_FIELD_SEP` label `NOTE_REF_END`.
pub(crate) const NOTE_REF_START: char = '\u{E004}';
pub(crate) const NOTE_REF_END: char = '\u{E005}';

/// Separates the fields of `NOTE_START` lines and `NOTE_REF_START` marks.
pub(crate) const NOTE_FIELD_SEP: char = '\u{E006}';

/// Starts an image line, placed in the page text where the image sits
/// vertically: `IMAGE_MARK` resource path `NOTE_FIELD_SEP` display width
/// percent (empty when none) `NOTE_FIELD_SEP` alt text.
pub(crate) const IMAGE_MARK: char = '\u{E007}';

/// An image to place in a page's text: its vertical centre on the page and the
/// complete `IMAGE_MARK` line that stands for it.
pub struct ImageAnchor {
    pub page_y: f32,
    pub mark: String,
}

/// Builds the `IMAGE_MARK` line for an image.
pub fn image_mark(resource_path: &str, display_width_pct: Option<u32>, alt: &str) -> String {
    let pct = display_width_pct.map(|p| p.to_string()).unwrap_or_default();
    format!("{IMAGE_MARK}{resource_path}{NOTE_FIELD_SEP}{pct}{NOTE_FIELD_SEP}{alt}")
}

/// Text pages extracted from a PDF, plus the 1-based PDF page number that
/// `pages[0]` came from (before blank-page skipping), so callers can line up
/// per-page resources such as images.
pub struct ExtractedText {
    pub pages: Vec<String>,
    pub first_page: u32,
}

/// Headings that open a table of contents, compared case-insensitively.
const TOC_TITLES: &[&str] = &[
    "contents",
    "table of contents",
    "brief contents",
    "contents at a glance",
];

/// How far into the document a text-detected table of contents may start.
/// Front matter rarely runs longer; past this, a "Contents" heading is more
/// likely a chapter title than the book's TOC.
const TOC_SCAN_PAGES: usize = 30;

fn is_toc_title(title: &str) -> bool {
    let normalized = title
        .trim()
        .trim_end_matches(|c: char| c == ':' || c == '.')
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    TOC_TITLES.contains(&normalized.as_str())
}

/// First non-blank line of a page's extracted text, without the heading marker.
fn first_line(page_text: &str) -> Option<&str> {
    page_text
        .lines()
        .map(|l| {
            let l = l.trim();
            match l.strip_prefix(HEADING_MARK) {
                Some(rest) => rest.get(1..).unwrap_or("").trim(),
                None => l,
            }
        })
        .find(|l| !l.is_empty())
}

fn find_toc_in_outline(outlines: &[mupdf::Outline]) -> Option<usize> {
    for outline in outlines {
        if is_toc_title(&outline.title) {
            if let Some(dest) = &outline.dest {
                return Some(dest.loc.page_number as usize);
            }
        }
        if let Some(page) = find_toc_in_outline(&outline.down) {
            return Some(page);
        }
    }
    None
}

/// 0-based index of the page where the table of contents starts. Prefers the
/// PDF's bookmarks; falls back to a page in the front matter whose first line
/// is a TOC heading. `None` when there is no TOC.
fn find_toc_page(outlines: &[mupdf::Outline], pages: &[String]) -> Option<usize> {
    let from_outline = find_toc_in_outline(outlines).filter(|&p| p < pages.len());
    if from_outline.is_some() {
        return from_outline;
    }
    pages
        .iter()
        .take(TOC_SCAN_PAGES)
        .position(|p| first_line(p).is_some_and(is_toc_title))
}

/// Flattens the outline tree into (title, 0-based page) pairs in reading order.
fn flatten_outline<'a>(outlines: &'a [mupdf::Outline], out: &mut Vec<(&'a str, Option<usize>)>) {
    for outline in outlines {
        out.push((
            outline.title.as_str(),
            outline.dest.as_ref().map(|d| d.loc.page_number as usize),
        ));
        flatten_outline(&outline.down, out);
    }
}

/// Page of the first bookmark after the TOC bookmark that points past
/// `toc_start`, i.e. where the content following the TOC begins.
fn find_toc_end_in_outline(outlines: &[mupdf::Outline], toc_start: usize) -> Option<usize> {
    let mut flat = Vec::new();
    flatten_outline(outlines, &mut flat);
    let toc_idx = flat.iter().position(|(title, _)| is_toc_title(title))?;
    flat[toc_idx + 1..]
        .iter()
        .filter_map(|&(_, page)| page)
        .find(|&p| p > toc_start)
}

/// Whether a line ends with a page reference (arabic or roman numeral), as TOC
/// entries do: "Preface . . . xi", "1. Introduction 3".
fn ends_with_page_ref(line: &str) -> bool {
    let last = line
        .trim()
        .trim_end_matches(|c: char| c == '.' || c == '|')
        .split_whitespace()
        .last()
        .unwrap_or("");
    !last.is_empty()
        && (last.chars().all(|c| c.is_ascii_digit())
            || last.chars().all(|c| "ivxlcdm".contains(c.to_ascii_lowercase())))
}

/// Whether a page reads like a TOC continuation: most of its lines end in a
/// page reference.
fn looks_like_toc_page(page_text: &str) -> bool {
    let lines: Vec<&str> = page_text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with(HEADING_MARK))
        .collect();
    if lines.len() < 3 {
        return false;
    }
    let refs = lines.iter().filter(|l| ends_with_page_ref(l)).count();
    refs * 2 >= lines.len()
}

/// 0-based index of the first page after the table of contents that starts at
/// `toc_start`. Prefers the PDF's bookmarks; falls back to scanning forward
/// while pages still look like TOC entries.
fn find_toc_end(outlines: &[mupdf::Outline], pages: &[String], toc_start: usize) -> usize {
    if let Some(end) = find_toc_end_in_outline(outlines, toc_start).filter(|&p| p <= pages.len()) {
        return end;
    }
    let mut end = toc_start + 1;
    while end < pages.len() && looks_like_toc_page(&pages[end]) {
        end += 1;
    }
    end.min(pages.len())
}

/// `image_anchors` maps a 1-based PDF page number to the images drawn on it;
/// each one's `IMAGE_MARK` line is placed in that page's text at the image's
/// vertical position, so a figure stays next to its caption.
pub fn extract_text(
    path: &str,
    options: &PageHandlingOptions,
    image_anchors: &std::collections::HashMap<u32, Vec<ImageAnchor>>,
) -> Result<ExtractedText, String> {
    let doc = mupdf::Document::open(path)
        .map_err(|e| format!("Failed to open PDF: {}", e))?;

    let page_count = doc
        .page_count()
        .map_err(|e| format!("Failed to get page count: {}", e))?;

    // Go through the structured text page (blocks/lines/chars, with bbox and
    // font info per char) rather than the flattened `Page::text()` convenience
    // call. Holding the `TextPage`s is the hook the code-block/heading-level/
    // column-order fixes need — those require the per-line bounds and per-char
    // font that the flattened string throws away. Collected up front (not
    // streamed page-by-page) because heading-level detection needs a
    // document-wide body-text size baseline before any page can be marked.
    let mut text_pages = Vec::with_capacity(page_count as usize);
    let mut page_heights = Vec::with_capacity(page_count as usize);
    for i in 0..page_count {
        let page = doc
            .load_page(i)
            .map_err(|e| format!("Failed to load page {}: {}", i + 1, e))?;
        // DEHYPHENATE marks a soft hyphen at end-of-line so MuPDF joins that line
        // with the next when flattening to text, instead of leaving e.g.
        // "Impera\u{2010} tive Shell" split across two lines.
        let text_page = page
            .to_text_page(TextPageFlags::DEHYPHENATE)
            .map_err(|e| format!("Failed to build text page for page {}: {}", i + 1, e))?;
        let bounds = page
            .bounds()
            .map_err(|e| format!("Failed to get bounds for page {}: {}", i + 1, e))?;
        page_heights.push(bounds.y1 - bounds.y0);
        text_pages.push(text_page);
    }

    let body_size = compute_body_size(&text_pages);

    let mut pages = Vec::with_capacity(text_pages.len());
    let mut next_note_id: u32 = 1;
    for (i, text_page) in text_pages.iter().enumerate() {
        let text = text_page
            .to_text()
            .map_err(|e| format!("Failed to extract text from page {}: {}", i + 1, e))?;
        let text = match image_anchors.get(&(i as u32 + 1)) {
            Some(anchors) if !anchors.is_empty() => insert_image_marks(&text, text_page, anchors),
            _ => text,
        };
        let text = if options.remove_page_numbers {
            strip_running_margins(&text, text_page, page_heights[i])
        } else {
            text
        };
        let text = process_notes(
            &text,
            text_page,
            page_heights[i],
            body_size,
            options.keep_notes,
            &mut next_note_id,
        );
        let text = mark_code_blocks(&text, text_page);
        let text = fix_split_words(&text, text_page);
        let text = fix_wrapped_code_terms(&text, text_page);
        let text = mark_headings(&text, text_page, body_size);
        pages.push(text);
    }

    let mut first_page: u32 = 1;

    if options.page_range == "custom" {
        let from = options.page_range_from.unwrap_or(1).max(1) as usize;
        let to = options
            .page_range_to
            .map(|t| t as usize)
            .unwrap_or(pages.len())
            .min(pages.len());

        if from > to || from > pages.len() {
            return Ok(ExtractedText { pages: Vec::new(), first_page: from as u32 });
        }

        pages = pages[(from - 1)..to].to_vec();
        first_page = from as u32;
    } else {
        let outlines = doc.outlines().unwrap_or_default();
        if let Some(toc_page) = find_toc_page(&outlines, &pages) {
            // Front matter before the TOC (half-title, back-cover blurb, copyright,
            // ...) is laid out for print and converts poorly; the cover itself is
            // carried separately by the EPUB cover, so start the text at the TOC.
            // Unless requested, skip the TOC pages too: the EPUB navigation
            // document already provides one.
            let start = if options.include_toc {
                toc_page
            } else {
                find_toc_end(&outlines, &pages, toc_page)
            };
            pages.drain(..start);
            first_page = start as u32 + 1;
        }
    }

    if options.remove_page_numbers {
        strip_page_numbers(&mut pages);
    }

    if options.skip_blank_pages {
        pages.retain(|p| !p.trim().is_empty());
    }

    Ok(ExtractedText { pages, first_page })
}

/// Inserts each anchor's `IMAGE_MARK` line into `text` (the flattened text of
/// `text_page`) just before the text block closest below the image, which is
/// where the image sits in reading order (and where a caption printed below a
/// figure starts). An image below all text goes at the end of the page.
fn insert_image_marks(text: &str, text_page: &TextPage, anchors: &[ImageAnchor]) -> String {
    // (byte offset of the block's first line in `text`, block top y).
    let mut blocks: Vec<(usize, f32)> = Vec::new();
    let mut cursor = 0usize;
    for block in text_page.blocks() {
        if block.r#type() != TextBlockType::Text {
            continue;
        }
        let Some(first) = block
            .lines()
            .map(|l| l.chars().filter_map(|c| c.char()).collect::<String>())
            .find(|l| !l.trim().is_empty())
        else {
            continue;
        };
        if let Some(rel) = text[cursor..].find(&first) {
            let pos = cursor + rel;
            // Back up to the start of the line so the mark is a line of its own.
            let line_start = text[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
            blocks.push((line_start, block.bounds().y0));
            cursor = pos + first.len();
        }
    }
    image_marks_into(text, &blocks, anchors)
}

/// Positioning half of `insert_image_marks`, separated for testing: `blocks`
/// are (line-start offset in `text`, top y) in text order.
fn image_marks_into(text: &str, blocks: &[(usize, f32)], anchors: &[ImageAnchor]) -> String {
    let mut inserts: Vec<(usize, usize, &str)> = Vec::new();
    for (order, anchor) in anchors.iter().enumerate() {
        let below = blocks
            .iter()
            .filter(|(_, y0)| *y0 >= anchor.page_y)
            .min_by(|a, b| a.1.total_cmp(&b.1));
        let offset = below.map(|(off, _)| *off).unwrap_or(text.len());
        inserts.push((offset, order, anchor.mark.as_str()));
    }
    inserts.sort_by_key(|(off, order, _)| (*off, *order));

    let mut out = String::with_capacity(text.len() + inserts.len() * 64);
    let mut last = 0usize;
    for (offset, _, mark) in inserts {
        out.push_str(&text[last..offset]);
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(mark);
        out.push_str("\n\n");
        last = offset;
    }
    out.push_str(&text[last..]);
    out
}

/// Finds text blocks whose lines are entirely set in a monospaced font (the
/// PDF's way of typesetting a code/terminal listing) and wraps each one in
/// `CODE_BLOCK_START`/`CODE_BLOCK_END` sentinel lines within `text`, so
/// `structure_detector` can render it as `<pre><code>` instead of flattening
/// it into ordinary prose paragraphs.
fn mark_code_blocks(text: &str, text_page: &TextPage) -> String {
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    let mut cursor = 0usize;

    for block in text_page.blocks() {
        if block.r#type() != TextBlockType::Text {
            continue;
        }

        let mut lines: Vec<String> = Vec::new();
        let mut mono_lines = 0usize;
        let mut text_lines = 0usize;
        // Every non-whitespace glyph of the block is set in a monospaced font.
        let mut all_glyphs_mono = true;

        for line in block.lines() {
            let mut line_text = String::new();
            let mut is_mono: Option<bool> = None;
            for ch in line.chars() {
                let Some(c) = ch.char() else { continue };
                line_text.push(c);
                let glyph_mono = ch.font().map(|f| f.is_monospaced());
                if is_mono.is_none() {
                    is_mono = glyph_mono;
                }
                if !c.is_whitespace() && glyph_mono != Some(true) {
                    all_glyphs_mono = false;
                }
            }
            if !line_text.trim().is_empty() {
                text_lines += 1;
                if is_mono == Some(true) {
                    mono_lines += 1;
                }
            }
            lines.push(line_text);
        }

        // A multi-line block counts as code when every line starts in a
        // monospaced font. A one-line block (e.g. a lone `import requests`
        // separated from the rest of its listing by a blank line) must be
        // monospaced throughout: an inline `code` word inside prose always
        // shares its block with proportional text, so it never qualifies.
        let is_code = match text_lines {
            0 => false,
            1 => all_glyphs_mono,
            _ => mono_lines == text_lines,
        };
        if !is_code {
            continue;
        }

        let key = lines.join("\n");
        if key.trim().is_empty() {
            continue;
        }

        if let Some(rel_pos) = text[cursor..].find(&key) {
            let start = cursor + rel_pos;
            let end = start + key.len();
            ranges.push((start, end));
            cursor = end;
        }
    }

    if ranges.is_empty() {
        return text.to_string();
    }

    let mut out = String::with_capacity(text.len() + ranges.len() * 4);
    let mut last = 0usize;
    for (start, end) in ranges {
        out.push_str(&text[last..start]);
        out.push(CODE_BLOCK_START);
        out.push('\n');
        out.push_str(&text[start..end]);
        out.push('\n');
        out.push(CODE_BLOCK_END);
        last = end;
    }
    out.push_str(&text[last..]);
    out
}

/// Removes a MuPDF-inserted "synthetic" space (`TextCharFlags::SYNTHETIC`) when
/// it sits between two monospaced word characters, e.g. "available_quan tity".
/// MuPDF inserts these when it judges the gap between two glyphs large enough
/// to be a real word break; monospaced/code fonts can trip that heuristic on
/// ordinary kerning (often around an underscore), splitting one identifier
/// into two. Real word breaks in prose almost always carry an actual space
/// glyph rather than a synthetic one, so this only touches the narrow case
/// MuPDF itself flagged as a guess, and only inside monospaced runs.
fn fix_split_words(text: &str, text_page: &TextPage) -> String {
    let mut cursor = 0usize;
    let mut last_copied = 0usize;
    let mut out = String::with_capacity(text.len());

    for block in text_page.blocks() {
        if block.r#type() != TextBlockType::Text {
            continue;
        }

        for line in block.lines() {
            let mut line_text = String::new();
            // (char, is a synthetic space MuPDF guessed at, char's font is monospaced)
            let mut chars: Vec<(char, bool, bool)> = Vec::new();

            for ch in line.chars() {
                let Some(c) = ch.char() else { continue };
                let is_synthetic_space = c == ' ' && ch.flags().contains(TextCharFlags::SYNTHETIC);
                let is_mono = ch.font().map(|f| f.is_monospaced()).unwrap_or(false);
                line_text.push(c);
                chars.push((c, is_synthetic_space, is_mono));
            }

            let mut needs_fix = false;
            let mut fixed = String::with_capacity(line_text.len());
            for (idx, &(c, is_synthetic_space, _)) in chars.iter().enumerate() {
                if is_synthetic_space {
                    let prev_is_mono_word = idx > 0 && chars[idx - 1].2 && is_word_char(chars[idx - 1].0);
                    let next_is_mono_word =
                        idx + 1 < chars.len() && chars[idx + 1].2 && is_word_char(chars[idx + 1].0);
                    if prev_is_mono_word && next_is_mono_word {
                        needs_fix = true;
                        continue;
                    }
                }
                fixed.push(c);
            }

            if !needs_fix {
                continue;
            }

            if let Some(rel_pos) = text[cursor..].find(&line_text) {
                let start = cursor + rel_pos;
                let end = start + line_text.len();
                out.push_str(&text[last_copied..start]);
                out.push_str(&fixed);
                last_copied = end;
                cursor = end;
            }
        }
    }

    out.push_str(&text[last_copied..]);
    out
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Joins a line-wrap that split a single inline monospaced term (e.g. a code
/// identifier like `available_quantity` mentioned inline in prose) across two
/// lines with neither a hyphen nor a space at the break — e.g.
/// "...an integer available_quan" / "tity, and we decrement...". DEHYPHENATE
/// (see `extract_text`) only rejoins breaks that end in a hyphen character;
/// this covers breaks that have neither, which normal prose line-wraps never
/// do (a real word boundary always leaves a hyphen or is itself the wrap
/// point), so it's safe to gate purely on "both sides of the break are inside
/// a monospaced word". Whole code blocks (all-monospace blocks, handled by
/// `mark_code_blocks`) are skipped: there, adjacent lines are separate lines
/// of code and must stay on separate lines.
fn fix_wrapped_code_terms(text: &str, text_page: &TextPage) -> String {
    let mut cursor = 0usize;
    let mut last_copied = 0usize;
    let mut out = String::with_capacity(text.len());

    for block in text_page.blocks() {
        if block.r#type() != TextBlockType::Text {
            continue;
        }

        let mut lines: Vec<String> = Vec::new();
        // (font of first char is monospace?, font of last char is monospace?, every char monospace?)
        let mut line_info: Vec<(Option<bool>, Option<bool>, bool)> = Vec::new();
        let mut mono_lines = 0usize;
        let mut text_lines = 0usize;

        for line in block.lines() {
            let mut line_text = String::new();
            let mut char_is_mono: Vec<bool> = Vec::new();
            for ch in line.chars() {
                if let Some(c) = ch.char() {
                    let is_mono = ch.font().map(|f| f.is_monospaced()).unwrap_or(false);
                    line_text.push(c);
                    char_is_mono.push(is_mono);
                }
            }
            let first_mono = char_is_mono.first().copied();
            let last_mono = char_is_mono.last().copied();
            let all_mono = !char_is_mono.is_empty() && char_is_mono.iter().all(|&m| m);
            if !line_text.trim().is_empty() {
                text_lines += 1;
                if all_mono {
                    mono_lines += 1;
                }
            }
            lines.push(line_text);
            line_info.push((first_mono, last_mono, all_mono));
        }

        // A whole-block code listing is mark_code_blocks's territory; its
        // line breaks are real and must not be merged away.
        if text_lines >= 2 && mono_lines == text_lines {
            continue;
        }

        for i in 0..lines.len().saturating_sub(1) {
            let (cur, next) = (&lines[i], &lines[i + 1]);
            if cur.trim().is_empty() || next.trim().is_empty() {
                continue;
            }

            let cur_ends_mono_word =
                line_info[i].1 == Some(true) && cur.chars().last().is_some_and(is_word_char);
            let next_starts_mono_word =
                line_info[i + 1].0 == Some(true) && next.chars().next().is_some_and(is_word_char);

            if !(cur_ends_mono_word && next_starts_mono_word) {
                continue;
            }

            let key = format!("{}\n{}", cur, next);
            if let Some(rel_pos) = text[cursor..].find(&key) {
                let start = cursor + rel_pos;
                let end = start + key.len();
                out.push_str(&text[last_copied..start]);
                out.push_str(cur);
                out.push_str(next);
                last_copied = end;
                cursor = end;
            }
        }
    }

    out.push_str(&text[last_copied..]);
    out
}

/// Finds the document's body-text font size: the most common non-whitespace
/// glyph size across the whole book, bucketed to the nearest 0.5pt. Headings
/// are then anything meaningfully larger than this baseline.
fn compute_body_size(text_pages: &[TextPage]) -> f32 {
    let mut counts: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();

    for text_page in text_pages {
        for block in text_page.blocks() {
            if block.r#type() != TextBlockType::Text {
                continue;
            }
            for line in block.lines() {
                for ch in line.chars() {
                    let Some(c) = ch.char() else { continue };
                    if c.is_whitespace() {
                        continue;
                    }
                    let bucket = (ch.size() * 2.0).round() as u32;
                    *counts.entry(bucket).or_insert(0) += 1;
                }
            }
        }
    }

    match counts.into_iter().max_by_key(|(_, count)| *count) {
        Some((bucket, _)) if bucket > 0 => bucket as f32 / 2.0,
        _ => 12.0,
    }
}

/// Prefixes lines set in a meaningfully larger font than the document's body
/// text with `HEADING_MARK` + a level digit, so `structure_detector` can
/// trust real typography instead of guessing from capitalization — which
/// misclassifies a short ALL-CAPS acronym like "CQRS" (body-sized, just a
/// list label) as a chapter heading. The three size tiers below were
/// calibrated against this book's actual heading sizes (chapter titles ~28pt,
/// section headings ~17-19pt, subsections ~14-15pt, against a ~10.5pt body),
/// with margin on both sides of each cutoff.
fn mark_headings(text: &str, text_page: &TextPage, body_size: f32) -> String {
    if body_size <= 0.0 {
        return text.to_string();
    }

    let mut ranges: Vec<(usize, usize, u8)> = Vec::new();
    let mut cursor = 0usize;

    for block in text_page.blocks() {
        if block.r#type() != TextBlockType::Text {
            continue;
        }
        for line in block.lines() {
            let mut line_text = String::new();
            let mut max_size: f32 = 0.0;
            for ch in line.chars() {
                let Some(c) = ch.char() else { continue };
                line_text.push(c);
                if !c.is_whitespace() {
                    max_size = max_size.max(ch.size());
                }
            }

            let trimmed = line_text.trim();
            if trimmed.is_empty() || trimmed.len() > 120 {
                continue;
            }

            let ratio = max_size / body_size;
            let level: u8 = if ratio >= 2.2 {
                1
            } else if ratio >= 1.55 {
                2
            } else if ratio >= 1.25 {
                3
            } else {
                continue;
            };

            if let Some(rel_pos) = text[cursor..].find(&line_text) {
                let start = cursor + rel_pos;
                let end = start + line_text.len();
                ranges.push((start, end, level));
                cursor = end;
            }
        }
    }

    if ranges.is_empty() {
        return text.to_string();
    }

    let mut out = String::with_capacity(text.len() + ranges.len() * 2);
    let mut last = 0usize;
    for (start, end, level) in ranges {
        out.push_str(&text[last..start]);
        out.push(HEADING_MARK);
        out.push_str(&level.to_string());
        out.push_str(&text[start..end]);
        last = end;
    }
    out.push_str(&text[last..]);
    out
}

/// Fraction of the page height, at the top and at the bottom, where running
/// headers and footers are looked for.
const MARGIN_BAND: f32 = 0.1;

/// Vertical tolerance (in points) for two lines to count as one visual row.
const ROW_TOLERANCE: f32 = 2.0;

/// Removes running headers/footers ("xii | Preface", "6 | Chapter 1: Domain
/// Modeling", "- 42 -") using line positions. MuPDF often splits such a row
/// into several lines ("6", "|", "Chapter 1: Domain Modeling"), so a purely
/// line-by-line text filter would drop the number but leave the chapter title
/// in the body. Here lines inside the top/bottom margin band are grouped into
/// visual rows; a row whose combined text reads as a page-number header or
/// footer is removed as a whole.
fn strip_running_margins(text: &str, text_page: &TextPage, page_height: f32) -> String {
    if page_height <= 0.0 {
        return text.to_string();
    }
    let top_limit = page_height * MARGIN_BAND;
    let bottom_limit = page_height * (1.0 - MARGIN_BAND);

    // (y0, x0, text) of every non-blank line lying entirely in a margin band.
    let mut margin_lines: Vec<(f32, f32, String)> = Vec::new();
    for block in text_page.blocks() {
        if block.r#type() != TextBlockType::Text {
            continue;
        }
        for line in block.lines() {
            let b = line.bounds();
            if b.y1 > top_limit && b.y0 < bottom_limit {
                continue;
            }
            let s: String = line.chars().filter_map(|c| c.char()).collect();
            if !s.trim().is_empty() {
                margin_lines.push((b.y0, b.x0, s.trim().to_string()));
            }
        }
    }

    let removable = margin_rows_to_remove(margin_lines);
    if removable.is_empty() {
        return text.to_string();
    }
    remove_lines(text, &removable)
}

/// Groups margin lines into rows by vertical position and returns the line
/// texts of every row whose combined text (left to right) is a page number,
/// header or footer.
fn margin_rows_to_remove(mut lines: Vec<(f32, f32, String)>) -> Vec<String> {
    lines.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));

    let mut rows: Vec<Vec<(f32, f32, String)>> = Vec::new();
    for line in lines {
        match rows.last_mut() {
            Some(row) if (line.0 - row[0].0).abs() <= ROW_TOLERANCE => row.push(line),
            _ => rows.push(vec![line]),
        }
    }

    let mut removable = Vec::new();
    for mut row in rows {
        row.sort_by(|a, b| a.1.total_cmp(&b.1));
        let joined = row
            .iter()
            .map(|(_, _, s)| s.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        if is_page_number(&joined) {
            removable.extend(row.into_iter().map(|(_, _, s)| s));
        }
    }
    removable
}

/// Removes one occurrence of each of `targets` (compared trimmed) from the
/// lines of `text`, scanning from the end so a footer is matched before any
/// identical line in the body.
fn remove_lines(text: &str, targets: &[String]) -> String {
    let mut lines: Vec<Option<&str>> = text.lines().map(Some).collect();
    for target in targets {
        if let Some(slot) = lines
            .iter_mut()
            .rev()
            .find(|l| l.is_some_and(|l| l.trim() == target))
        {
            *slot = None;
        }
    }
    lines.into_iter().flatten().collect::<Vec<_>>().join("\n")
}

/// Notes are set in a font at most this fraction of the body text size
/// (e.g. 8pt notes against a 10.5pt body; sidebars at ~9.6pt stay body text).
const NOTE_MAX_SIZE_RATIO: f32 = 0.85;

/// A note block starts below this fraction of the page height.
const NOTE_REGION_START: f32 = 0.5;

/// A reference mark ("algorithms.¹") is set at most this fraction of the size
/// of the text on its line.
const NOTE_MARK_MAX_SIZE_RATIO: f32 = 0.8;

/// A reference mark's baseline sits at least this fraction of its own size
/// above the baseline of the text before it.
const NOTE_MARK_MIN_RISE_RATIO: f32 = 0.15;

/// A footnote found on a page: its label ("1", "*", ...) and its full text.
#[derive(Debug, PartialEq)]
struct PageNote {
    label: String,
    text: String,
}

/// Handles the page's notes (footnotes at the bottom of the page) and the
/// reference marks pointing at them.
///
/// MuPDF often orders a footnote block ahead of the body text, so the note
/// ends up in the middle of the EPUB prose. Notes are found by layout: text
/// blocks in the lower half of the page, set noticeably smaller than the
/// body text, whose first line starts with a note label ("1 If you've...",
/// "¹ ...", "* ..."). A small block right below a note continues it.
///
/// With `keep` off, notes and their reference marks are removed. With `keep`
/// on, each note is taken out of the prose and re-added at the end of the
/// page as a `NOTE_START` line with a document-unique id (labels restart in
/// every chapter), and each reference mark becomes a `NOTE_REF_START` mark
/// carrying the same id, so the EPUB can link the two.
fn process_notes(
    text: &str,
    text_page: &TextPage,
    page_height: f32,
    body_size: f32,
    keep: bool,
    next_id: &mut u32,
) -> String {
    if page_height <= 0.0 || body_size <= 0.0 {
        return text.to_string();
    }

    let notes = find_page_notes(text_page, page_height, body_size);
    if notes.is_empty() {
        return text.to_string();
    }

    let mut out = text.to_string();
    if !keep {
        for note in &notes {
            if let Some((rest, _)) = take_text_run(&out, &note.text) {
                out = rest;
            }
        }
        let marks: Vec<(&str, String)> =
            notes.iter().map(|n| (n.label.as_str(), String::new())).collect();
        return replace_note_marks(&out, text_page, &marks);
    }

    let mut kept: Vec<(u32, &str, String)> = Vec::new();
    for note in &notes {
        // Notes MuPDF did not emit as a matching run of lines stay in place.
        let Some((rest, removed)) = take_text_run(&out, &note.text) else { continue };
        out = rest;
        let body = strip_note_label(&removed.join(" "), &note.label);
        kept.push((*next_id, note.label.as_str(), body));
        *next_id += 1;
    }
    if kept.is_empty() {
        return out;
    }

    // A label repeated on one page binds to the first note that carries it.
    let mut marks: Vec<(&str, String)> = Vec::new();
    for (id, label, _) in &kept {
        if !marks.iter().any(|(l, _)| l == label) {
            marks.push((label, note_ref(*id, label)));
        }
    }
    let mut out = replace_note_marks(&out, text_page, &marks);
    for (id, label, body) in &kept {
        out.push_str(&format!("\n\n{NOTE_START}{id}{NOTE_FIELD_SEP}{label}{NOTE_FIELD_SEP}{body}"));
    }
    out
}

fn note_ref(id: u32, label: &str) -> String {
    format!("{NOTE_REF_START}{id}{NOTE_FIELD_SEP}{label}{NOTE_REF_END}")
}

/// Drops the leading label ("1 ", "12. ", "¹", "† ") from a note's text.
fn strip_note_label(text: &str, label: &str) -> String {
    let text = text.trim();
    let normalized: String = text
        .chars()
        .map(|c| superscript_digit(c).unwrap_or(c))
        .collect();
    let rest = if normalized.starts_with(label) {
        // Labels are ASCII digits or single-codepoint symbols; skip as many
        // chars of the original text as the label has.
        let skip = label.chars().count();
        text.char_indices().nth(skip).map_or("", |(i, _)| &text[i..])
    } else {
        text
    };
    let rest = rest.strip_prefix(['.', ')']).unwrap_or(rest);
    rest.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn find_page_notes(text_page: &TextPage, page_height: f32, body_size: f32) -> Vec<PageNote> {
    // (y0, y1, max glyph size, lines) of every small-font block low on the page.
    let mut candidates: Vec<(f32, f32, f32, Vec<String>)> = Vec::new();
    for block in text_page.blocks() {
        if block.r#type() != TextBlockType::Text {
            continue;
        }
        let bounds = block.bounds();
        if bounds.y0 < page_height * NOTE_REGION_START {
            continue;
        }
        let mut max_size: f32 = 0.0;
        let mut lines = Vec::new();
        for line in block.lines() {
            let mut s = String::new();
            for ch in line.chars() {
                let Some(c) = ch.char() else { continue };
                s.push(c);
                if !c.is_whitespace() {
                    max_size = max_size.max(ch.size());
                }
            }
            lines.push(s);
        }
        if max_size > 0.0 && max_size <= body_size * NOTE_MAX_SIZE_RATIO {
            candidates.push((bounds.y0, bounds.y1, max_size, lines));
        }
    }
    candidates.sort_by(|a, b| a.0.total_cmp(&b.0));
    group_page_notes(candidates)
}

/// Turns small-font blocks (sorted top to bottom) into notes: a block whose
/// first line starts with a note label opens a note; a block directly below
/// a note (gap under 1.5 lines) continues it; anything else is ignored.
fn group_page_notes(candidates: Vec<(f32, f32, f32, Vec<String>)>) -> Vec<PageNote> {
    let mut notes: Vec<PageNote> = Vec::new();
    let mut last_bottom: Option<f32> = None;
    for (y0, y1, size, lines) in candidates {
        let first = lines.iter().map(|l| l.trim()).find(|l| !l.is_empty()).unwrap_or("");
        let label = if is_page_number(first) { None } else { parse_note_label(first) };
        let joined = lines.join("\n");
        if let Some(label) = label {
            notes.push(PageNote { label, text: joined });
            last_bottom = Some(y1);
        } else if let (Some(bottom), Some(note)) = (last_bottom, notes.last_mut()) {
            if y0 - bottom <= size * 1.5 {
                note.text.push('\n');
                note.text.push_str(&joined);
                last_bottom = Some(y1);
            } else {
                last_bottom = None;
            }
        }
    }
    notes
}

fn superscript_digit(c: char) -> Option<char> {
    match c {
        '\u{2070}' => Some('0'),
        '\u{00B9}' => Some('1'),
        '\u{00B2}' => Some('2'),
        '\u{00B3}' => Some('3'),
        '\u{2074}'..='\u{2079}' => char::from_digit(c as u32 - 0x2070, 10),
        _ => None,
    }
}

fn is_note_symbol(c: char) -> bool {
    matches!(c, '*' | '\u{2020}' | '\u{2021}' | '\u{00A7}')
}

/// Returns the note label a note's first line starts with: "1 Text" -> "1",
/// "12. Text" -> "12", "¹ Text" -> "1", "† Text" -> "†". The label must be
/// followed by note text.
fn parse_note_label(line: &str) -> Option<String> {
    let line = line.trim();

    let sup: String = line.chars().map_while(superscript_digit).collect();
    if !sup.is_empty() {
        let rest = line.chars().skip(sup.chars().count()).collect::<String>();
        return (!rest.trim().is_empty()).then_some(sup);
    }

    let digits: String = line.chars().take_while(|c| c.is_ascii_digit()).collect();
    if !digits.is_empty() {
        if digits.len() > 3 {
            return None;
        }
        let rest = &line[digits.len()..];
        let rest = rest.strip_prefix(['.', ')']).unwrap_or(rest);
        // "1 If you've..." — the label is separated from the note text.
        if !rest.starts_with(char::is_whitespace) {
            return None;
        }
        let rest = rest.trim_start();
        return rest
            .chars()
            .next()
            .filter(|c| !c.is_ascii_digit())
            .map(|_| digits);
    }

    let symbols: String = line.chars().take_while(|&c| is_note_symbol(c)).collect();
    if !symbols.is_empty() {
        let rest = &line[symbols.len()..];
        return (!rest.trim().is_empty()).then_some(symbols);
    }

    None
}

/// Letters and digits only: compares text regardless of how MuPDF joined
/// lines (dehyphenation drops the hyphen and the line break).
fn text_key(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).collect()
}

/// Takes out the run of consecutive lines of `text` whose combined text is
/// `target` (compared with `text_key`), returning the remaining text and the
/// removed lines. Scans from the end so a note is matched before any similar
/// line in the body. `None` when no such run exists.
fn take_text_run(text: &str, target: &str) -> Option<(String, Vec<String>)> {
    let target_key = text_key(target);
    if target_key.is_empty() {
        return None;
    }
    let lines: Vec<&str> = text.lines().collect();
    for start in (0..lines.len()).rev() {
        let mut acc = text_key(lines[start]);
        if acc.is_empty() || !target_key.starts_with(&acc) {
            continue;
        }
        let mut end = start;
        while acc.len() < target_key.len() && end + 1 < lines.len() {
            end += 1;
            acc.push_str(&text_key(lines[end]));
            if !target_key.starts_with(&acc) {
                break;
            }
        }
        if acc == target_key {
            let rest = lines[..start]
                .iter()
                .chain(&lines[end + 1..])
                .copied()
                .collect::<Vec<_>>()
                .join("\n");
            let removed = lines[start..=end].iter().map(|l| l.to_string()).collect();
            return Some((rest, removed));
        }
    }
    None
}

/// Replaces reference marks to the page's notes in the body text: digits or
/// note symbols set small and raised right after a word ("algorithms.¹").
/// `marks` maps each note label to its replacement (empty to remove it).
/// Only marks whose label matches a note found on the page are touched, so
/// exponents and other superscripts stay.
fn replace_note_marks(text: &str, text_page: &TextPage, marks: &[(&str, String)]) -> String {
    let labels: Vec<&str> = marks.iter().map(|(l, _)| *l).collect();
    let mut out = text.to_string();
    for block in text_page.blocks() {
        if block.r#type() != TextBlockType::Text {
            continue;
        }
        for line in block.lines() {
            // (char, size, baseline y)
            let glyphs: Vec<(char, f32, f32)> = line
                .chars()
                .filter_map(|ch| ch.char().map(|c| (c, ch.size(), ch.origin().y)))
                .collect();
            for (start, end) in find_note_marks(&glyphs, &labels) {
                let label: String = glyphs[start..end]
                    .iter()
                    .map(|g| superscript_digit(g.0).unwrap_or(g.0))
                    .collect();
                let Some((_, replacement)) = marks.iter().find(|(l, _)| *l == label) else {
                    continue;
                };
                let context_start = start.saturating_sub(24);
                let original: String = glyphs[context_start..end].iter().map(|g| g.0).collect();
                let mut replaced: String = glyphs[context_start..start].iter().map(|g| g.0).collect();
                replaced.push_str(replacement);
                if let Some(pos) = out.find(&original) {
                    out.replace_range(pos..pos + original.len(), &replaced);
                }
            }
        }
    }
    out
}

/// Returns the glyph ranges `start..end` of reference marks on one line.
fn find_note_marks(glyphs: &[(char, f32, f32)], labels: &[&str]) -> Vec<(usize, usize)> {
    let line_size = glyphs
        .iter()
        .filter(|g| !g.0.is_whitespace())
        .map(|g| g.1)
        .fold(0.0f32, f32::max);
    if line_size <= 0.0 {
        return Vec::new();
    }
    let is_mark_glyph = |(c, size, _): (char, f32, f32)| {
        (c.is_ascii_digit() || superscript_digit(c).is_some() || is_note_symbol(c))
            && size <= line_size * NOTE_MARK_MAX_SIZE_RATIO
    };

    let mut marks = Vec::new();
    let mut i = 1;
    while i < glyphs.len() {
        let (prev_char, _, prev_y) = glyphs[i - 1];
        if !is_mark_glyph(glyphs[i]) || prev_char.is_whitespace() || prev_char.is_ascii_digit() {
            i += 1;
            continue;
        }
        let start = i;
        while i < glyphs.len() && is_mark_glyph(glyphs[i]) {
            i += 1;
        }
        let (_, size, y) = glyphs[start];
        let raised = prev_y - y >= size * NOTE_MARK_MIN_RISE_RATIO;
        let label: String = glyphs[start..i]
            .iter()
            .map(|g| superscript_digit(g.0).unwrap_or(g.0))
            .collect();
        if raised && labels.contains(&label.as_str()) {
            marks.push((start, i));
        }
    }
    marks
}

fn strip_page_numbers(pages: &mut Vec<String>) {
    for page in pages.iter_mut() {
        // Scan every line, not just the page string's first/last line: running
        // headers/footers can end up mid-string when MuPDF's block ordering
        // places a marginal text block ahead of the body text ending, so a
        // boundary-only check misses them.
        let filtered: Vec<&str> = page
            .lines()
            .filter(|l| !is_page_number(l.trim()))
            .collect();
        *page = filtered.join("\n");
    }
}

fn is_page_number(line: &str) -> bool {
    if line.is_empty() {
        return false;
    }

    if line.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }

    if let Some(rest) = line
        .strip_prefix("Page ")
        .or_else(|| line.strip_prefix("page "))
    {
        if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) {
            return true;
        }
    }

    if is_dashed_number(line) {
        return true;
    }

    if line.len() >= 2 && is_roman_numeral(line) {
        return true;
    }

    // Pipe-separated footers: "|", "| v", "6 | Chapter 1", "xviii | Introduction"
    // Any segment around a "|" that is a bare page number (digits or Roman numerals)
    // identifies the whole line as a footer.
    if line.contains('|') {
        let is_page_num_segment = |s: &str| -> bool {
            let s = s.trim();
            s.is_empty()
                || s.chars().all(|c| c.is_ascii_digit())
                || is_roman_numeral(s)
        };
        if line.split('|').any(is_page_num_segment) {
            return true;
        }
    }

    false
}

fn is_dashed_number(line: &str) -> bool {
    fn is_dash(c: char) -> bool {
        matches!(c, '-' | '\u{2013}' | '\u{2014}')
    }

    let mut chars = line.chars().peekable();

    if chars.peek().map_or(true, |c| !is_dash(*c)) {
        return false;
    }
    chars.next();

    while chars.peek() == Some(&' ') {
        chars.next();
    }

    let mut has_digit = false;
    while chars.peek().map_or(false, |c| c.is_ascii_digit()) {
        has_digit = true;
        chars.next();
    }
    if !has_digit {
        return false;
    }

    while chars.peek() == Some(&' ') {
        chars.next();
    }

    if chars.peek().map_or(true, |c| !is_dash(*c)) {
        return false;
    }
    chars.next();

    chars.peek().is_none()
}

fn is_roman_numeral(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| matches!(c, 'i' | 'v' | 'x' | 'l' | 'c' | 'd' | 'm' | 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M'))
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- table of contents detection --

    #[test]
    fn toc_title_matches_common_headings() {
        assert!(is_toc_title("Table of Contents"));
        assert!(is_toc_title("  CONTENTS "));
        assert!(is_toc_title("Contents:"));
        assert!(is_toc_title("Brief  Contents"));
        assert!(!is_toc_title("Contents of the Repository"));
        assert!(!is_toc_title("Preface"));
    }

    #[test]
    fn first_line_skips_blanks_and_heading_marker() {
        let text = format!("\n  \n{}1Table of Contents\nPreface . . . 1", HEADING_MARK);
        assert_eq!(first_line(&text), Some("Table of Contents"));
        assert_eq!(first_line("   \n"), None);
    }

    #[test]
    fn find_toc_page_falls_back_to_text_heading() {
        let pages: Vec<String> = vec![
            "Architecture Patterns".into(),
            "".into(),
            "Copyright © 2020".into(),
            "Table of Contents\nPreface . . . xi".into(),
            "Preface\nYou may be wondering".into(),
        ];
        assert_eq!(find_toc_page(&[], &pages), Some(3));
    }

    #[test]
    fn find_toc_page_none_without_toc() {
        let pages: Vec<String> = vec!["Title".into(), "Chapter 1\nText".into()];
        assert_eq!(find_toc_page(&[], &pages), None);
    }

    #[test]
    fn find_toc_end_scans_continuation_pages() {
        let pages: Vec<String> = vec![
            "Table of Contents\nPreface . . . xi\n1. Intro 1\n2. Basics 9".into(),
            "3. Types . . . 20\n4. Traits 35\n5. Generics 50\nIndex 300".into(),
            "Preface\nThis book is about Rust.\nIt covers many things.\nEnjoy.".into(),
        ];
        assert_eq!(find_toc_end(&[], &pages, 0), 2);
    }

    #[test]
    fn find_toc_end_single_page_toc() {
        let pages: Vec<String> = vec![
            "Contents\nPreface xi\nIntro 1".into(),
            "Chapter 1\nOnce upon a time there was a crab.\nIt liked Rust.".into(),
        ];
        assert_eq!(find_toc_end(&[], &pages, 0), 1);
    }

    #[test]
    fn find_toc_end_stops_at_document_end() {
        let pages: Vec<String> = vec!["Contents\nA 1\nB 2\nC 3".into(), "D 4\nE 5\nF 6".into()];
        assert_eq!(find_toc_end(&[], &pages, 0), 2);
    }

    #[test]
    fn ends_with_page_ref_detects_numbers() {
        assert!(ends_with_page_ref("Preface . . . xi"));
        assert!(ends_with_page_ref("1. Introduction 3"));
        assert!(!ends_with_page_ref("This is a sentence."));
    }

    #[test]
    fn find_toc_page_ignores_contents_heading_past_front_matter() {
        let mut pages: Vec<String> = (0..TOC_SCAN_PAGES).map(|i| format!("Page {}", i)).collect();
        pages.push("Contents\nChapter about contents".into());
        assert_eq!(find_toc_page(&[], &pages), None);
    }

    #[test]
    fn is_page_number_bare_digits() {
        assert!(is_page_number("1"));
        assert!(is_page_number("42"));
        assert!(is_page_number("999"));
    }

    #[test]
    fn is_page_number_page_prefix() {
        assert!(is_page_number("Page 1"));
        assert!(is_page_number("page 42"));
        assert!(is_page_number("Page 100"));
    }

    #[test]
    fn is_page_number_dashed() {
        assert!(is_page_number("- 42 -"));
        assert!(is_page_number("-42-"));
        assert!(is_page_number("\u{2014} 7 \u{2014}"));
        assert!(is_page_number("\u{2013}12\u{2013}"));
    }

    #[test]
    fn is_page_number_roman_numerals() {
        assert!(is_page_number("ii"));
        assert!(is_page_number("iv"));
        assert!(is_page_number("xii"));
        assert!(is_page_number("III"));
        assert!(is_page_number("XIV"));
    }

    #[test]
    fn is_page_number_rejects_text() {
        assert!(!is_page_number("Chapter 1"));
        assert!(!is_page_number("Hello world"));
        assert!(!is_page_number(""));
        assert!(!is_page_number("Page"));
        assert!(!is_page_number("Introduction"));
    }

    #[test]
    fn is_page_number_rejects_single_roman_char() {
        assert!(!is_page_number("I"));
        assert!(!is_page_number("V"));
        assert!(!is_page_number("X"));
    }

    // -- note removal --

    #[test]
    fn parse_note_label_digits_then_text() {
        assert_eq!(parse_note_label("1 If you've come across"), Some("1".into()));
        assert_eq!(parse_note_label("12. See chapter 3."), Some("12".into()));
        assert_eq!(parse_note_label("3) Or perhaps"), Some("3".into()));
    }

    #[test]
    fn parse_note_label_superscript_and_symbols() {
        assert_eq!(parse_note_label("\u{00B9} First note."), Some("1".into()));
        assert_eq!(parse_note_label("\u{00B9}\u{2070}Tenth note."), Some("10".into()));
        assert_eq!(parse_note_label("* Starred note."), Some("*".into()));
        assert_eq!(parse_note_label("\u{2020} Dagger note."), Some("\u{2020}".into()));
    }

    #[test]
    fn parse_note_label_rejects_non_notes() {
        assert_eq!(parse_note_label("6"), None);
        assert_eq!(parse_note_label("2024 was a year"), None);
        assert_eq!(parse_note_label("10 20 30"), None);
        assert_eq!(parse_note_label("3rd edition"), None);
        assert_eq!(parse_note_label("Introduction"), None);
        assert_eq!(parse_note_label("*"), None);
    }

    fn candidate(y0: f32, y1: f32, lines: &[&str]) -> (f32, f32, f32, Vec<String>) {
        (y0, y1, 8.0, lines.iter().map(|s| s.to_string()).collect())
    }

    #[test]
    fn group_page_notes_joins_continuation_block() {
        let notes = group_page_notes(vec![
            candidate(560.0, 580.0, &["1 First note that wraps"]),
            candidate(582.0, 590.0, &["onto a second block."]),
            candidate(592.0, 600.0, &["2 Second note."]),
        ]);
        assert_eq!(
            notes,
            vec![
                PageNote { label: "1".into(), text: "1 First note that wraps\nonto a second block.".into() },
                PageNote { label: "2".into(), text: "2 Second note.".into() },
            ]
        );
    }

    #[test]
    fn group_page_notes_ignores_distant_and_unlabelled_blocks() {
        let notes = group_page_notes(vec![
            candidate(500.0, 510.0, &["A small caption"]),
            candidate(560.0, 580.0, &["1 A note."]),
            candidate(610.0, 621.0, &["Introduction"]),
            candidate(630.0, 640.0, &["20 | Chapter 1"]),
        ]);
        assert_eq!(notes, vec![PageNote { label: "1".into(), text: "1 A note.".into() }]);
    }

    #[test]
    fn take_text_run_handles_dehyphenated_note() {
        let text = "1 If you\u{2019}ve come across cards: thinking about it helps.\n\nIn the literature of the OO world.\nThe end.";
        let note = "1 If you\u{2019}ve come across cards: think\u{2010}\ning about it helps.";
        let (rest, removed) = take_text_run(text, note).unwrap();
        assert_eq!(rest, "\nIn the literature of the OO world.\nThe end.");
        assert_eq!(removed, vec!["1 If you\u{2019}ve come across cards: thinking about it helps."]);
    }

    #[test]
    fn take_text_run_spans_several_lines() {
        let text = "Body.\n2 A note on\ntwo lines.\nMore body.";
        let (rest, removed) = take_text_run(text, "2 A note on two lines.").unwrap();
        assert_eq!(rest, "Body.\nMore body.");
        assert_eq!(removed, vec!["2 A note on", "two lines."]);
    }

    #[test]
    fn take_text_run_none_without_match() {
        assert!(take_text_run("Body.\nMore body.", "1 Missing note.").is_none());
    }

    #[test]
    fn strip_note_label_variants() {
        assert_eq!(strip_note_label("1 If you\u{2019}ve come", "1"), "If you\u{2019}ve come");
        assert_eq!(strip_note_label("12. See  chapter 3.", "12"), "See chapter 3.");
        assert_eq!(strip_note_label("\u{00B9}First note.", "1"), "First note.");
        assert_eq!(strip_note_label("\u{2020} Dagger note.", "\u{2020}"), "Dagger note.");
    }

    #[test]
    fn note_ref_format() {
        assert_eq!(
            note_ref(7, "1"),
            format!("{NOTE_REF_START}7{NOTE_FIELD_SEP}1{NOTE_REF_END}")
        );
    }

    fn glyphs(body: &str, mark: &str) -> Vec<(char, f32, f32)> {
        body.chars()
            .map(|c| (c, 9.6, 108.0))
            .chain(mark.chars().map(|c| (c, 6.0, 104.8)))
            .collect()
    }

    #[test]
    fn find_note_marks_detects_small_raised_label() {
        let g = glyphs("data or algorithms.", "1");
        assert_eq!(find_note_marks(&g, &["1"]), vec![(19, 20)]);
    }

    #[test]
    fn find_note_marks_requires_matching_label() {
        let g = glyphs("data or algorithms.", "2");
        assert!(find_note_marks(&g, &["1"]).is_empty());
    }

    #[test]
    fn find_note_marks_ignores_baseline_digits() {
        let g: Vec<_> = "chapter 1 and x1".chars().map(|c| (c, 9.6, 108.0)).collect();
        assert!(find_note_marks(&g, &["1"]).is_empty());
    }

    #[test]
    fn find_note_marks_ignores_exponent_after_digit() {
        let g = glyphs("10", "1");
        assert!(find_note_marks(&g, &["1"]).is_empty());
    }

    // -- running header/footer removal --

    fn margin_line(y: f32, x: f32, s: &str) -> (f32, f32, String) {
        (y, x, s.to_string())
    }

    #[test]
    fn margin_row_split_footer_removed_whole() {
        let lines = vec![
            margin_line(610.2, 95.5, "Chapter 1: Domain Modeling"),
            margin_line(610.2, 72.0, "6"),
            margin_line(610.2, 84.6, "|"),
        ];
        let mut got = margin_rows_to_remove(lines);
        got.sort();
        assert_eq!(got, vec!["6", "Chapter 1: Domain Modeling", "|"]);
    }

    #[test]
    fn margin_row_right_aligned_roman_footer_removed() {
        let lines = vec![
            margin_line(610.0, 300.0, "Preface"),
            margin_line(610.5, 340.0, "|"),
            margin_line(610.3, 350.0, "xi"),
        ];
        assert_eq!(margin_rows_to_remove(lines).len(), 3);
    }

    #[test]
    fn margin_row_without_page_number_kept() {
        let lines = vec![
            margin_line(40.0, 72.0, "What Is a Domain Model?"),
            margin_line(612.0, 72.0, "Some closing sentence of the page."),
        ];
        assert!(margin_rows_to_remove(lines).is_empty());
    }

    #[test]
    fn margin_rows_judged_independently() {
        let lines = vec![
            margin_line(590.0, 72.0, "Last body line near the bottom."),
            margin_line(610.0, 428.0, "5"),
        ];
        assert_eq!(margin_rows_to_remove(lines), vec!["5"]);
    }

    #[test]
    fn remove_lines_drops_footer_not_body_duplicate() {
        let text = "Preface\nBody text.\n\nxii \n| \nPreface";
        let targets = vec!["xii".to_string(), "|".to_string(), "Preface".to_string()];
        assert_eq!(remove_lines(text, &targets), "Preface\nBody text.\n");
    }

    #[test]
    fn strip_removes_trailing_page_number() {
        let mut pages = vec!["Some content here.\n\n42".to_string()];
        strip_page_numbers(&mut pages);
        assert_eq!(pages[0], "Some content here.\n");
    }

    #[test]
    fn strip_removes_leading_page_number() {
        let mut pages = vec!["7\n\nSome content here.".to_string()];
        strip_page_numbers(&mut pages);
        assert_eq!(pages[0], "\nSome content here.");
    }

    #[test]
    fn strip_removes_both_header_and_footer_numbers() {
        let mut pages = vec!["Page 3\n\nContent\n\n- 3 -".to_string()];
        strip_page_numbers(&mut pages);
        assert_eq!(pages[0], "\nContent\n");
    }

    #[test]
    fn strip_leaves_content_only_pages_intact() {
        let mut pages = vec!["Just a paragraph of text.".to_string()];
        let original = pages[0].clone();
        strip_page_numbers(&mut pages);
        assert_eq!(pages[0], original);
    }

    #[test]
    fn strip_handles_roman_numeral_footer() {
        let mut pages = vec!["Preface text.\n\niv".to_string()];
        strip_page_numbers(&mut pages);
        assert_eq!(pages[0], "Preface text.\n");
    }

    #[test]
    fn strip_skips_empty_pages() {
        let mut pages = vec!["".to_string(), "   ".to_string()];
        strip_page_numbers(&mut pages);
        assert_eq!(pages[0], "");
        assert_eq!(pages[1], "   ");
    }

    #[test]
    fn is_page_number_rejects_digits_mixed_with_text() {
        assert!(!is_page_number("42 Main Street"));
        assert!(!is_page_number("Chapter 1"));
        assert!(!is_page_number("3rd Edition"));
        assert!(!is_page_number("Page 42 of 100"));
    }

    #[test]
    fn is_page_number_rejects_page_prefix_without_number() {
        assert!(!is_page_number("Page abc"));
        assert!(!is_page_number("page "));
        assert!(!is_page_number("Page"));
    }

    #[test]
    fn is_page_number_rejects_incomplete_dashed_patterns() {
        assert!(!is_dashed_number("-42"));
        assert!(!is_dashed_number("42-"));
        assert!(!is_dashed_number("- -"));
        assert!(!is_dashed_number("--"));
        assert!(!is_dashed_number("- hello -"));
    }

    #[test]
    fn is_page_number_accepts_mixed_dash_types() {
        assert!(is_dashed_number("- 42 \u{2014}"));
        assert!(is_dashed_number("\u{2013} 7 -"));
    }

    #[test]
    fn is_roman_numeral_rejects_non_roman_chars() {
        assert!(!is_roman_numeral("hello"));
        assert!(!is_roman_numeral("abc"));
        assert!(!is_roman_numeral("ixa"));
        assert!(!is_roman_numeral(""));
    }

    #[test]
    fn strip_processes_multiple_pages_independently() {
        let mut pages = vec![
            "42\n\nFirst page content.".to_string(),
            "Some middle content.".to_string(),
            "Last page content.\n\n99".to_string(),
        ];
        strip_page_numbers(&mut pages);
        assert_eq!(pages[0], "\nFirst page content.");
        assert_eq!(pages[1], "Some middle content.");
        assert_eq!(pages[2], "Last page content.\n");
    }

    #[test]
    fn strip_removes_page_that_is_only_a_number() {
        let mut pages = vec!["42".to_string()];
        strip_page_numbers(&mut pages);
        assert_eq!(pages[0], "");
    }

    #[test]
    fn strip_skips_past_leading_blank_lines_to_find_page_number() {
        let mut pages = vec!["\n\n  \n42\n\nContent here.".to_string()];
        strip_page_numbers(&mut pages);
        assert_eq!(pages[0], "\n\n  \n\nContent here.");
    }

    #[test]
    fn strip_skips_past_trailing_blank_lines_to_find_page_number() {
        let mut pages = vec!["Content here.\n\n42\n\n  \n".to_string()];
        strip_page_numbers(&mut pages);
        assert_eq!(pages[0], "Content here.\n\n\n  ");
    }

    #[test]
    fn strip_preserves_interior_numbers() {
        let mut pages = vec!["Heading\n\nThere are 42 items in the list.\n\nMore text.".to_string()];
        let original = pages[0].clone();
        strip_page_numbers(&mut pages);
        assert_eq!(pages[0], original);
    }

    #[test]
    fn strip_does_not_remove_long_first_or_last_lines() {
        let mut pages = vec!["This is a real sentence, not a page number.\n\nMiddle.\n\nAnother real sentence at the end.".to_string()];
        let original = pages[0].clone();
        strip_page_numbers(&mut pages);
        assert_eq!(pages[0], original);
    }

    #[test]
    fn is_page_number_handles_large_numbers() {
        assert!(is_page_number("1234"));
        assert!(is_page_number("9999"));
    }

    #[test]
    fn is_page_number_pipe_standalone() {
        assert!(is_page_number("|"));
    }

    #[test]
    fn is_page_number_pipe_prefixed_roman() {
        assert!(is_page_number("| v"));
        assert!(is_page_number("| xii"));
        assert!(is_page_number("| IV"));
        assert!(is_page_number("|v"));
    }

    #[test]
    fn is_page_number_pipe_prefixed_digits() {
        assert!(is_page_number("| 42"));
        assert!(is_page_number("|5"));
    }

    #[test]
    fn is_page_number_pipe_num_then_title() {
        assert!(is_page_number("6 | Chapter 1: Domain Modeling"));
        assert!(is_page_number("xviii | Introduction"));
        assert!(is_page_number("iv | Table of Contents"));
    }

    #[test]
    fn is_page_number_pipe_rejects_text_both_sides() {
        assert!(!is_page_number("foo | bar"));
        assert!(!is_page_number("See also | Further reading"));
    }

    #[test]
    fn strip_removes_pipe_prefixed_footer() {
        let mut pages = vec!["Table of Contents text.\n\n| v".to_string()];
        strip_page_numbers(&mut pages);
        assert_eq!(pages[0], "Table of Contents text.\n");
    }

    #[test]
    fn strip_removes_num_pipe_title_footer() {
        let mut pages = vec!["Chapter content.\n\nxviii | Introduction".to_string()];
        strip_page_numbers(&mut pages);
        assert_eq!(pages[0], "Chapter content.\n");
    }

    #[test]
    fn strip_removes_two_line_footer_pipe_then_number() {
        // Footer split across two lines: page number on last, pipe on second-to-last
        let mut pages = vec!["Content here.\n|\nvi".to_string()];
        strip_page_numbers(&mut pages);
        assert_eq!(pages[0], "Content here.");
    }

    #[test]
    fn strip_removes_two_line_footer_number_then_pipe() {
        // Footer split: page number on first line, pipe on next
        let mut pages = vec!["42\n|\nContent here.".to_string()];
        strip_page_numbers(&mut pages);
        assert_eq!(pages[0], "Content here.");
    }

    #[test]
    fn strip_with_dashed_page_number_at_footer() {
        let mut pages = vec!["Chapter content goes here.\n\n- 15 -".to_string()];
        strip_page_numbers(&mut pages);
        assert_eq!(pages[0], "Chapter content goes here.\n");
    }

    #[test]
    fn strip_with_page_prefix_at_header() {
        let mut pages = vec!["Page 7\n\nThe actual chapter text starts here.".to_string()];
        strip_page_numbers(&mut pages);
        assert_eq!(pages[0], "\nThe actual chapter text starts here.");
    }

    #[test]
    fn strip_removes_footer_stuck_in_the_middle_of_the_page() {
        // Regression: MuPDF block ordering can place a running footer ahead of
        // trailing body text, so it's neither the first nor the last line.
        let mut pages =
            vec!["Body text before.\n\n6 | Chapter 1: Domain Modeling\n\nBody text after.".to_string()];
        strip_page_numbers(&mut pages);
        assert_eq!(pages[0], "Body text before.\n\n\nBody text after.");
    }

    // -- image anchoring --

    fn anchor(page_y: f32, mark: &str) -> ImageAnchor {
        ImageAnchor { page_y, mark: mark.to_string() }
    }

    #[test]
    fn image_mark_goes_before_block_below_it() {
        let text = "Intro text.\n\nFigure 1. Caption\n\nMore text.\n";
        let caption = text.find("Figure").unwrap();
        let more = text.find("More").unwrap();
        let blocks = [(0, 50.0), (caption, 300.0), (more, 330.0)];
        let out = image_marks_into(text, &blocks, &[anchor(200.0, "IMG")]);
        assert_eq!(out, "Intro text.\n\nIMG\n\nFigure 1. Caption\n\nMore text.\n");
    }

    #[test]
    fn image_mark_picks_closest_block_below_not_first_in_text_order() {
        // Text order lists a lower block (y 500) before the caption (y 300).
        let text = "Low block.\n\nCaption.\n";
        let caption = text.find("Caption").unwrap();
        let blocks = [(0, 500.0), (caption, 300.0)];
        let out = image_marks_into(text, &blocks, &[anchor(200.0, "IMG")]);
        assert_eq!(out, "Low block.\n\nIMG\n\nCaption.\n");
    }

    #[test]
    fn image_mark_below_all_text_goes_at_end() {
        let text = "Only text.";
        let out = image_marks_into(text, &[(0, 50.0)], &[anchor(400.0, "IMG")]);
        assert_eq!(out, "Only text.\nIMG\n\n");
    }

    #[test]
    fn image_mark_round_trips_fields() {
        let mark = image_mark("images/a_1_10.jpg", Some(12), "Image from page 3");
        assert!(mark.starts_with(IMAGE_MARK));
        assert!(mark.contains("images/a_1_10.jpg"));
    }
}
