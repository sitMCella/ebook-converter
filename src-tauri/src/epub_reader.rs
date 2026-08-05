use serde::Serialize;
use std::io::Read;
use zip::ZipArchive;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpubPreviewData {
    pub chapters: Vec<ChapterPreview>,
    pub cover_image: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterPreview {
    pub title: String,
    pub html: String,
}

#[tauri::command]
pub fn read_epub_preview(path: String) -> Result<EpubPreviewData, String> {
    let file = std::fs::File::open(&path)
        .map_err(|e| format!("Failed to open EPUB: {}", e))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("Failed to read EPUB archive: {}", e))?;

    let spine_entries = parse_spine(&mut archive)?;

    let mut chapters = Vec::new();
    for entry in &spine_entries {
        let html = read_entry(&mut archive, entry)?;
        let body = extract_body(&html);
        let title = extract_title(&html);
        if entry.contains("cover") && body.trim().is_empty() {
            continue;
        }
        chapters.push(ChapterPreview {
            title: title.unwrap_or_else(|| format!("Chapter {}", chapters.len() + 1)),
            html: body,
        });
    }

    let cover_image = extract_cover_image_data(&mut archive);

    Ok(EpubPreviewData {
        chapters,
        cover_image,
    })
}

fn parse_spine(archive: &mut ZipArchive<std::fs::File>) -> Result<Vec<String>, String> {
    let opf_path = find_opf_path(archive)?;
    let opf_content = read_entry(archive, &opf_path)?;

    let mut manifest: Vec<(String, String)> = Vec::new();
    let opf_dir = opf_path.rsplit_once('/').map(|(d, _)| d).unwrap_or("");

    for line in opf_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<item ") || trimmed.starts_with("<item>") {
            if let (Some(id), Some(href)) = (extract_attr(trimmed, "id"), extract_attr(trimmed, "href")) {
                let full_path = if opf_dir.is_empty() {
                    href.clone()
                } else {
                    format!("{}/{}", opf_dir, href)
                };
                manifest.push((id, full_path));
            }
        }
    }

    let mut spine_ids: Vec<String> = Vec::new();
    let mut in_spine = false;
    for line in opf_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<spine") {
            in_spine = true;
        }
        if in_spine {
            if trimmed.starts_with("<itemref ") {
                if let Some(idref) = extract_attr(trimmed, "idref") {
                    spine_ids.push(idref);
                }
            }
            if trimmed.contains("</spine>") || (trimmed.starts_with("<spine") && trimmed.ends_with("/>")) {
                break;
            }
        }
    }

    let entries: Vec<String> = spine_ids
        .iter()
        .filter_map(|id| {
            manifest.iter().find(|(mid, _)| mid == id).map(|(_, path)| path.clone())
        })
        .collect();

    if entries.is_empty() {
        let fallback: Vec<String> = manifest
            .iter()
            .filter(|(_, path)| path.ends_with(".xhtml") || path.ends_with(".html"))
            .map(|(_, path)| path.clone())
            .collect();
        Ok(fallback)
    } else {
        Ok(entries)
    }
}

fn find_opf_path(archive: &mut ZipArchive<std::fs::File>) -> Result<String, String> {
    if let Ok(container_content) = read_entry(archive, "META-INF/container.xml") {
        for line in container_content.lines() {
            let trimmed = line.trim();
            if trimmed.contains("rootfile") {
                if let Some(path) = extract_attr(trimmed, "full-path") {
                    return Ok(path);
                }
            }
        }
    }

    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index(i) {
            let name = entry.name().to_string();
            if name.ends_with(".opf") {
                return Ok(name);
            }
        }
    }

    Err("Could not find OPF file in EPUB".to_string())
}

fn read_entry(archive: &mut ZipArchive<std::fs::File>, name: &str) -> Result<String, String> {
    let mut file = archive.by_name(name)
        .map_err(|e| format!("Entry '{}' not found: {}", name, e))?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| format!("Failed to read '{}': {}", name, e))?;
    Ok(content)
}

fn extract_body(html: &str) -> String {
    let lower = html.to_lowercase();
    let body_start = lower.find("<body");
    let body_end = lower.find("</body>");

    match (body_start, body_end) {
        (Some(start), Some(end)) => {
            let after_tag = html[start..].find('>').map(|i| start + i + 1).unwrap_or(start);
            html[after_tag..end].trim().to_string()
        }
        _ => html.to_string(),
    }
}

fn extract_title(html: &str) -> Option<String> {
    let lower = html.to_lowercase();

    for tag in ["<h1>", "<h1 ", "<h2>", "<h2 "] {
        if let Some(start) = lower.find(tag) {
            let after_tag = html[start..].find('>')? + start + 1;
            let close_tag = if tag.starts_with("<h1") { "</h1>" } else { "</h2>" };
            let end = lower[after_tag..].find(close_tag)? + after_tag;
            let title = strip_html_tags(&html[after_tag..end]).trim().to_string();
            if !title.is_empty() {
                return Some(title);
            }
        }
    }

    None
}

fn strip_html_tags(s: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(c);
        }
    }
    result
}

fn extract_attr(tag: &str, attr_name: &str) -> Option<String> {
    let patterns = [
        format!("{}=\"", attr_name),
        format!("{}='", attr_name),
    ];

    for pattern in &patterns {
        if let Some(start) = tag.find(pattern.as_str()) {
            let value_start = start + pattern.len();
            let quote = tag.as_bytes()[value_start - 1] as char;
            if let Some(end) = tag[value_start..].find(quote) {
                return Some(tag[value_start..value_start + end].to_string());
            }
        }
    }
    None
}

fn extract_cover_image_data(archive: &mut ZipArchive<std::fs::File>) -> Option<String> {
    let image_names: Vec<String> = (0..archive.len())
        .filter_map(|i| {
            let entry = archive.by_index(i).ok()?;
            let name = entry.name().to_string();
            if name.contains("cover") && is_image_ext(&name) {
                Some(name)
            } else {
                None
            }
        })
        .collect();

    let name = image_names.first()?;

    let mut file = archive.by_name(name).ok()?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).ok()?;

    let mime = if name.ends_with(".png") {
        "image/png"
    } else if name.ends_with(".webp") {
        "image/webp"
    } else {
        "image/jpeg"
    };

    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
    Some(format!("data:{};base64,{}", mime, b64))
}

fn is_image_ext(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".jpg") || lower.ends_with(".jpeg") || lower.ends_with(".png") || lower.ends_with(".webp")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_body_returns_body_content() {
        let html = r#"<html><head></head><body><p>Hello</p></body></html>"#;
        assert_eq!(extract_body(html), "<p>Hello</p>");
    }

    #[test]
    fn extract_body_handles_no_body_tag() {
        let html = "<p>No body tag</p>";
        assert_eq!(extract_body(html), html);
    }

    #[test]
    fn extract_title_from_h1() {
        let html = r#"<body><h1>My Chapter</h1><p>Text</p></body>"#;
        assert_eq!(extract_title(html), Some("My Chapter".to_string()));
    }

    #[test]
    fn extract_title_from_h2_when_no_h1() {
        let html = r#"<body><h2>Section Title</h2></body>"#;
        assert_eq!(extract_title(html), Some("Section Title".to_string()));
    }

    #[test]
    fn extract_title_returns_none_when_no_heading() {
        let html = r#"<body><p>Just text</p></body>"#;
        assert_eq!(extract_title(html), None);
    }

    #[test]
    fn extract_attr_finds_double_quoted() {
        let tag = r#"<item id="ch1" href="chapter1.xhtml" />"#;
        assert_eq!(extract_attr(tag, "id"), Some("ch1".to_string()));
        assert_eq!(extract_attr(tag, "href"), Some("chapter1.xhtml".to_string()));
    }

    #[test]
    fn extract_attr_finds_single_quoted() {
        let tag = "<item id='ch1' href='chapter1.xhtml' />";
        assert_eq!(extract_attr(tag, "id"), Some("ch1".to_string()));
    }

    #[test]
    fn extract_attr_returns_none_for_missing() {
        let tag = r#"<item id="ch1" />"#;
        assert_eq!(extract_attr(tag, "href"), None);
    }

    #[test]
    fn strip_html_tags_removes_tags() {
        assert_eq!(strip_html_tags("<b>bold</b> text"), "bold text");
        assert_eq!(strip_html_tags("no tags"), "no tags");
    }

    #[test]
    fn is_image_ext_matches_images() {
        assert!(is_image_ext("cover.jpg"));
        assert!(is_image_ext("img/COVER.PNG"));
        assert!(is_image_ext("photo.webp"));
        assert!(!is_image_ext("chapter.xhtml"));
    }
}
