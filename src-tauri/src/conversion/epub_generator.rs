use super::css;
use super::image_extractor::ExtractedImage;
use super::structure_detector::{StructuredContent, superscript_to_digit};
use super::text_extractor::{NOTE_FIELD_SEP, NOTE_REF_END, NOTE_REF_START};
use std::collections::HashSet;
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
            .add_cover_image(&cover_resource_path, cover.data.as_slice(), &cover.mime_type)
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
        let resource_path = image.resource_path();
        builder
            .add_resource(&resource_path, image.data.as_slice(), &image.mime_type)
            .map_err(|e| format!("Failed to add image {}: {}", image.id, e))?;
    }

    let xhtml = content_to_xhtml(
        content,
        options.page_handling.keep_page_breaks,
        options.output.epub_version != "epub2",
    );
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

/// Note-reference state shared across the document while rendering.
struct InlineState<'a> {
    footnote_numbers: &'a HashSet<u32>,
    link_footnotes: bool,
    linked_footnotes: HashSet<u32>,
    /// Ids of kept notes whose first reference got an `id="noteref-N"`
    /// anchor, so the note can link back to it.
    linked_notes: HashSet<u32>,
    epub3: bool,
}

impl InlineState<'_> {
    /// Escapes `text` and turns footnote/note reference marks into links.
    fn render(&mut self, text: &str) -> String {
        let escaped = escape_xml(text);
        let escaped = if self.link_footnotes {
            linkify_footnote_refs(&escaped, self.footnote_numbers, &mut self.linked_footnotes)
        } else {
            escaped
        };
        render_note_refs(&escaped, self.epub3, &mut self.linked_notes)
    }
}

/// Replaces `NOTE_REF_START` id `NOTE_FIELD_SEP` label `NOTE_REF_END` marks
/// (already XML-escaped text) with superscript links to `#note-<id>`. The
/// first reference to each note gets the `noteref-<id>` anchor the note
/// links back to. Malformed marks are dropped, never leaked into the XHTML.
fn render_note_refs(text: &str, epub3: bool, linked: &mut HashSet<u32>) -> String {
    if !text.contains(NOTE_REF_START) {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len() + 64);
    let mut rest = text;
    while let Some(start) = rest.find(NOTE_REF_START) {
        out.push_str(&rest[..start]);
        let after = &rest[start + NOTE_REF_START.len_utf8()..];
        let Some(end) = after.find(NOTE_REF_END) else {
            rest = after;
            continue;
        };
        let inner = &after[..end];
        rest = &after[end + NOTE_REF_END.len_utf8()..];
        let Some((id, label)) = inner.split_once(NOTE_FIELD_SEP) else { continue };
        let Ok(id) = id.parse::<u32>() else { continue };
        let anchor = if linked.insert(id) {
            format!(" id=\"noteref-{id}\"")
        } else {
            String::new()
        };
        let semantics = if epub3 {
            " epub:type=\"noteref\" role=\"doc-noteref\""
        } else {
            ""
        };
        out.push_str(&format!(
            "<a class=\"noteref\" href=\"#note-{id}\"{anchor}{semantics}><sup>{label}</sup></a>"
        ));
    }
    out.push_str(rest);
    out
}

fn content_to_xhtml(content: &[StructuredContent], keep_page_breaks: bool, epub3: bool) -> String {
    let html_open = if epub3 {
        r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">"#
    } else {
        r#"<html xmlns="http://www.w3.org/1999/xhtml">"#
    };
    let mut html = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
{html_open}
<head><title></title><link rel="stylesheet" type="text/css" href="stylesheet.css" /></head>
<body>
"#,
    );

    let footnote_numbers: HashSet<u32> = content
        .iter()
        .filter_map(|c| match c {
            StructuredContent::Footnote { number, .. } => Some(*number),
            _ => None,
        })
        .collect();
    let footnote_count = content
        .iter()
        .filter(|c| matches!(c, StructuredContent::Footnote { .. }))
        .count();
    let link_refs = !footnote_numbers.is_empty() && footnote_numbers.len() == footnote_count;

    let mut in_ul = false;
    let mut in_ol = false;
    let mut inline = InlineState {
        footnote_numbers: &footnote_numbers,
        link_footnotes: link_refs,
        linked_footnotes: HashSet::new(),
        linked_notes: HashSet::new(),
        epub3,
    };

    for item in content {
        match item {
            StructuredContent::Heading { level, text } => {
                close_lists(&mut html, &mut in_ul, &mut in_ol);
                let tag = format!("h{}", level.min(&6));
                let text_out = inline.render(text);
                html.push_str(&format!("<{}>{}</{}>\n", tag, text_out, tag));
            }
            StructuredContent::ChapterLabel { text } => {
                close_lists(&mut html, &mut in_ul, &mut in_ol);
                html.push_str(&format!("<p class=\"chapter-label\">{}</p>\n", inline.render(text)));
            }
            StructuredContent::Paragraph { text } => {
                close_lists(&mut html, &mut in_ul, &mut in_ol);
                let text_out = inline.render(text);
                html.push_str(&format!("<p>{}</p>\n", text_out));
            }
            StructuredContent::ListItem { text, ordered, number } => {
                if *ordered {
                    if in_ul {
                        html.push_str("</ul>\n");
                        in_ul = false;
                    }
                    if !in_ol {
                        match number {
                            Some(n) if *n != 1 => html.push_str(&format!("<ol start=\"{}\">\n", n)),
                            _ => html.push_str("<ol>\n"),
                        }
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
                let text_out = inline.render(text);
                html.push_str(&format!("<li>{}</li>\n", text_out));
            }
            StructuredContent::Image { resource_path, alt, display_width_pct } => {
                close_lists(&mut html, &mut in_ul, &mut in_ol);
                if let Some(pct) = display_width_pct {
                    html.push_str(&format!(
                        "<div class=\"image\"><img src=\"{}\" alt=\"{}\" style=\"max-width:{}%\" /></div>\n",
                        escape_xml(resource_path),
                        escape_xml(alt),
                        pct
                    ));
                } else {
                    html.push_str(&format!(
                        "<div class=\"image\"><img src=\"{}\" alt=\"{}\" /></div>\n",
                        escape_xml(resource_path),
                        escape_xml(alt)
                    ));
                }
            }
            StructuredContent::CodeBlock { text } => {
                close_lists(&mut html, &mut in_ul, &mut in_ol);
                html.push_str(&format!("<pre><code>{}</code></pre>\n", inline.render(text)));
            }
            StructuredContent::Note { id, label, text } => {
                close_lists(&mut html, &mut in_ul, &mut in_ol);
                let label_text = format!("Note {}", escape_xml(label));
                // Link back to the reference when the text cites this note.
                let label_html = if inline.linked_notes.contains(id) {
                    format!("<a class=\"note-label\" href=\"#noteref-{id}\">{label_text}</a>")
                } else {
                    format!("<span class=\"note-label\">{label_text}</span>")
                };
                let semantics = if epub3 {
                    " epub:type=\"footnote\" role=\"doc-footnote\""
                } else {
                    ""
                };
                html.push_str(&format!(
                    "<div class=\"note\" id=\"note-{id}\"{semantics}><p>{label_html} {}</p></div>\n",
                    inline.render(text)
                ));
            }
            StructuredContent::Footnote { .. } => {
                close_lists(&mut html, &mut in_ul, &mut in_ol);
            }
            StructuredContent::BlankLine => {
                close_lists(&mut html, &mut in_ul, &mut in_ol);
            }
            StructuredContent::PageBreak => {
                close_lists(&mut html, &mut in_ul, &mut in_ol);
                if keep_page_breaks {
                    html.push_str("<hr class=\"page-break\" />\n");
                }
            }
        }
    }

    close_lists(&mut html, &mut in_ul, &mut in_ol);

    let footnotes: Vec<_> = content
        .iter()
        .filter_map(|c| match c {
            StructuredContent::Footnote { number, text } => Some((*number, text.as_str())),
            _ => None,
        })
        .collect();

    if !footnotes.is_empty() {
        html.push_str("<section class=\"footnotes\">\n");
        html.push_str("<hr class=\"footnote-separator\" />\n");
        html.push_str("<ol>\n");
        for (num, text) in &footnotes {
            let back_link = if inline.linked_footnotes.contains(num) {
                format!(" <a href=\"#fnref{}\">\u{21A9}</a>", num)
            } else {
                String::new()
            };
            html.push_str(&format!(
                "<li id=\"fn{}\" value=\"{}\"><p>{}{}</p></li>\n",
                num,
                num,
                escape_xml(text),
                back_link
            ));
        }
        html.push_str("</ol>\n");
        html.push_str("</section>\n");
    }

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

fn linkify_footnote_refs(
    text: &str,
    footnote_numbers: &HashSet<u32>,
    linked_refs: &mut HashSet<u32>,
) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(&c) = chars.peek() {
        if superscript_to_digit(c).is_some() {
            let mut num = 0u32;
            let mut raw = String::new();
            while let Some(&sc) = chars.peek() {
                if let Some(d) = superscript_to_digit(sc) {
                    num = num * 10 + d;
                    raw.push(sc);
                    chars.next();
                } else {
                    break;
                }
            }

            if num > 0 && footnote_numbers.contains(&num) {
                if linked_refs.insert(num) {
                    result.push_str(&format!(
                        "<a href=\"#fn{}\" id=\"fnref{}\"><sup>{}</sup></a>",
                        num, num, num
                    ));
                } else {
                    result.push_str(&format!(
                        "<a href=\"#fn{}\"><sup>{}</sup></a>",
                        num, num
                    ));
                }
            } else {
                result.push_str(&raw);
            }
        } else {
            result.push(chars.next().unwrap());
        }
    }

    result
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
                include_toc: false,
                keep_notes: false,
                keep_chapter_labels: true,
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
            display_width_pct: None,
            page_y: None,
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

    fn read_opf(path: &std::path::Path) -> String {
        use std::io::Read;
        let file = std::fs::File::open(path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let mut opf = String::new();
        archive.by_name("OEBPS/content.opf").unwrap().read_to_string(&mut opf).unwrap();
        opf
    }

    #[test]
    fn epub3_cover_is_declared_in_opf() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("cover_meta.epub");
        let cover = make_cover("image/png");
        generate_epub(&make_content("Text"), &[], Some(&cover), &test_metadata(), &test_options(), output.to_str().unwrap()).unwrap();
        let opf = read_opf(&output);
        assert!(opf.contains("<meta name=\"cover\" content=\"cover-image\"/>"));
        assert!(opf.contains("properties=\"cover-image\""));
    }

    #[test]
    fn epub2_cover_is_declared_in_opf() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("cover_meta2.epub");
        let cover = make_cover("image/png");
        let mut opts = test_options();
        opts.output.epub_version = "epub2".to_string();
        generate_epub(&make_content("Text"), &[], Some(&cover), &test_metadata(), &opts, output.to_str().unwrap()).unwrap();
        assert!(read_opf(&output).contains("<meta name=\"cover\" content=\"cover-image\"/>"));
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
            display_width_pct: None,
            page_y: None,
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

    #[test]
    fn xhtml_renders_chapter_label_before_title() {
        let content = vec![
            StructuredContent::ChapterLabel { text: "CHAPTER 1".to_string() },
            StructuredContent::Heading { level: 1, text: "Domain Modeling".to_string() },
        ];
        let xhtml = content_to_xhtml(&content, false, true);
        assert!(xhtml.contains("<p class=\"chapter-label\">CHAPTER 1</p>\n<h1>Domain Modeling</h1>"));
    }

    // -- keep_page_breaks --

    #[test]
    fn xhtml_omits_page_break_when_disabled() {
        let content = vec![
            StructuredContent::Paragraph { text: "Page one".to_string() },
            StructuredContent::PageBreak,
            StructuredContent::Paragraph { text: "Page two".to_string() },
        ];
        let xhtml = content_to_xhtml(&content, false, true);
        assert!(!xhtml.contains("page-break"));
    }

    #[test]
    fn xhtml_inserts_page_break_hr_when_enabled() {
        let content = vec![
            StructuredContent::Paragraph { text: "Page one".to_string() },
            StructuredContent::PageBreak,
            StructuredContent::Paragraph { text: "Page two".to_string() },
        ];
        let xhtml = content_to_xhtml(&content, true, true);
        assert!(xhtml.contains(r#"<hr class="page-break" />"#));
    }

    #[test]
    fn xhtml_page_break_closes_list_before_hr() {
        let content = vec![
            StructuredContent::ListItem { text: "item".to_string(), ordered: false, number: None },
            StructuredContent::PageBreak,
            StructuredContent::Paragraph { text: "next page".to_string() },
        ];
        let xhtml = content_to_xhtml(&content, true, true);
        let ul_close = xhtml.find("</ul>").unwrap();
        let hr_pos = xhtml.find(r#"<hr class="page-break" />"#).unwrap();
        assert!(ul_close < hr_pos);
    }

    #[test]
    fn xhtml_resumed_ordered_list_keeps_numbering() {
        let content = vec![
            StructuredContent::ListItem { text: "first".to_string(), ordered: true, number: Some(1) },
            StructuredContent::Paragraph { text: "interlude".to_string() },
            StructuredContent::ListItem { text: "second".to_string(), ordered: true, number: Some(2) },
        ];
        let xhtml = content_to_xhtml(&content, false, true);
        assert!(xhtml.contains("<ol>\n<li>first</li>"));
        assert!(xhtml.contains("<ol start=\"2\">\n<li>second</li>"));
    }

    #[test]
    fn generate_epub_respects_keep_page_breaks() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("breaks.epub");
        let content = vec![
            StructuredContent::Paragraph { text: "One".to_string() },
            StructuredContent::PageBreak,
            StructuredContent::Paragraph { text: "Two".to_string() },
        ];
        let mut opts = test_options();
        opts.page_handling.keep_page_breaks = true;
        generate_epub(&content, &[], None, &test_metadata(), &opts, output.to_str().unwrap()).unwrap();
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

    // -- footnotes --

    #[test]
    fn xhtml_renders_endnotes_section() {
        let content = vec![
            StructuredContent::Paragraph { text: "Body text.".to_string() },
            StructuredContent::Footnote { number: 1, text: "A footnote.".to_string() },
        ];
        let xhtml = content_to_xhtml(&content, false, true);
        assert!(xhtml.contains("<section class=\"footnotes\">"));
        assert!(xhtml.contains("id=\"fn1\""));
        assert!(xhtml.contains("A footnote."));
    }

    #[test]
    fn xhtml_links_inline_superscript_refs() {
        let content = vec![
            StructuredContent::Paragraph { text: "Text\u{00B9} here.".to_string() },
            StructuredContent::Footnote { number: 1, text: "The note.".to_string() },
        ];
        let xhtml = content_to_xhtml(&content, false, true);
        assert!(xhtml.contains("href=\"#fn1\""));
        assert!(xhtml.contains("id=\"fnref1\""));
        assert!(xhtml.contains("<sup>1</sup>"));
    }

    #[test]
    fn xhtml_footnote_back_links() {
        let content = vec![
            StructuredContent::Paragraph { text: "Ref\u{00B9} here.".to_string() },
            StructuredContent::Footnote { number: 1, text: "Note text.".to_string() },
        ];
        let xhtml = content_to_xhtml(&content, false, true);
        assert!(xhtml.contains("href=\"#fnref1\""));
        assert!(xhtml.contains("\u{21A9}"));
    }

    #[test]
    fn xhtml_no_footnotes_section_without_footnotes() {
        let content = vec![
            StructuredContent::Paragraph { text: "Just text.".to_string() },
        ];
        let xhtml = content_to_xhtml(&content, false, true);
        assert!(!xhtml.contains("footnotes"));
    }

    // -- code blocks --

    #[test]
    fn xhtml_renders_code_block_as_pre_code() {
        let content = vec![
            StructuredContent::CodeBlock {
                text: "class Batch:\n    def __init__(self):\n        pass".to_string(),
            },
        ];
        let xhtml = content_to_xhtml(&content, false, true);
        assert!(xhtml.contains("<pre><code>class Batch:\n    def __init__(self):\n        pass</code></pre>"));
    }

    #[test]
    fn xhtml_escapes_code_block_content() {
        let content = vec![
            StructuredContent::CodeBlock {
                text: "if x < 1 && y > 0:".to_string(),
            },
        ];
        let xhtml = content_to_xhtml(&content, false, true);
        assert!(xhtml.contains("if x &lt; 1 &amp;&amp; y &gt; 0:"));
    }

    // -- kept notes --

    fn note_ref_text(id: u32, label: &str) -> String {
        format!("{NOTE_REF_START}{id}{NOTE_FIELD_SEP}{label}{NOTE_REF_END}")
    }

    #[test]
    fn xhtml_links_note_reference_and_note_both_ways() {
        let content = vec![
            StructuredContent::Paragraph { text: format!("about algorithms.{}", note_ref_text(4, "1")) },
            StructuredContent::Note { id: 4, label: "1".into(), text: "CRC cards & more.".into() },
        ];
        let xhtml = content_to_xhtml(&content, false, true);
        assert!(xhtml.contains(
            r##"algorithms.<a class="noteref" href="#note-4" id="noteref-4" epub:type="noteref" role="doc-noteref"><sup>1</sup></a>"##
        ));
        assert!(xhtml.contains(
            r##"<div class="note" id="note-4" epub:type="footnote" role="doc-footnote"><p><a class="note-label" href="#noteref-4">Note 1</a> CRC cards &amp; more.</p></div>"##
        ));
        assert!(xhtml.contains(r#"xmlns:epub="http://www.idpf.org/2007/ops""#));
        assert!(!xhtml.contains(NOTE_REF_START));
    }

    #[test]
    fn xhtml_same_label_in_two_chapters_links_to_distinct_notes() {
        let content = vec![
            StructuredContent::Paragraph { text: format!("Chapter one{}", note_ref_text(1, "1")) },
            StructuredContent::Note { id: 1, label: "1".into(), text: "First.".into() },
            StructuredContent::Paragraph { text: format!("Chapter two{}", note_ref_text(2, "1")) },
            StructuredContent::Note { id: 2, label: "1".into(), text: "Second.".into() },
        ];
        let xhtml = content_to_xhtml(&content, false, true);
        assert!(xhtml.contains(r##"href="#note-1" id="noteref-1""##));
        assert!(xhtml.contains(r##"href="#note-2" id="noteref-2""##));
        assert!(xhtml.contains(r#"id="note-1""#));
        assert!(xhtml.contains(r#"id="note-2""#));
    }

    #[test]
    fn xhtml_repeated_reference_gets_anchor_once() {
        let r = note_ref_text(9, "*");
        let content = vec![
            StructuredContent::Paragraph { text: format!("a{r} b{r}") },
            StructuredContent::Note { id: 9, label: "*".into(), text: "Star.".into() },
        ];
        let xhtml = content_to_xhtml(&content, false, true);
        assert_eq!(xhtml.matches(r#"id="noteref-9""#).count(), 1);
        assert_eq!(xhtml.matches(r##"href="#note-9""##).count(), 2);
    }

    #[test]
    fn xhtml_uncited_note_has_no_back_link() {
        let content = vec![StructuredContent::Note { id: 3, label: "2".into(), text: "Alone.".into() }];
        let xhtml = content_to_xhtml(&content, false, true);
        assert!(xhtml.contains(r#"<span class="note-label">Note 2</span> Alone."#));
        assert!(!xhtml.contains("#noteref-3"));
    }

    #[test]
    fn xhtml_epub2_notes_omit_epub3_attributes() {
        let content = vec![
            StructuredContent::Paragraph { text: format!("text{}", note_ref_text(1, "1")) },
            StructuredContent::Note { id: 1, label: "1".into(), text: "Note.".into() },
        ];
        let xhtml = content_to_xhtml(&content, false, false);
        assert!(xhtml.contains(r##"<a class="noteref" href="#note-1" id="noteref-1"><sup>1</sup></a>"##));
        assert!(xhtml.contains(r#"<div class="note" id="note-1"><p>"#));
        assert!(!xhtml.contains("epub:type"));
        assert!(!xhtml.contains("xmlns:epub"));
    }

    #[test]
    fn render_note_refs_drops_malformed_marks() {
        let mut linked = HashSet::new();
        let text = format!("a{NOTE_REF_START}x{NOTE_FIELD_SEP}1{NOTE_REF_END}b{NOTE_REF_START}c");
        assert_eq!(render_note_refs(&text, true, &mut linked), "abc");
    }
}
