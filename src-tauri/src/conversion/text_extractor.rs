use super::PageHandlingOptions;
use mupdf::TextExtractOptions;

pub fn extract_text(
    path: &str,
    options: &PageHandlingOptions,
) -> Result<Vec<String>, String> {
    let doc = mupdf::Document::open(path)
        .map_err(|e| format!("Failed to open PDF: {}", e))?;

    let page_count = doc
        .page_count()
        .map_err(|e| format!("Failed to get page count: {}", e))?;

    let extract_opts = TextExtractOptions::default();
    let mut pages = Vec::with_capacity(page_count as usize);

    for i in 0..page_count {
        let page = doc
            .load_page(i)
            .map_err(|e| format!("Failed to load page {}: {}", i + 1, e))?;
        let text = page
            .text(extract_opts)
            .map_err(|e| format!("Failed to extract text from page {}: {}", i + 1, e))?;
        pages.push(text);
    }

    if options.page_range == "custom" {
        let from = options.page_range_from.unwrap_or(1).max(1) as usize;
        let to = options
            .page_range_to
            .map(|t| t as usize)
            .unwrap_or(pages.len())
            .min(pages.len());

        if from > to || from > pages.len() {
            return Ok(Vec::new());
        }

        pages = pages[(from - 1)..to].to_vec();
    }

    if options.remove_page_numbers {
        strip_page_numbers(&mut pages);
    }

    if options.skip_blank_pages {
        pages.retain(|p| !p.trim().is_empty());
    }

    Ok(pages)
}

fn strip_page_numbers(pages: &mut Vec<String>) {
    for page in pages.iter_mut() {
        let lines: Vec<&str> = page.lines().collect();
        if lines.is_empty() {
            continue;
        }

        let first_non_empty = lines.iter().position(|l| !l.trim().is_empty());
        let last_non_empty = lines.iter().rposition(|l| !l.trim().is_empty());

        let mut remove_indices = Vec::new();

        if let Some(idx) = first_non_empty {
            if is_page_number(lines[idx].trim()) {
                remove_indices.push(idx);
            }
        }

        if let Some(idx) = last_non_empty {
            if !remove_indices.contains(&idx) && is_page_number(lines[idx].trim()) {
                remove_indices.push(idx);
            }
        }

        if !remove_indices.is_empty() {
            let new_lines: Vec<&str> = lines
                .iter()
                .enumerate()
                .filter(|(i, _)| !remove_indices.contains(i))
                .map(|(_, l)| *l)
                .collect();
            *page = new_lines.join("\n");
        }
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

    // Pipe separator or pipe-prefixed footers: "|", "| v", "| 42", "| xii"
    if let Some(rest) = line.strip_prefix('|') {
        let rest = rest.trim();
        if rest.is_empty()
            || rest.chars().all(|c| c.is_ascii_digit())
            || is_roman_numeral(rest)
        {
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
    fn is_page_number_pipe_standalone() {
        assert!(is_page_number("|"));
    }

    #[test]
    fn is_page_number_pipe_rejects_text() {
        assert!(!is_page_number("| See also"));
        assert!(!is_page_number("| Chapter One"));
    }

    #[test]
    fn strip_removes_pipe_prefixed_footer() {
        let mut pages = vec!["Table of Contents text.\n\n| v".to_string()];
        strip_page_numbers(&mut pages);
        assert_eq!(pages[0], "Table of Contents text.\n");
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
}
