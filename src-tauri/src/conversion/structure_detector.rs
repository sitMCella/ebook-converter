use super::StructureOptions;

#[derive(Debug, Clone)]
pub enum StructuredContent {
    Heading { level: u8, text: String },
    Paragraph { text: String },
    ListItem { text: String, ordered: bool },
    Image { resource_path: String, alt: String },
    BlankLine,
    PageBreak,
}

pub fn detect_structure(
    pages: &[String],
    options: &StructureOptions,
) -> Vec<StructuredContent> {
    let mut result = Vec::new();

    for (i, page) in pages.iter().enumerate() {
        let lines: Vec<&str> = page.lines().collect();
        let mut line_idx = 0;

        while line_idx < lines.len() {
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
}
