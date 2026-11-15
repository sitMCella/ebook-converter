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

pub fn extract_text(
    path: &str,
    options: &PageHandlingOptions,
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
        text_pages.push(text_page);
    }

    let body_size = compute_body_size(&text_pages);

    let mut pages = Vec::with_capacity(text_pages.len());
    for (i, text_page) in text_pages.iter().enumerate() {
        let text = text_page
            .to_text()
            .map_err(|e| format!("Failed to extract text from page {}: {}", i + 1, e))?;
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

        for line in block.lines() {
            let mut line_text = String::new();
            let mut is_mono: Option<bool> = None;
            for ch in line.chars() {
                if let Some(c) = ch.char() {
                    line_text.push(c);
                }
                if is_mono.is_none() {
                    if let Some(font) = ch.font() {
                        is_mono = Some(font.is_monospaced());
                    }
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

        // Require at least two non-blank lines, all set in a monospaced font,
        // to treat the block as code rather than a single inline `code` word
        // or a stray monospace caption sharing a line with regular prose.
        if text_lines < 2 || mono_lines != text_lines {
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
}
