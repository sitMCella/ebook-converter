use super::epub_generator;
use super::image_extractor;
use super::structure_detector;
use super::text_extractor;
use super::{ConversionOptions, ConversionProgress, ConversionResult};
use crate::pdf;
use crate::storage;
use lopdf::Document;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::Emitter;

pub async fn run_conversion(
    app: &tauri::AppHandle,
    path: &str,
    options: &ConversionOptions,
    cancel_token: Arc<AtomicBool>,
) -> Result<ConversionResult, String> {
    let path_owned = path.to_string();
    let output_folder = options.output_folder.clone();

    if cancel_token.load(Ordering::Relaxed) {
        return Err("Conversion cancelled".to_string());
    }

    // Images are extracted before the text so each one can be anchored in its
    // page's text at the position it occupies on the page.
    emit_progress(app, &path_owned, "extracting_images", 0, "Extracting images...");

    let mut images = if options.images.extract_images {
        let app_ref = app;
        let path_ref = path_owned.clone();
        match image_extractor::extract_images(path, &options.images, &|msg| {
            emit_progress(app_ref, &path_ref, "extracting_images", 10, msg);
        }) {
            Ok(imgs) => {
                emit_progress(
                    app,
                    &path_owned,
                    "extracting_images",
                    20,
                    &format!("Extracted {} image(s) total", imgs.len()),
                );
                imgs
            }
            Err(e) => {
                log::warn!("Image extraction failed: {}", e);
                emit_progress(
                    app,
                    &path_owned,
                    "extracting_images",
                    20,
                    &format!("Image extraction skipped: {}", e),
                );
                Vec::new()
            }
        }
    } else {
        emit_progress(app, &path_owned, "extracting_images", 20, "Image extraction disabled");
        Vec::new()
    };

    if cancel_token.load(Ordering::Relaxed) {
        return Err("Conversion cancelled".to_string());
    }

    emit_progress(app, &path_owned, "extracting_text", 20, "Extracting text from PDF...");

    let extracted = text_extractor::extract_text(path, &options.page_handling, &image_anchors(&images))?;
    let mut pages = extracted.pages;
    let mut first_content_page = extracted.first_page;

    if pages.is_empty() {
        return Err("No text could be extracted from the PDF".to_string());
    }

    let cover_image = match image_extractor::extract_cover_image(path, &options.page_handling.cover_page) {
        Ok(img) => img,
        Err(e) => {
            log::warn!("Cover extraction failed: {}", e);
            None
        }
    };

    // Page 1 is still in the text only when nothing was trimmed from the front.
    if cover_image.is_some() && first_content_page == 1 && pages.len() > 1 {
        let pdf_page_count = Document::load(path)
            .map(|doc| doc.get_pages().len())
            .unwrap_or(0);
        if pages.len() >= pdf_page_count {
            pages.remove(0);
            first_content_page = 2;
        }
    }

    if pages.is_empty() && cover_image.is_none() {
        return Err("No text could be extracted from the PDF".to_string());
    }

    emit_progress(
        app,
        &path_owned,
        "extracting_text",
        40,
        &format!("Extracted text from {} pages", pages.len()),
    );

    if cancel_token.load(Ordering::Relaxed) {
        return Err("Conversion cancelled".to_string());
    }

    emit_progress(app, &path_owned, "detecting_structure", 50, "Detecting headings and structure...");

    let mut content = structure_detector::detect_structure(&pages, &options.structure);
    content = structure_detector::apply_chapter_labels(content, options.page_handling.keep_chapter_labels);

    emit_progress(app, &path_owned, "detecting_structure", 70, "Structure detection complete");

    if cancel_token.load(Ordering::Relaxed) {
        return Err("Conversion cancelled".to_string());
    }

    // Page-1 images are the cover art (already in the EPUB as the cover);
    // repeating them in the body shows the cover again, often as tiles.
    // Images from pages trimmed off the front (e.g. front matter before the
    // TOC) go with them.
    images.retain(|img| {
        let page = image_page_number(img);
        page >= first_content_page && !(cover_image.is_some() && page == 1)
    });

    // Image lines anchored in the text of pages whose images were dropped
    // above leave no resource behind; remove them. Kept images not anchored
    // anywhere fall back to the end of their page.
    let kept: std::collections::HashSet<String> =
        images.iter().map(|img| img.resource_path()).collect();
    content.retain(|c| match c {
        structure_detector::StructuredContent::Image { resource_path, .. } => kept.contains(resource_path),
        _ => true,
    });
    let placed: std::collections::HashSet<String> = content
        .iter()
        .filter_map(|c| match c {
            structure_detector::StructuredContent::Image { resource_path, .. } => Some(resource_path.clone()),
            _ => None,
        })
        .collect();
    let unplaced: Vec<&image_extractor::ExtractedImage> = images
        .iter()
        .filter(|img| !placed.contains(&img.resource_path()))
        .collect();
    insert_images_into_content(&mut content, &unplaced, first_content_page);

    if cancel_token.load(Ordering::Relaxed) {
        return Err("Conversion cancelled".to_string());
    }

    emit_progress(app, &path_owned, "assembling_epub", 80, "Generating EPUB structure...");

    let output_path = if let Some(ref book_id) = options.book_id {
        storage::get_epub_output_path(app, book_id, path)?
            .to_str()
            .ok_or_else(|| "Invalid output path".to_string())?
            .to_string()
    } else {
        resolve_output_path(path, &output_folder)?
    };

    if let Some(parent) = std::path::Path::new(&output_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    let metadata = pdf::get_pdf_metadata_internal(path)?;

    emit_progress(app, &path_owned, "assembling_epub", 90, "Writing EPUB file...");

    let result = epub_generator::generate_epub(&content, &images, cover_image.as_ref(), &metadata, options, &output_path)?;

    emit_progress(app, &path_owned, "complete", 100, "Conversion complete.");

    Ok(result)
}

fn emit_progress(
    app: &tauri::AppHandle,
    path: &str,
    stage: &str,
    percent: u8,
    message: &str,
) {
    let _ = app.emit(
        "conversion-progress",
        ConversionProgress {
            path: path.to_string(),
            stage: stage.to_string(),
            percent,
            message: message.to_string(),
        },
    );
}

fn resolve_output_path(pdf_path: &str, output_folder: &str) -> Result<String, String> {
    let pdf = std::path::Path::new(pdf_path);
    let stem = pdf
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "Invalid PDF filename".to_string())?;

    let output_dir = std::path::Path::new(output_folder);
    let base_path = output_dir.join(format!("{}.epub", stem));

    if !base_path.exists() {
        return base_path
            .to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Invalid output path".to_string());
    }

    for i in 1..1000 {
        let numbered_path = output_dir.join(format!("{} ({}).epub", stem, i));
        if !numbered_path.exists() {
            return numbered_path
                .to_str()
                .map(|s| s.to_string())
                .ok_or_else(|| "Invalid output path".to_string());
        }
    }

    Err("Too many existing files with the same name".to_string())
}

fn image_page_number(img: &image_extractor::ExtractedImage) -> u32 {
    img.id
        .strip_prefix("img_p")
        .and_then(|rest| rest.split('_').next())
        .and_then(|n| n.parse::<u32>().ok())
        .unwrap_or(0)
}

/// Groups images by PDF page as text anchors, so `text_extractor` can place
/// each image line at the image's position in its page's text.
fn image_anchors(
    images: &[image_extractor::ExtractedImage],
) -> std::collections::HashMap<u32, Vec<text_extractor::ImageAnchor>> {
    let mut anchors: std::collections::HashMap<u32, Vec<text_extractor::ImageAnchor>> =
        std::collections::HashMap::new();
    for img in images {
        let Some(page_y) = img.page_y else { continue };
        let page = image_page_number(img);
        anchors.entry(page).or_default().push(text_extractor::ImageAnchor {
            page_y,
            mark: text_extractor::image_mark(
                &img.resource_path(),
                img.display_width_pct,
                &format!("Image from page {}", page),
            ),
        });
    }
    anchors
}

fn insert_images_into_content(
    content: &mut Vec<structure_detector::StructuredContent>,
    images: &[&image_extractor::ExtractedImage],
    first_page: u32,
) {
    if images.is_empty() {
        return;
    }

    let mut page_images: std::collections::HashMap<u32, Vec<&image_extractor::ExtractedImage>> =
        std::collections::HashMap::new();

    for &img in images {
        page_images.entry(image_page_number(img)).or_default().push(img);
    }

    let mut current_page: u32 = first_page;
    let mut i = 0;

    while i < content.len() {
        if matches!(content[i], structure_detector::StructuredContent::PageBreak) {
            if let Some(imgs) = page_images.remove(&current_page) {
                let nodes: Vec<_> = imgs
                    .iter()
                    .map(|img| {
                        structure_detector::StructuredContent::Image {
                            resource_path: img.resource_path(),
                            alt: format!("Image from page {}", current_page),
                            display_width_pct: img.display_width_pct,
                        }
                    })
                    .collect();
                let count = nodes.len();
                for (j, node) in nodes.into_iter().enumerate() {
                    content.insert(i + j, node);
                }
                i += count;
            }
            current_page += 1;
        }
        i += 1;
    }

    if let Some(imgs) = page_images.remove(&current_page) {
        for img in imgs {
            content.push(structure_detector::StructuredContent::Image {
                resource_path: img.resource_path(),
                alt: format!("Image from page {}", current_page),
                display_width_pct: img.display_width_pct,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_image(id: &str) -> image_extractor::ExtractedImage {
        image_extractor::ExtractedImage {
            id: id.to_string(),
            data: vec![],
            mime_type: "image/jpeg".to_string(),
            width: 10,
            height: 10,
            display_width_pct: None,
            page_y: None,
        }
    }

    fn image_positions(content: &[structure_detector::StructuredContent]) -> Vec<(usize, String)> {
        content
            .iter()
            .enumerate()
            .filter_map(|(i, c)| match c {
                structure_detector::StructuredContent::Image { resource_path, .. } => {
                    Some((i, resource_path.clone()))
                }
                _ => None,
            })
            .collect()
    }

    #[test]
    fn image_page_number_parses_id() {
        assert_eq!(image_page_number(&test_image("img_p12_0_3")), 12);
        assert_eq!(image_page_number(&test_image("cover_Im0")), 0);
    }

    #[test]
    fn insert_images_respects_first_page_offset() {
        use structure_detector::StructuredContent::{PageBreak, Paragraph};
        // Content starts at PDF page 2 (page 1 became the cover).
        let mut content = vec![
            Paragraph { text: "page 2".into() },
            PageBreak,
            Paragraph { text: "page 3".into() },
        ];
        let images = vec![test_image("img_p2_0_1"), test_image("img_p3_0_2")];
        let refs: Vec<_> = images.iter().collect();
        insert_images_into_content(&mut content, &refs, 2);
        let positions = image_positions(&content);
        assert_eq!(positions.len(), 2);
        // Page-2 image lands before the first page break, page-3 image at the end.
        assert!(positions[0].1.starts_with("images/img_p2_"));
        assert_eq!(positions[0].0, 1);
        assert!(positions[1].1.starts_with("images/img_p3_"));
        assert_eq!(positions[1].0, content.len() - 1);
    }

    #[test]
    fn resolve_output_path_basic() {
        let dir = tempfile::tempdir().unwrap();
        let result = resolve_output_path("/some/file.pdf", dir.path().to_str().unwrap()).unwrap();
        assert!(result.ends_with("file.epub"));
    }

    #[test]
    fn resolve_output_path_collision() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("file.epub"), b"").unwrap();
        let result = resolve_output_path("/some/file.pdf", dir.path().to_str().unwrap()).unwrap();
        assert!(result.ends_with("file (1).epub"));
    }

    #[test]
    fn resolve_output_path_multiple_collisions() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("file.epub"), b"").unwrap();
        std::fs::write(dir.path().join("file (1).epub"), b"").unwrap();
        let result = resolve_output_path("/some/file.pdf", dir.path().to_str().unwrap()).unwrap();
        assert!(result.ends_with("file (2).epub"));
    }
}
