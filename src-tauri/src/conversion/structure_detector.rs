use super::StructureOptions;

#[derive(Debug, Clone)]
pub enum StructuredContent {
    Heading { level: u8, text: String },
    Paragraph { text: String },
    ListItem { text: String, ordered: bool },
    Image { resource_path: String, alt: String, display_width_pct: Option<u32> },
    Footnote { number: u32, text: String },
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

            if line.is_empty() {
                if options.paragraph_detection {
                    result.push(StructuredContent::BlankLine);
                }
                line_idx += 1;
                continue;
            }

            if options.list_detection {
                if let Some(item) = detect_list_item(line) {
                    result.push(item);
                    line_idx += 1;
                    continue;
                }
            }

            if options.detect_headings && is_heading(line, &lines, line_idx, options.heading_level_threshold) {
                let level = detect_heading_level(line);
                if level <= options.heading_level_threshold {
                    result.push(StructuredContent::Heading {
                        level,
                        text: line.to_string(),
                    });
                    line_idx += 1;
                    continue;
                }
            }

            if options.paragraph_detection {
                let mut paragraph_text = String::from(line);
                line_idx += 1;
                while line_idx < lines.len() {
                    let next = lines[line_idx].trim();
                    if next.is_empty() {
                        break;
                    }
                    if options.detect_headings && is_heading(next, &lines, line_idx, options.heading_level_threshold) {
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

    result
}

fn is_heading(line: &str, lines: &[&str], idx: usize, _max_level: u8) -> bool {
    if line.len() > 80 || line.ends_with('.') || line.ends_with(',') {
        return false;
    }

    if line.trim().is_empty() {
        return false;
    }

    let has_blank_before = idx == 0 || (idx > 0 && lines[idx - 1].trim().is_empty());

    if !has_blank_before {
        return false;
    }

    if looks_like_noise(line) {
        return false;
    }

    is_uppercase(line) || is_title_case(line)
}

fn looks_like_noise(line: &str) -> bool {
    let alpha_count = line.chars().filter(|c| c.is_alphabetic()).count();
    if alpha_count <= 1 {
        return true;
    }

    if line.contains('=') || line.contains(';') || line.contains('$')
        || line.contains('{') || line.contains('}') || line.contains('_')
    {
        return true;
    }

    let upper = line.to_uppercase();
    if upper.starts_with("ISBN") {
        return true;
    }

    let word_count = line.split_whitespace().count();
    if word_count == 1 && alpha_count <= 3 {
        return true;
    }

    false
}

fn detect_heading_level(line: &str) -> u8 {
    if is_uppercase(line) {
        1
    } else {
        2
    }
}

fn is_uppercase(text: &str) -> bool {
    let alpha_chars: String = text.chars().filter(|c| c.is_alphabetic()).collect();
    !alpha_chars.is_empty() && alpha_chars == alpha_chars.to_uppercase()
}

fn is_title_case(text: &str) -> bool {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return false;
    }

    let skip_words = ["a", "an", "the", "and", "but", "or", "for", "nor",
                       "at", "by", "in", "of", "on", "to", "up", "is", "it"];

    let significant_words: Vec<&&str> = words
        .iter()
        .filter(|w| !skip_words.contains(&w.to_lowercase().as_str()))
        .collect();

    if significant_words.is_empty() {
        return false;
    }

    significant_words.iter().all(|w| {
        w.chars()
            .next()
            .map_or(false, |c| c.is_uppercase() || !c.is_alphabetic())
    })
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
                });
            }
        }
    }

    if let Some(rest) = strip_numbered_prefix(trimmed) {
        if !rest.is_empty() {
            return Some(StructuredContent::ListItem {
                text: rest.to_string(),
                ordered: true,
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
    fn detects_uppercase_heading() {
        let pages = vec!["\nINTRODUCTION\n\nSome paragraph text here.".to_string()];
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
    fn respects_heading_detection_off() {
        let mut opts = default_options();
        opts.detect_headings = false;
        let pages = vec!["\nINTRODUCTION\n\nText here.".to_string()];
        let result = detect_structure(&pages, &opts);

        let has_heading = result.iter().any(|c| matches!(c, StructuredContent::Heading { .. }));
        assert!(!has_heading);
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
    fn rejects_single_letter_as_heading() {
        let pages = vec!["\nA\n\nSome text.".to_string()];
        let result = detect_structure(&pages, &default_options());
        let has_heading = result.iter().any(|c| matches!(c, StructuredContent::Heading { .. }));
        assert!(!has_heading);
    }

    #[test]
    fn rejects_short_code_as_heading() {
        let pages = vec!["\nLB\n\nSome text.".to_string()];
        let result = detect_structure(&pages, &default_options());
        let has_heading = result.iter().any(|c| matches!(c, StructuredContent::Heading { .. }));
        assert!(!has_heading);
    }

    #[test]
    fn rejects_lines_with_code_characters() {
        for line in &[
            "VAGRANTFILE_API_VERSION = \"2\"",
            "$ANSIBLE_VAULT;1.1;AES256",
            "DOWNLOAD_WORDPRESS",
        ] {
            let pages = vec![format!("\n{}\n\nText.", line)];
            let result = detect_structure(&pages, &default_options());
            let has_heading = result.iter().any(|c| matches!(c, StructuredContent::Heading { .. }));
            assert!(!has_heading, "should not detect '{}' as heading", line);
        }
    }

    #[test]
    fn rejects_isbn_as_heading() {
        let pages = vec!["\nISBN 978-1-78439-829-3\n\nText.".to_string()];
        let result = detect_structure(&pages, &default_options());
        let has_heading = result.iter().any(|c| matches!(c, StructuredContent::Heading { .. }));
        assert!(!has_heading);
    }

    #[test]
    fn accepts_real_chapter_headings() {
        for title in &["INTRODUCTION", "CHAPTER ONE", "GETTING STARTED", "Part Two"] {
            let pages = vec![format!("\n{}\n\nText.", title)];
            let result = detect_structure(&pages, &default_options());
            let has_heading = result.iter().any(|c| matches!(c, StructuredContent::Heading { .. }));
            assert!(has_heading, "should detect '{}' as heading", title);
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
}
