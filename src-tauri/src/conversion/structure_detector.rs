use super::text_extractor::{
    CODE_BLOCK_END, CODE_BLOCK_START, HEADING_MARK, IMAGE_MARK, NOTE_FIELD_SEP, NOTE_REF_START, NOTE_START,
};
use super::StructureOptions;

#[derive(Debug, Clone)]
pub enum StructuredContent {
    Heading { level: u8, text: String },
    Paragraph { text: String },
    /// `number` is the item's printed label for ordered lists ("3." -> 3),
    /// so a list split by an interleaved paragraph keeps its numbering.
    ListItem { text: String, ordered: bool, number: Option<u32> },
    Image { resource_path: String, alt: String, display_width_pct: Option<u32> },
    Footnote { number: u32, text: String },
    /// A note kept from the bottom of a PDF page (Keep notes on). `id` is
    /// unique across the document and matches the `NOTE_REF_START` marks in
    /// the text that cite it; `label` is the printed mark ("1", "*").
    Note { id: u32, label: String, text: String },
    CodeBlock { text: String },
    /// A chapter label printed above a chapter title ("CHAPTER 1", "Part II",
    /// "Appendix A"), rendered as a small line attached to the title.
    ChapterLabel { text: String },
    BlankLine,
    PageBreak,
}

pub fn detect_structure(
    pages: &[String],
    options: &StructureOptions,
) -> Vec<StructuredContent> {
    let mut result = Vec::new();
    let mut all_footnotes = Vec::new();

    for (i, page) in pages.iter().enumerate() {
        let lines: Vec<&str> = page.lines().collect();

        let body_end = if options.detect_footnotes {
            let (footnotes, end) = extract_page_footnotes(&lines);
            all_footnotes.extend(footnotes);
            end
        } else {
            lines.len()
        };

        let mut line_idx = 0;

        while line_idx < body_end {
            let line = lines[line_idx].trim();

            if let Some(note) = parse_note_line(line) {
                result.push(note);
                line_idx += 1;
                continue;
            }

            if let Some(image) = parse_image_line(line) {
                result.push(image);
                line_idx += 1;
                continue;
            }

            if line.chars().next() == Some(CODE_BLOCK_START) {
                line_idx += 1;
                let mut code_lines = Vec::new();
                while line_idx < body_end && lines[line_idx].trim().chars().next() != Some(CODE_BLOCK_END) {
                    code_lines.push(lines[line_idx]);
                    line_idx += 1;
                }
                // Skip the closing marker line itself, if found.
                if line_idx < body_end {
                    line_idx += 1;
                }
                // MuPDF splits a listing into one text block per run of lines
                // without a vertical gap, so a blank line inside the code
                // ends up as two adjacent code blocks. Merge code blocks that
                // are separated only by blank lines back into one listing.
                loop {
                    let mut peek = line_idx;
                    while peek < body_end && lines[peek].trim().is_empty() {
                        peek += 1;
                    }
                    if peek >= body_end || lines[peek].trim().chars().next() != Some(CODE_BLOCK_START) {
                        break;
                    }
                    code_lines.push("");
                    line_idx = peek + 1;
                    while line_idx < body_end && lines[line_idx].trim().chars().next() != Some(CODE_BLOCK_END) {
                        code_lines.push(lines[line_idx]);
                        line_idx += 1;
                    }
                    if line_idx < body_end {
                        line_idx += 1;
                    }
                }
                result.push(StructuredContent::CodeBlock {
                    text: code_lines.join("\n"),
                });
                continue;
            }

            if line.is_empty() {
                if options.paragraph_detection {
                    result.push(StructuredContent::BlankLine);
                }
                line_idx += 1;
                continue;
            }

            if options.list_detection {
                if let Some(mut item) = detect_list_item(line) {
                    line_idx += 1;
                    // A list item wrapped over several PDF lines arrives as
                    // consecutive lines; join them into the one item so the
                    // continuation keeps the item's indentation instead of
                    // becoming a separate paragraph.
                    if options.paragraph_detection {
                        if let StructuredContent::ListItem { text, .. } = &mut item {
                            while line_idx < body_end {
                                let next = lines[line_idx].trim();
                                if !is_continuation_line(next, options) {
                                    break;
                                }
                                text.push(' ');
                                text.push_str(next);
                                line_idx += 1;
                            }
                        }
                    }
                    result.push(item);
                    continue;
                }
            }

            if options.detect_headings {
                if let Some((level, text)) = parse_heading_mark(line) {
                    if level <= options.heading_level_threshold {
                        result.push(StructuredContent::Heading {
                            level,
                            text: text.to_string(),
                        });
                        line_idx += 1;
                        continue;
                    }
                    // Marked but below the requested threshold: fall through
                    // and treat its real text as ordinary content below.
                }
            }
            let line = strip_heading_mark(line);

            if options.paragraph_detection {
                let mut paragraph_text = String::from(line);
                line_idx += 1;
                while line_idx < lines.len() {
                    let next = lines[line_idx].trim();
                    if next.is_empty() || next.starts_with(NOTE_START) || next.starts_with(IMAGE_MARK) {
                        break;
                    }
                    if options.detect_headings && next.chars().next() == Some(HEADING_MARK) {
                        break;
                    }
                    if options.list_detection && detect_list_item(next).is_some() {
                        break;
                    }
                    paragraph_text.push(' ');
                    paragraph_text.push_str(next);
                    line_idx += 1;
                }
                result.push(StructuredContent::Paragraph {
                    text: paragraph_text,
                });
                continue;
            }

            result.push(StructuredContent::Paragraph {
                text: line.to_string(),
            });
            line_idx += 1;
        }

        if i < pages.len() - 1 {
            result.push(StructuredContent::PageBreak);
        }
    }

    for (number, text) in all_footnotes {
        result.push(StructuredContent::Footnote { number, text });
    }

    place_notes_after_refs(result)
}

/// Words that introduce a chapter label line ("CHAPTER 1", "Part II").
const CHAPTER_LABEL_WORDS: &[&str] = &["chapter", "part", "appendix", "book", "section", "lesson", "unit"];

/// True when `text` is a bare chapter label: one of `CHAPTER_LABEL_WORDS`
/// followed by a single short number/letter token ("1", "II", "A", "One").
fn is_chapter_label(text: &str) -> bool {
    let mut words = text.split_whitespace();
    let (Some(word), Some(number), None) = (words.next(), words.next(), words.next()) else {
        return false;
    };
    CHAPTER_LABEL_WORDS.contains(&word.to_lowercase().as_str())
        && number.chars().count() <= 12
        && number.chars().all(|c| c.is_alphanumeric())
}

/// Handles the chapter label printed above a chapter title ("CHAPTER 1" over
/// "Domain Modeling"). Font-size heading detection sees it as a separate,
/// smaller heading, so it ended up stranded before the title's page break.
/// A heading or paragraph that is only a chapter label and is followed
/// (blank lines aside) by a level-1/2 heading becomes a `ChapterLabel` kept
/// with that title, or is removed when `keep` is false.
pub fn apply_chapter_labels(content: Vec<StructuredContent>, keep: bool) -> Vec<StructuredContent> {
    let mut result = Vec::with_capacity(content.len());
    let mut items = content.into_iter().peekable();
    let mut pending: Vec<StructuredContent> = Vec::new();
    while let Some(item) = items.next() {
        let label = match &item {
            StructuredContent::Heading { text, .. } | StructuredContent::Paragraph { text }
                if is_chapter_label(text.trim()) =>
            {
                Some(text.trim().to_string())
            }
            _ => None,
        };
        let Some(label) = label else {
            result.push(item);
            continue;
        };
        // Look past blank lines for the chapter title.
        pending.clear();
        while matches!(items.peek(), Some(StructuredContent::BlankLine)) {
            pending.push(items.next().unwrap());
        }
        let followed_by_title = matches!(
            items.peek(),
            Some(StructuredContent::Heading { level, text }) if *level <= 2 && !is_chapter_label(text.trim())
        );
        if !followed_by_title {
            result.push(item);
        } else if keep {
            result.push(StructuredContent::ChapterLabel { text: label });
        }
        result.append(&mut pending);
    }
    result
}

/// Parses a `NOTE_START` line written by `text_extractor` for a kept note.
fn parse_note_line(line: &str) -> Option<StructuredContent> {
    let rest = line.strip_prefix(NOTE_START)?;
    let mut fields = rest.splitn(3, NOTE_FIELD_SEP);
    let id = fields.next()?.parse().ok()?;
    let label = fields.next()?.to_string();
    let text = fields.next()?.trim().to_string();
    Some(StructuredContent::Note { id, label, text })
}

/// Parses an `IMAGE_MARK` line written by `text_extractor::image_mark`.
fn parse_image_line(line: &str) -> Option<StructuredContent> {
    let rest = line.strip_prefix(IMAGE_MARK)?;
    let mut fields = rest.splitn(3, NOTE_FIELD_SEP);
    let resource_path = fields.next()?.to_string();
    let display_width_pct = fields.next()?.parse().ok();
    let alt = fields.next().unwrap_or("").to_string();
    Some(StructuredContent::Image { resource_path, alt, display_width_pct })
}

/// True when `item`'s text holds a reference mark to note `id`.
fn cites_note(item: &StructuredContent, id: u32) -> bool {
    let text = match item {
        StructuredContent::Heading { text, .. }
        | StructuredContent::Paragraph { text }
        | StructuredContent::ListItem { text, .. }
        | StructuredContent::CodeBlock { text }
        | StructuredContent::Note { text, .. } => text,
        _ => return false,
    };
    text.contains(&format!("{NOTE_REF_START}{id}{NOTE_FIELD_SEP}"))
}

/// Moves every kept note to just after the block that cites it (after any
/// notes already placed there), so a reader meets the note right where it is
/// referenced. Notes nobody cites stay at the end of their page.
fn place_notes_after_refs(content: Vec<StructuredContent>) -> Vec<StructuredContent> {
    let (notes, mut result): (Vec<_>, Vec<_>) = content
        .into_iter()
        .enumerate()
        .partition(|(_, c)| matches!(c, StructuredContent::Note { .. }));
    if notes.is_empty() {
        return result.into_iter().map(|(_, c)| c).collect();
    }

    for (orig_pos, note) in notes {
        let StructuredContent::Note { id, .. } = &note else { unreachable!() };
        let target = result.iter().position(|(_, c)| cites_note(c, *id));
        let insert_at = match target {
            Some(mut i) => {
                i += 1;
                while i < result.len() && matches!(result[i].1, StructuredContent::Note { .. }) {
                    i += 1;
                }
                i
            }
            // Uncited: back where it was, i.e. before the first item that
            // originally followed it.
            None => result.iter().position(|(p, _)| *p > orig_pos).unwrap_or(result.len()),
        };
        result.insert(insert_at, (orig_pos, note));
    }
    result.into_iter().map(|(_, c)| c).collect()
}

/// Reads a `HEADING_MARK` + level-digit prefix (written by
/// `text_extractor::mark_headings` from real font-size data), returning the
/// level and the heading's real text with the marker stripped.
fn parse_heading_mark(line: &str) -> Option<(u8, &str)> {
    let mut chars = line.char_indices();
    let (_, first) = chars.next()?;
    if first != HEADING_MARK {
        return None;
    }
    let (digit_idx, digit_char) = chars.next()?;
    let level = digit_char.to_digit(10)? as u8;
    if level == 0 {
        return None;
    }
    Some((level, &line[digit_idx + digit_char.len_utf8()..]))
}

fn strip_heading_mark(line: &str) -> &str {
    match parse_heading_mark(line) {
        Some((_, text)) => text,
        None => line,
    }
}

/// True when `next` continues the preceding text block: non-empty and not
/// the start of a heading, list item or code block.
fn is_continuation_line(next: &str, options: &StructureOptions) -> bool {
    let first = next.chars().next();
    match first {
        None => false,
        Some(c) if c == CODE_BLOCK_START || c == NOTE_START || c == IMAGE_MARK => false,
        Some(c) if options.detect_headings && c == HEADING_MARK => false,
        _ => !(options.list_detection && detect_list_item(next).is_some()),
    }
}

fn detect_list_item(line: &str) -> Option<StructuredContent> {
    let trimmed = line.trim();

    let bullet_prefixes = ["• ", "– ", "- ", "* "];
    for prefix in &bullet_prefixes {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            if !rest.is_empty() {
                return Some(StructuredContent::ListItem {
                    text: rest.to_string(),
                    ordered: false,
                    number: None,
                });
            }
        }
    }

    if let Some(rest) = strip_numbered_prefix(trimmed) {
        if !rest.is_empty() {
            let number = trimmed
                .split(|c: char| !c.is_ascii_digit())
                .next()
                .and_then(|d| d.parse().ok());
            return Some(StructuredContent::ListItem {
                text: rest.to_string(),
                ordered: true,
                number,
            });
        }
    }

    None
}

#[allow(clippy::manual_strip)]
fn strip_numbered_prefix(line: &str) -> Option<&str> {
    let bytes = line.as_bytes();
    let mut i = 0;

    // Pattern: digits followed by ". " or ") "
    if i < bytes.len() && bytes[i].is_ascii_digit() {
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i < bytes.len() && (bytes[i] == b'.' || bytes[i] == b')') {
            i += 1;
            if i < bytes.len() && bytes[i] == b' ' {
                return Some(&line[i + 1..]);
            }
        }
        return None;
    }

    // Pattern: (letter) or (digit) followed by space
    if i < bytes.len() && bytes[i] == b'(' {
        i += 1;
        if i < bytes.len() && (bytes[i].is_ascii_alphanumeric()) {
            i += 1;
            if i < bytes.len() && bytes[i] == b')' {
                i += 1;
                if i < bytes.len() && bytes[i] == b' ' {
                    return Some(&line[i + 1..]);
                }
            }
        }
    }

    None
}

pub(crate) fn superscript_to_digit(c: char) -> Option<u32> {
    match c {
        '\u{2070}' => Some(0),
        '\u{00B9}' => Some(1),
        '\u{00B2}' => Some(2),
        '\u{00B3}' => Some(3),
        '\u{2074}' => Some(4),
        '\u{2075}' => Some(5),
        '\u{2076}' => Some(6),
        '\u{2077}' => Some(7),
        '\u{2078}' => Some(8),
        '\u{2079}' => Some(9),
        _ => None,
    }
}

fn parse_superscript_start(line: &str) -> Option<(u32, &str)> {
    let mut end_byte = 0;
    let mut number = 0u32;
    let mut has_digit = false;

    for (i, c) in line.char_indices() {
        if let Some(d) = superscript_to_digit(c) {
            number = number * 10 + d;
            has_digit = true;
            end_byte = i + c.len_utf8();
        } else {
            break;
        }
    }

    if !has_digit || number == 0 {
        return None;
    }

    let rest = &line[end_byte..];
    if rest.is_empty() {
        return None;
    }

    let first_after = rest.chars().next()?;
    if !matches!(first_after, ' ' | '\t' | '.' | ':' | ')') {
        return None;
    }

    let rest = if matches!(first_after, '.' | ':' | ')') {
        &rest[1..]
    } else {
        rest
    };
    let rest = rest.trim_start();

    if rest.is_empty() { None } else { Some((number, rest)) }
}

fn parse_regular_digit_start(line: &str) -> Option<(u32, &str)> {
    let bytes = line.as_bytes();
    if bytes.is_empty() || !bytes[0].is_ascii_digit() {
        return None;
    }

    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }

    let num: u32 = line[..i].parse().ok()?;
    if num == 0 || i >= bytes.len() {
        return None;
    }

    let rest = match bytes[i] {
        b'.' | b')' | b':' => &line[i + 1..],
        b' ' | b'\t' => &line[i..],
        _ => return None,
    };
    let rest = rest.trim_start();

    if rest.is_empty() { None } else { Some((num, rest)) }
}

fn parse_footnote_start(line: &str, allow_regular_digits: bool) -> Option<(u32, &str)> {
    if let result @ Some(_) = parse_superscript_start(line) {
        return result;
    }
    if allow_regular_digits {
        return parse_regular_digit_start(line);
    }
    None
}

fn is_footnote_separator(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.len() < 3 {
        return false;
    }
    let total = trimmed.chars().count();
    let sep_count = trimmed
        .chars()
        .filter(|c| matches!(c, '_' | '-' | '─' | '━' | '=' | '—'))
        .count();
    sep_count * 100 / total > 80
}

fn find_footnote_separator(lines: &[&str]) -> Option<usize> {
    let start = lines.len() / 2;
    for i in (start..lines.len()).rev() {
        if is_footnote_separator(lines[i]) {
            return Some(i);
        }
    }
    None
}

fn find_superscript_region_start(lines: &[&str]) -> usize {
    if lines.is_empty() {
        return 0;
    }
    let bottom_portion = lines.len().min(20);
    let search_start = lines.len() - bottom_portion;
    for i in search_start..lines.len() {
        if parse_superscript_start(lines[i].trim()).is_some() {
            return i;
        }
    }
    lines.len()
}

fn parse_footnote_region(lines: &[&str], allow_regular_digits: bool) -> Vec<(u32, String)> {
    let mut footnotes = Vec::new();
    let mut current_num: Option<u32> = None;
    let mut current_text = String::new();

    for line in lines {
        let trimmed = line.trim();

        if trimmed.is_empty() || is_footnote_separator(trimmed) {
            continue;
        }

        if let Some((num, text)) = parse_footnote_start(trimmed, allow_regular_digits) {
            if let Some(n) = current_num {
                let t = current_text.trim().to_string();
                if !t.is_empty() {
                    footnotes.push((n, t));
                }
            }
            current_num = Some(num);
            current_text = text.to_string();
        } else if current_num.is_some() {
            current_text.push(' ');
            current_text.push_str(trimmed);
        }
    }

    if let Some(n) = current_num {
        let t = current_text.trim().to_string();
        if !t.is_empty() {
            footnotes.push((n, t));
        }
    }

    footnotes
}

fn extract_page_footnotes(lines: &[&str]) -> (Vec<(u32, String)>, usize) {
    if lines.is_empty() {
        return (vec![], 0);
    }

    if let Some(sep) = find_footnote_separator(lines) {
        let footnotes = parse_footnote_region(&lines[sep + 1..], true);
        if !footnotes.is_empty() {
            return (footnotes, sep);
        }
    }

    let region_start = find_superscript_region_start(lines);
    if region_start < lines.len() {
        let footnotes = parse_footnote_region(&lines[region_start..], false);
        if !footnotes.is_empty() {
            return (footnotes, region_start);
        }
    }

    (vec![], lines.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversion::text_extractor::NOTE_REF_END;

    fn heading(level: u8, text: &str) -> StructuredContent {
        StructuredContent::Heading { level, text: text.to_string() }
    }

    #[test]
    fn chapter_label_recognized() {
        assert!(is_chapter_label("CHAPTER 1"));
        assert!(is_chapter_label("Part II"));
        assert!(is_chapter_label("Appendix A"));
        assert!(!is_chapter_label("Chapter 1: Domain Modeling"));
        assert!(!is_chapter_label("Chapters"));
        assert!(!is_chapter_label("Domain Modeling"));
    }

    #[test]
    fn chapter_label_kept_before_title() {
        let content = vec![
            heading(2, "CHAPTER 1"),
            StructuredContent::BlankLine,
            heading(1, "Domain Modeling"),
        ];
        let result = apply_chapter_labels(content, true);
        assert!(matches!(&result[0], StructuredContent::ChapterLabel { text } if text == "CHAPTER 1"));
        assert!(matches!(result[1], StructuredContent::BlankLine));
        assert!(matches!(&result[2], StructuredContent::Heading { level: 1, text } if text == "Domain Modeling"));
    }

    #[test]
    fn chapter_label_removed_before_title() {
        let content = vec![heading(2, "CHAPTER 1"), heading(1, "Domain Modeling")];
        let result = apply_chapter_labels(content, false);
        assert_eq!(result.len(), 1);
        assert!(matches!(&result[0], StructuredContent::Heading { level: 1, text } if text == "Domain Modeling"));
    }

    #[test]
    fn chapter_label_without_title_left_alone() {
        let content = vec![
            heading(2, "Chapter 3"),
            StructuredContent::Paragraph { text: "Body.".into() },
        ];
        let result = apply_chapter_labels(content, false);
        assert!(matches!(&result[0], StructuredContent::Heading { level: 2, text } if text == "Chapter 3"));
        assert_eq!(result.len(), 2);
    }

    fn default_options() -> StructureOptions {
        StructureOptions {
            detect_headings: true,
            detect_footnotes: false,
            heading_level_threshold: 3,
            paragraph_detection: true,
            list_detection: true,
        }
    }

    #[test]
    fn detects_marked_heading() {
        let pages = vec![format!("\n{}1INTRODUCTION\n\nSome paragraph text here.", HEADING_MARK)];
        let result = detect_structure(&pages, &default_options());

        let has_heading = result.iter().any(|c| matches!(c, StructuredContent::Heading { level: 1, text } if text == "INTRODUCTION"));
        assert!(has_heading);
    }

    #[test]
    fn detects_paragraph() {
        let pages = vec!["This is a paragraph of text.".to_string()];
        let result = detect_structure(&pages, &default_options());

        let has_paragraph = result.iter().any(|c| matches!(c, StructuredContent::Paragraph { .. }));
        assert!(has_paragraph);
    }

    #[test]
    fn detects_bullet_list() {
        let pages = vec!["- First item\n- Second item".to_string()];
        let result = detect_structure(&pages, &default_options());

        let list_count = result.iter().filter(|c| matches!(c, StructuredContent::ListItem { ordered: false, .. })).count();
        assert_eq!(list_count, 2);
    }

    #[test]
    fn detects_numbered_list() {
        let pages = vec!["1. First item\n2. Second item".to_string()];
        let result = detect_structure(&pages, &default_options());

        let list_count = result.iter().filter(|c| matches!(c, StructuredContent::ListItem { ordered: true, .. })).count();
        assert_eq!(list_count, 2);
    }

    #[test]
    fn wrapped_list_item_lines_join_into_one_item() {
        let pages = vec![
            "1. Test-driven development (TDD) helps us to build code that is correct and enables\nus to refactor or add new features.\n\nNext paragraph.".to_string(),
        ];
        let result = detect_structure(&pages, &default_options());

        let items: Vec<&String> = result
            .iter()
            .filter_map(|c| match c {
                StructuredContent::ListItem { text, .. } => Some(text),
                _ => None,
            })
            .collect();
        assert_eq!(items.len(), 1);
        assert!(items[0].ends_with("enables us to refactor or add new features."));
        let paragraphs = result.iter().filter(|c| matches!(c, StructuredContent::Paragraph { .. })).count();
        assert_eq!(paragraphs, 1);
    }

    #[test]
    fn wrapped_list_item_stops_at_next_item() {
        let pages = vec!["- First item\ncontinued\n- Second item".to_string()];
        let result = detect_structure(&pages, &default_options());

        let items: Vec<&String> = result
            .iter()
            .filter_map(|c| match c {
                StructuredContent::ListItem { text, .. } => Some(text),
                _ => None,
            })
            .collect();
        assert_eq!(items, vec!["First item continued", "Second item"]);
    }

    #[test]
    fn note_moves_after_citing_paragraph() {
        let note = format!("{NOTE_START}3{NOTE_FIELD_SEP}1{NOTE_FIELD_SEP}The note text.");
        let page = format!(
            "First paragraph about algorithms.{NOTE_REF_START}3{NOTE_FIELD_SEP}1{NOTE_REF_END}\n\nSecond paragraph.\n\n{note}"
        );
        let result = detect_structure(&[page], &default_options());
        let kinds: Vec<&str> = result
            .iter()
            .filter_map(|c| match c {
                StructuredContent::Paragraph { .. } => Some("p"),
                StructuredContent::Note { .. } => Some("note"),
                _ => None,
            })
            .collect();
        assert_eq!(kinds, vec!["p", "note", "p"]);
        assert!(result.iter().any(|c| matches!(
            c,
            StructuredContent::Note { id: 3, label, text } if label == "1" && text == "The note text."
        )));
    }

    #[test]
    fn uncited_note_stays_at_end_of_its_page() {
        let note = format!("{NOTE_START}1{NOTE_FIELD_SEP}*{NOTE_FIELD_SEP}Lonely note.");
        let pages = vec![format!("Page one.\n\n{note}"), "Page two.".to_string()];
        let result = detect_structure(&pages, &default_options());
        let note_pos = result.iter().position(|c| matches!(c, StructuredContent::Note { .. })).unwrap();
        let break_pos = result.iter().position(|c| matches!(c, StructuredContent::PageBreak)).unwrap();
        assert!(note_pos < break_pos);
    }

    #[test]
    fn note_line_ends_a_paragraph_without_blank_line() {
        let note = format!("{NOTE_START}1{NOTE_FIELD_SEP}1{NOTE_FIELD_SEP}Note.");
        let page = format!("Body line one\n{note}");
        let result = detect_structure(&[page], &default_options());
        assert!(matches!(&result[0], StructuredContent::Paragraph { text } if text == "Body line one"));
        assert!(matches!(&result[1], StructuredContent::Note { .. }));
    }

    #[test]
    fn ordered_list_item_keeps_its_number() {
        let pages = vec!["12. Twelfth item".to_string()];
        let result = detect_structure(&pages, &default_options());
        assert!(result.iter().any(|c| matches!(c, StructuredContent::ListItem { number: Some(12), .. })));
    }

    #[test]
    fn respects_heading_detection_off() {
        let mut opts = default_options();
        opts.detect_headings = false;
        let pages = vec![format!("\n{}1INTRODUCTION\n\nText here.", HEADING_MARK)];
        let result = detect_structure(&pages, &opts);

        let has_heading = result.iter().any(|c| matches!(c, StructuredContent::Heading { .. }));
        assert!(!has_heading);

        // The heading's real text still surfaces as a paragraph, marker stripped.
        let paragraphs: Vec<_> = result.iter()
            .filter_map(|c| match c {
                StructuredContent::Paragraph { text } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert!(paragraphs.contains(&"INTRODUCTION"));
    }

    #[test]
    fn heading_above_level_threshold_falls_back_to_paragraph() {
        let mut opts = default_options();
        opts.heading_level_threshold = 2;
        let pages = vec![format!("\n{}3Some Subsection\n\nText here.", HEADING_MARK)];
        let result = detect_structure(&pages, &opts);

        let has_heading = result.iter().any(|c| matches!(c, StructuredContent::Heading { .. }));
        assert!(!has_heading);
        let has_paragraph = result.iter().any(|c| matches!(c, StructuredContent::Paragraph { text } if text == "Some Subsection"));
        assert!(has_paragraph);
    }

    #[test]
    fn respects_list_detection_off() {
        let mut opts = default_options();
        opts.list_detection = false;
        let pages = vec!["- item one\n- item two".to_string()];
        let result = detect_structure(&pages, &opts);

        let has_list = result.iter().any(|c| matches!(c, StructuredContent::ListItem { .. }));
        assert!(!has_list);
    }

    #[test]
    fn inserts_page_breaks_between_pages() {
        let pages = vec!["Page one.".to_string(), "Page two.".to_string()];
        let result = detect_structure(&pages, &default_options());

        let has_break = result.iter().any(|c| matches!(c, StructuredContent::PageBreak));
        assert!(has_break);
    }

    #[test]
    fn unmarked_lines_are_never_headings_regardless_of_content() {
        // Heading detection is driven entirely by the font-size marker
        // `text_extractor::mark_headings` writes from real PDF typography, not
        // by guessing from capitalization/shape. A short ALL-CAPS acronym like
        // "CQRS" set at body text size (a list label, not a heading) must not
        // be promoted just because it looks heading-shaped.
        for line in &[
            "A",
            "LB",
            "CQRS",
            "VAGRANTFILE_API_VERSION = \"2\"",
            "$ANSIBLE_VAULT;1.1;AES256",
            "DOWNLOAD_WORDPRESS",
            "ISBN 978-1-78439-829-3",
            "INTRODUCTION",
            "CHAPTER ONE",
            "GETTING STARTED",
            "Part Two",
        ] {
            let pages = vec![format!("\n{}\n\nText.", line)];
            let result = detect_structure(&pages, &default_options());
            let has_heading = result.iter().any(|c| matches!(c, StructuredContent::Heading { .. }));
            assert!(!has_heading, "'{}' has no heading marker, should never be a heading", line);
        }
    }

    #[test]
    fn accepts_real_chapter_headings_regardless_of_casing() {
        // Marker-driven detection trusts the font size, not the text's shape:
        // even lowercase text is a heading when MuPDF's real size data marked it.
        for (level, title) in [(1u8, "INTRODUCTION"), (2, "Chapter One"), (3, "getting started")] {
            let pages = vec![format!("\n{}{}{}\n\nText.", HEADING_MARK, level, title)];
            let result = detect_structure(&pages, &default_options());
            let has_heading = result.iter().any(|c| matches!(c, StructuredContent::Heading { level: l, text } if *l == level && text == title));
            assert!(has_heading, "should detect '{}' as a level-{} heading", title, level);
        }
    }

    // -- footnote detection --

    fn footnote_options() -> StructureOptions {
        StructureOptions {
            detect_headings: true,
            detect_footnotes: true,
            heading_level_threshold: 3,
            paragraph_detection: true,
            list_detection: true,
        }
    }

    #[test]
    fn detects_superscript_footnotes() {
        let pages = vec!["Body text here.\n\n\u{00B9} First footnote.\n\u{00B2} Second footnote.".to_string()];
        let result = detect_structure(&pages, &footnote_options());

        let footnotes: Vec<_> = result.iter()
            .filter_map(|c| match c {
                StructuredContent::Footnote { number, text } => Some((*number, text.as_str())),
                _ => None,
            })
            .collect();
        assert_eq!(footnotes.len(), 2);
        assert_eq!(footnotes[0], (1, "First footnote."));
        assert_eq!(footnotes[1], (2, "Second footnote."));
    }

    #[test]
    fn detects_footnotes_with_separator() {
        let pages = vec!["Body text.\n\n___________\n1 Footnote one.\n2 Footnote two.".to_string()];
        let result = detect_structure(&pages, &footnote_options());

        let footnotes: Vec<_> = result.iter()
            .filter_map(|c| match c {
                StructuredContent::Footnote { number, .. } => Some(*number),
                _ => None,
            })
            .collect();
        assert_eq!(footnotes, vec![1, 2]);
    }

    #[test]
    fn excludes_footnotes_from_body_text() {
        let pages = vec!["Body paragraph.\n\n\u{00B9} Footnote text here.".to_string()];
        let result = detect_structure(&pages, &footnote_options());

        let paragraphs: Vec<_> = result.iter()
            .filter_map(|c| match c {
                StructuredContent::Paragraph { text } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert!(paragraphs.iter().all(|p| !p.contains("Footnote text")));
    }

    #[test]
    fn respects_detect_footnotes_off() {
        let pages = vec!["Body text.\n\n\u{00B9} Footnote text.".to_string()];
        let result = detect_structure(&pages, &default_options());

        let has_footnote = result.iter().any(|c| matches!(c, StructuredContent::Footnote { .. }));
        assert!(!has_footnote);
    }

    #[test]
    fn detects_multi_line_footnotes() {
        let pages = vec!["Body.\n\n\u{00B9} First footnote that spans\nmultiple lines here.\n\u{00B2} Second.".to_string()];
        let result = detect_structure(&pages, &footnote_options());

        let footnotes: Vec<_> = result.iter()
            .filter_map(|c| match c {
                StructuredContent::Footnote { number, text } => Some((*number, text.clone())),
                _ => None,
            })
            .collect();
        assert_eq!(footnotes.len(), 2);
        assert!(footnotes[0].1.contains("spans"));
        assert!(footnotes[0].1.contains("multiple lines"));
    }

    #[test]
    fn no_false_positive_without_footnotes() {
        let pages = vec!["Just a regular page.\n\nWith paragraphs.\n\nNothing special.".to_string()];
        let result = detect_structure(&pages, &footnote_options());

        let has_footnote = result.iter().any(|c| matches!(c, StructuredContent::Footnote { .. }));
        assert!(!has_footnote);
    }

    #[test]
    fn collects_footnotes_from_multiple_pages() {
        let pages = vec![
            "Page one text.\n\n\u{00B9} First page footnote.".to_string(),
            "Page two text.\n\n\u{00B2} Second page footnote.".to_string(),
        ];
        let result = detect_structure(&pages, &footnote_options());

        let footnotes: Vec<_> = result.iter()
            .filter_map(|c| match c {
                StructuredContent::Footnote { number, .. } => Some(*number),
                _ => None,
            })
            .collect();
        assert_eq!(footnotes, vec![1, 2]);
    }

    #[test]
    fn separator_with_regular_digit_footnotes() {
        let pages = vec!["Body text.\n\n----------\n1. First note.\n2. Second note.".to_string()];
        let result = detect_structure(&pages, &footnote_options());

        let footnotes: Vec<_> = result.iter()
            .filter_map(|c| match c {
                StructuredContent::Footnote { number, text } => Some((*number, text.as_str())),
                _ => None,
            })
            .collect();
        assert_eq!(footnotes.len(), 2);
        assert_eq!(footnotes[0], (1, "First note."));
    }

    #[test]
    fn no_false_positive_for_ordinals() {
        assert!(parse_superscript_start("\u{00B2}nd edition").is_none());
    }

    // -- code block detection --

    #[test]
    fn extracts_marked_code_block() {
        let page = format!(
            "Some prose before.\n\n{}\nclass Batch:\n    def __init__(self):\n        pass\n{}\n\nMore prose after.",
            CODE_BLOCK_START, CODE_BLOCK_END
        );
        let result = detect_structure(&[page], &default_options());

        let code_blocks: Vec<_> = result.iter()
            .filter_map(|c| match c {
                StructuredContent::CodeBlock { text } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(code_blocks, vec!["class Batch:\n    def __init__(self):\n        pass"]);
    }

    #[test]
    fn adjacent_code_blocks_separated_by_blank_lines_merge() {
        let (s, e) = (CODE_BLOCK_START, CODE_BLOCK_END);
        let page = format!(
            "{s}\nimport requests\n{e}\n\n{s}\nparams = dict()\nparsed = get()\n{e}\n\n\
             {s}\nfor r in results:\n    print(r)\n{e}\n\nDo a search\n\n{s}\nimport json\n{e}"
        );
        let result = detect_structure(&[page], &default_options());

        let code_blocks: Vec<_> = result.iter()
            .filter_map(|c| match c {
                StructuredContent::CodeBlock { text } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(
            code_blocks,
            vec![
                "import requests\n\nparams = dict()\nparsed = get()\n\nfor r in results:\n    print(r)",
                "import json",
            ]
        );
    }

    #[test]
    fn code_block_lines_are_not_parsed_as_paragraphs_or_headings() {
        let page = format!(
            "{}\nCQRS\ndef allocate(line):\n    pass\n{}",
            CODE_BLOCK_START, CODE_BLOCK_END
        );
        let result = detect_structure(&[page], &default_options());

        let has_heading = result.iter().any(|c| matches!(c, StructuredContent::Heading { .. }));
        let has_paragraph = result.iter().any(|c| matches!(c, StructuredContent::Paragraph { .. }));
        assert!(!has_heading);
        assert!(!has_paragraph);
    }

    #[test]
    fn text_around_code_block_is_still_parsed_normally() {
        let page = format!(
            "Intro paragraph.\n\n{}\nprint(\"hi\")\n{}\n\nClosing paragraph.",
            CODE_BLOCK_START, CODE_BLOCK_END
        );
        let result = detect_structure(&[page], &default_options());

        let paragraphs: Vec<_> = result.iter()
            .filter_map(|c| match c {
                StructuredContent::Paragraph { text } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(paragraphs, vec!["Intro paragraph.", "Closing paragraph."]);
    }

    #[test]
    fn image_line_becomes_image_between_paragraphs() {
        use crate::conversion::text_extractor::image_mark;
        let mark = image_mark("images/img_p5_0_1_400.jpg", None, "Image from page 5");
        let pages = vec![format!(
            "Text before the figure.\n{mark}\n\nFigure P-2. Layered architecture\n\nText after."
        )];
        let result = detect_structure(&pages, &default_options());
        let kinds: Vec<String> = result
            .iter()
            .filter_map(|c| match c {
                StructuredContent::Paragraph { text } => Some(format!("P:{text}")),
                StructuredContent::Image { resource_path, alt, display_width_pct } => {
                    assert_eq!(alt, "Image from page 5");
                    assert_eq!(*display_width_pct, None);
                    Some(format!("I:{resource_path}"))
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            kinds,
            vec![
                "P:Text before the figure.",
                "I:images/img_p5_0_1_400.jpg",
                "P:Figure P-2. Layered architecture",
                "P:Text after.",
            ]
        );
    }
}

