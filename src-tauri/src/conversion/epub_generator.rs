use super::css;
use super::image_extractor::ExtractedImage;
use super::structure_detector::StructuredContent;
use super::{ConversionOptions, ConversionResult};
use crate::pdf::PdfMetadata;
use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipLibrary};

pub fn generate_epub(
    content: &[StructuredContent],
    images: &[ExtractedImage],
    cover_image: Option<&ExtractedImage>,
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

    let has_cover = cover_image.is_some();
    if let Some(cover) = cover_image {
        let cover_ext = match cover.mime_type.as_str() {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/webp" => "webp",
            _ => "jpg",
        };
        let cover_resource_path = format!("images/cover.{}", cover_ext);

        builder
            .add_resource(&cover_resource_path, cover.data.as_slice(), &cover.mime_type)
            .map_err(|e| format!("Failed to add cover image: {}", e))?;

        let cover_xhtml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title>Cover</title><style>body {{ margin: 0; padding: 0; text-align: center; }} img {{ max-width: 100%; max-height: 100%; }}</style></head>
<body>
<div><img src="{}" alt="Cover" /></div>
</body>
</html>"#,
            cover_resource_path
        );

        let cover_content = EpubContent::new("cover.xhtml", cover_xhtml.as_bytes())
            .title("Cover")
            .reftype(ReferenceType::Cover);
        builder
            .add_content(cover_content)
            .map_err(|e| format!("Failed to add cover page: {}", e))?;
    }

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

    let xhtml = content_to_xhtml(content);
    let epub_content = EpubContent::new("content.xhtml", xhtml.as_bytes())
        .title(title)
        .reftype(ReferenceType::Text);
    builder
        .add_content(epub_content)
        .map_err(|e| format!("Failed to add content: {}", e))?;

    let mut output = Vec::new();
    builder
        .generate(&mut output)
        .map_err(|e| format!("Failed to generate EPUB: {}", e))?;

    std::fs::write(output_path, &output)
        .map_err(|e| format!("Failed to write EPUB file: {}", e))?;

    Ok(ConversionResult {
        output_path: output_path.to_string(),
        images: images.len(),
        file_size: output.len() as u64,
        has_cover,
    })
}

fn content_to_xhtml(content: &[StructuredContent]) -> String {
    let mut html = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title></title><link rel="stylesheet" type="text/css" href="stylesheet.css" /></head>
<body>
"#,
    );

    let mut in_ul = false;
    let mut in_ol = false;

    for item in content {
        match item {
            StructuredContent::Heading { level, text } => {
                close_lists(&mut html, &mut in_ul, &mut in_ol);
                let tag = format!("h{}", level.min(&6));
                html.push_str(&format!("<{}>{}</{}>\n", tag, escape_xml(text), tag));
            }
            StructuredContent::Paragraph { text } => {
                close_lists(&mut html, &mut in_ul, &mut in_ol);
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
                close_lists(&mut html, &mut in_ul, &mut in_ol);
            }
        }
    }

    close_lists(&mut html, &mut in_ul, &mut in_ol);

    html.push_str("</body>\n</html>");
    html
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversion::{ImageOptions, PageHandlingOptions, StructureOptions, OutputOptions};

    fn test_options() -> ConversionOptions {
        ConversionOptions {
            structure: StructureOptions {
                detect_headings: true,
                detect_footnotes: false,
                heading_level_threshold: 3,
                paragraph_detection: true,
                list_detection: true,
            },
            images: ImageOptions {
                extract_images: false,
                image_quality: "medium".to_string(),
                max_image_width: 800,
                convert_to_webp: false,
            },
            output: OutputOptions {
                epub_version: "epub3".to_string(),
                embed_fonts: false,
                font_family: "default".to_string(),
                base_font_size: 12,
                line_height: 1.5,
                margins: 1.0,
                text_alignment: "justify".to_string(),
            },
            page_handling: PageHandlingOptions {
                skip_blank_pages: true,
                page_range: "all".to_string(),
                page_range_from: None,
                page_range_to: None,
                keep_page_breaks: false,
                remove_page_numbers: true,
                cover_page: "auto".to_string(),
            },
            output_folder: String::new(),
            book_id: None,
        }
    }

    fn test_metadata() -> PdfMetadata {
        PdfMetadata {
            title: Some("Test Book".to_string()),
            author: Some("Test Author".to_string()),
            page_count: 10,
            pdf_version: "1.7".to_string(),
            created_date: None,
            modified_date: None,
            producer: None,
            file_size: 1000,
        }
    }

    fn create_minimal_jpeg() -> Vec<u8> {
        let img = image::RgbImage::from_pixel(2, 2, image::Rgb([128, 128, 128]));
        let dynamic = image::DynamicImage::ImageRgb8(img);
        let mut buf = std::io::Cursor::new(Vec::new());
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 50);
        dynamic.write_with_encoder(encoder).unwrap();
        buf.into_inner()
    }

    fn make_content(text: &str) -> Vec<StructuredContent> {
        vec![StructuredContent::Paragraph { text: text.to_string() }]
    }

    fn make_cover(mime: &str) -> ExtractedImage {
        ExtractedImage {
            id: "cover_test".to_string(),
            data: create_minimal_jpeg(),
            mime_type: mime.to_string(),
            width: 600,
            height: 800,
        }
    }

    // -- has_cover flag --

    #[test]
    fn generate_epub_without_cover_sets_has_cover_false() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("test.epub");
        let content = make_content("Hello");
        let result = generate_epub(&content, &[], None, &test_metadata(), &test_options(), output.to_str().unwrap()).unwrap();
        assert!(!result.has_cover);
    }

    #[test]
    fn generate_epub_with_cover_sets_has_cover_true() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("test.epub");
        let content = make_content("Hello");
        let cover = make_cover("image/jpeg");
        let result = generate_epub(&content, &[], Some(&cover), &test_metadata(), &test_options(), output.to_str().unwrap()).unwrap();
        assert!(result.has_cover);
    }

    // -- file output --

    #[test]
    fn generate_epub_creates_file_without_cover() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("no_cover.epub");
        let content = make_content("Text");
        generate_epub(&content, &[], None, &test_metadata(), &test_options(), output.to_str().unwrap()).unwrap();
        assert!(output.exists());
        assert!(std::fs::metadata(&output).unwrap().len() > 0);
    }

    #[test]
    fn generate_epub_creates_file_with_cover() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("with_cover.epub");
        let content = make_content("Text");
        let cover = make_cover("image/jpeg");
        generate_epub(&content, &[], Some(&cover), &test_metadata(), &test_options(), output.to_str().unwrap()).unwrap();
        assert!(output.exists());
        assert!(std::fs::metadata(&output).unwrap().len() > 0);
    }

    #[test]
    fn epub_with_cover_is_larger_than_without() {
        let dir = tempfile::tempdir().unwrap();

        let no_cover_path = dir.path().join("no_cover.epub");
        let content = make_content("Text");
        let r1 = generate_epub(&content, &[], None, &test_metadata(), &test_options(), no_cover_path.to_str().unwrap()).unwrap();

        let with_cover_path = dir.path().join("with_cover.epub");
        let cover = make_cover("image/jpeg");
        let r2 = generate_epub(&content, &[], Some(&cover), &test_metadata(), &test_options(), with_cover_path.to_str().unwrap()).unwrap();

        assert!(r2.file_size > r1.file_size);
    }

    // -- cover with different MIME types --

    #[test]
    fn generate_epub_with_png_cover() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("png_cover.epub");
        let content = make_content("Text");
        let cover = make_cover("image/png");
        let result = generate_epub(&content, &[], Some(&cover), &test_metadata(), &test_options(), output.to_str().unwrap()).unwrap();
        assert!(result.has_cover);
        assert!(output.exists());
    }

    #[test]
    fn generate_epub_with_webp_cover() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("webp_cover.epub");
        let content = make_content("Text");
        let cover = make_cover("image/webp");
        let result = generate_epub(&content, &[], Some(&cover), &test_metadata(), &test_options(), output.to_str().unwrap()).unwrap();
        assert!(result.has_cover);
    }

    // -- result stats --

    #[test]
    fn result_image_count_excludes_cover() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("test.epub");
        let content = make_content("Text");
        let body_images = vec![ExtractedImage {
            id: "body_img".to_string(),
            data: create_minimal_jpeg(),
            mime_type: "image/jpeg".to_string(),
            width: 200,
            height: 200,
        }];
        let cover = make_cover("image/jpeg");
        let result = generate_epub(&content, &body_images, Some(&cover), &test_metadata(), &test_options(), output.to_str().unwrap()).unwrap();
        assert_eq!(result.images, 1);
    }

    // -- EPUB version compat --

    #[test]
    fn generate_epub2_with_cover() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("epub2_cover.epub");
        let content = make_content("Text");
        let cover = make_cover("image/jpeg");
        let mut opts = test_options();
        opts.output.epub_version = "epub2".to_string();
        let result = generate_epub(&content, &[], Some(&cover), &test_metadata(), &opts, output.to_str().unwrap()).unwrap();
        assert!(result.has_cover);
        assert!(output.exists());
    }

    // -- output_path in result --

    #[test]
    fn result_output_path_matches_input() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("specific_name.epub");
        let content = make_content("Text");
        let cover = make_cover("image/jpeg");
        let result = generate_epub(&content, &[], Some(&cover), &test_metadata(), &test_options(), output.to_str().unwrap()).unwrap();
        assert_eq!(result.output_path, output.to_str().unwrap());
    }
}
