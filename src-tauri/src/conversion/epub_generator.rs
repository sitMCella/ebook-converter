use super::chapter_splitter::Chapter;
use super::css;
use super::image_extractor::ExtractedImage;
use super::structure_detector::StructuredContent;
use super::{ConversionOptions, ConversionResult};
use crate::pdf::PdfMetadata;
use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipLibrary};

pub fn generate_epub(
    chapters: &[Chapter],
    images: &[ExtractedImage],
    metadata: &PdfMetadata,
    options: &ConversionOptions,
    output_path: &str,
) -> Result<ConversionResult, String> {
    let mut builder = EpubBuilder::new(ZipLibrary::new().map_err(|e| format!("Failed to create ZIP: {}", e))?)
        .map_err(|e| format!("Failed to create EPUB builder: {}", e))?;

    if options.output.epub_version == "epub2" {
        builder.epub_version(epub_builder::EpubVersion::V20);
    } else {
        builder.epub_version(epub_builder::EpubVersion::V30);
    }

    let title = metadata
        .title
        .as_deref()
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| {
            std::path::Path::new(output_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
        });

    builder
        .metadata("title", title)
        .map_err(|e| format!("Failed to set title: {}", e))?;

    if let Some(ref author) = metadata.author {
        if !author.is_empty() {
            builder
                .metadata("author", author)
                .map_err(|e| format!("Failed to set author: {}", e))?;
        }
    }

    let stylesheet = css::generate_css(&options.output);
    builder
        .stylesheet(stylesheet.as_bytes())
        .map_err(|e| format!("Failed to add stylesheet: {}", e))?;

    for image in images {
        let ext = match image.mime_type.as_str() {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/webp" => "webp",
            _ => "bin",
        };
        let resource_path = format!("images/{}_{}.{}", image.id, image.width, ext);
        builder
            .add_resource(&resource_path, image.data.as_slice(), &image.mime_type)
            .map_err(|e| format!("Failed to add image {}: {}", image.id, e))?;
    }

    for (i, chapter) in chapters.iter().enumerate() {
        let xhtml = chapter_to_xhtml(chapter, images);
        let filename = format!("chapter{}.xhtml", i + 1);

        let mut content = EpubContent::new(&filename, xhtml.as_bytes())
            .title(&chapter.title);

        if i == 0 {
            content = content.reftype(ReferenceType::Text);
        }

        builder
            .add_content(content)
            .map_err(|e| format!("Failed to add chapter {}: {}", i + 1, e))?;
    }

    let mut output = Vec::new();
    builder
        .generate(&mut output)
        .map_err(|e| format!("Failed to generate EPUB: {}", e))?;

    std::fs::write(output_path, &output)
        .map_err(|e| format!("Failed to write EPUB file: {}", e))?;

    Ok(ConversionResult {
        output_path: output_path.to_string(),
        chapters: chapters.len(),
        images: images.len(),
        file_size: output.len() as u64,
    })
}

fn chapter_to_xhtml(chapter: &Chapter, _images: &[ExtractedImage]) -> String {
    let mut html = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title></title><link rel="stylesheet" type="text/css" href="stylesheet.css" /></head>
<body>
"#,
    );

    content_to_xhtml(&chapter.content, &mut html);

    html.push_str("</body>\n</html>");
    html
}

fn content_to_xhtml(content: &[StructuredContent], html: &mut String) {
    let mut in_ul = false;
    let mut in_ol = false;

    for item in content {
        match item {
            StructuredContent::Heading { level, text } => {
                close_lists(html, &mut in_ul, &mut in_ol);
                let tag = format!("h{}", level.min(&6));
                html.push_str(&format!("<{}>{}</{}>\n", tag, escape_xml(text), tag));
            }
            StructuredContent::Paragraph { text } => {
                close_lists(html, &mut in_ul, &mut in_ol);
                html.push_str(&format!("<p>{}</p>\n", escape_xml(text)));
            }
            StructuredContent::ListItem { text, ordered } => {
                if *ordered {
                    if in_ul {
                        html.push_str("</ul>\n");
                        in_ul = false;
                    }
                    if !in_ol {
                        html.push_str("<ol>\n");
                        in_ol = true;
                    }
                } else {
                    if in_ol {
                        html.push_str("</ol>\n");
                        in_ol = false;
                    }
                    if !in_ul {
                        html.push_str("<ul>\n");
                        in_ul = true;
                    }
                }
                html.push_str(&format!("<li>{}</li>\n", escape_xml(text)));
            }
            StructuredContent::BlankLine | StructuredContent::PageBreak => {
                close_lists(html, &mut in_ul, &mut in_ol);
            }
        }
    }

    close_lists(html, &mut in_ul, &mut in_ol);
}

fn close_lists(html: &mut String, in_ul: &mut bool, in_ol: &mut bool) {
    if *in_ul {
        html.push_str("</ul>\n");
        *in_ul = false;
    }
    if *in_ol {
        html.push_str("</ol>\n");
        *in_ol = false;
    }
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
