use super::structure_detector::StructuredContent;
use super::PageHandlingOptions;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Chapter {
    pub title: String,
    pub content: Vec<StructuredContent>,
    pub images: Vec<String>,
}

pub fn split_chapters(
    content: Vec<StructuredContent>,
    options: &PageHandlingOptions,
) -> Vec<Chapter> {
    match options.split_chapters_by.as_str() {
        "heading1" => split_by_heading(content, 1),
        "heading2" => split_by_heading(content, 2),
        "pageBreak" => split_by_page_break(content),
        _ => vec![single_chapter(content)],
    }
}

fn split_by_heading(content: Vec<StructuredContent>, max_level: u8) -> Vec<Chapter> {
    let mut chapters: Vec<Chapter> = Vec::new();
    let mut current_content: Vec<StructuredContent> = Vec::new();
    let mut current_title: Option<String> = None;

    for item in content {
        if let StructuredContent::Heading { level, ref text } = item {
            if level <= max_level {
                if !current_content.is_empty() || current_title.is_some() {
                    chapters.push(make_chapter(current_title.take(), current_content));
                    current_content = Vec::new();
                }
                current_title = Some(text.clone());
            }
        }
        current_content.push(item);
    }

    if !current_content.is_empty() || current_title.is_some() {
        chapters.push(make_chapter(current_title, current_content));
    }

    if chapters.is_empty() {
        return vec![Chapter {
            title: "Chapter 1".to_string(),
            content: Vec::new(),
            images: Vec::new(),
        }];
    }

    assign_default_titles(&mut chapters);
    chapters
}

fn split_by_page_break(content: Vec<StructuredContent>) -> Vec<Chapter> {
    let mut chapters: Vec<Chapter> = Vec::new();
    let mut current_content: Vec<StructuredContent> = Vec::new();

    for item in content {
        if matches!(item, StructuredContent::PageBreak) {
            if !current_content.is_empty() {
                chapters.push(make_chapter(None, current_content));
                current_content = Vec::new();
            }
            continue;
        }
        current_content.push(item);
    }

    if !current_content.is_empty() {
        chapters.push(make_chapter(None, current_content));
    }

    if chapters.is_empty() {
        return vec![Chapter {
            title: "Chapter 1".to_string(),
            content: Vec::new(),
            images: Vec::new(),
        }];
    }

    assign_default_titles(&mut chapters);
    chapters
}

fn single_chapter(content: Vec<StructuredContent>) -> Chapter {
    let title = content.iter().find_map(|item| {
        if let StructuredContent::Heading { text, .. } = item {
            Some(text.clone())
        } else {
            None
        }
    });

    Chapter {
        title: title.unwrap_or_else(|| "Chapter 1".to_string()),
        content,
        images: Vec::new(),
    }
}

fn make_chapter(title: Option<String>, content: Vec<StructuredContent>) -> Chapter {
    Chapter {
        title: title.unwrap_or_default(),
        content,
        images: Vec::new(),
    }
}

fn assign_default_titles(chapters: &mut [Chapter]) {
    for (i, chapter) in chapters.iter_mut().enumerate() {
        if chapter.title.is_empty() {
            chapter.title = format!("Chapter {}", i + 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn heading(level: u8, text: &str) -> StructuredContent {
        StructuredContent::Heading { level, text: text.to_string() }
    }

    fn para(text: &str) -> StructuredContent {
        StructuredContent::Paragraph { text: text.to_string() }
    }

    fn opts(split_by: &str) -> PageHandlingOptions {
        PageHandlingOptions {
            skip_blank_pages: true,
            page_range: "all".to_string(),
            page_range_from: None,
            page_range_to: None,
            split_chapters_by: split_by.to_string(),
        }
    }

    #[test]
    fn split_by_heading1() {
        let content = vec![
            heading(1, "Chapter One"),
            para("Text in chapter one."),
            heading(1, "Chapter Two"),
            para("Text in chapter two."),
        ];
        let chapters = split_chapters(content, &opts("heading1"));
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].title, "Chapter One");
        assert_eq!(chapters[1].title, "Chapter Two");
    }

    #[test]
    fn split_by_heading2_includes_both_levels() {
        let content = vec![
            heading(1, "Part One"),
            para("Intro."),
            heading(2, "Section A"),
            para("Section text."),
        ];
        let chapters = split_chapters(content, &opts("heading2"));
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].title, "Part One");
        assert_eq!(chapters[1].title, "Section A");
    }

    #[test]
    fn split_none_single_chapter() {
        let content = vec![
            heading(1, "Title"),
            para("Some text."),
            heading(1, "Another Title"),
            para("More text."),
        ];
        let chapters = split_chapters(content, &opts("none"));
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0].title, "Title");
    }

    #[test]
    fn split_by_page_break() {
        let content = vec![
            para("Page one text."),
            StructuredContent::PageBreak,
            para("Page two text."),
        ];
        let chapters = split_chapters(content, &opts("pageBreak"));
        assert_eq!(chapters.len(), 2);
    }

    #[test]
    fn default_title_when_no_heading() {
        let content = vec![para("Just some text.")];
        let chapters = split_chapters(content, &opts("heading1"));
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0].title, "Chapter 1");
    }

    #[test]
    fn fallback_to_single_chapter_when_no_headings_found() {
        let content = vec![para("Line one."), para("Line two.")];
        let chapters = split_chapters(content, &opts("heading1"));
        assert_eq!(chapters.len(), 1);
    }
}
