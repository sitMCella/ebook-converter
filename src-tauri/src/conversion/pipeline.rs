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

    emit_progress(app, &path_owned, "extracting_text", 0, "Extracting text from PDF...");

    if cancel_token.load(Ordering::Relaxed) {
        return Err("Conversion cancelled".to_string());
    }

    let mut pages = text_extractor::extract_text(path, &options.page_handling)?;

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

    if cover_image.is_some() && pages.len() > 1 {
        let pdf_page_count = Document::load(path)
            .map(|doc| doc.get_pages().len())
            .unwrap_or(0);
        if pages.len() >= pdf_page_count {
            pages.remove(0);
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

    emit_progress(app, &path_owned, "detecting_structure", 40, "Detecting headings and structure...");

    let mut content = structure_detector::detect_structure(&pages, &options.structure);

    emit_progress(app, &path_owned, "detecting_structure", 50, "Structure detection complete");

    if cancel_token.load(Ordering::Relaxed) {
        return Err("Conversion cancelled".to_string());
    }

    emit_progress(app, &path_owned, "extracting_images", 50, "Extracting images...");

    let images = if options.images.extract_images {
        match image_extractor::extract_images(path, &options.images) {
            Ok(imgs) => {
                emit_progress(
                    app,
                    &path_owned,
                    "extracting_images",
                    70,
                    &format!("Extracted {} images", imgs.len()),
                );
                imgs
            }
            Err(e) => {
                log::warn!("Image extraction failed: {}", e);
                emit_progress(
                    app,
                    &path_owned,
                    "extracting_images",
                    70,
                    &format!("Image extraction skipped: {}", e),
                );
                Vec::new()
            }
        }
    } else {
        emit_progress(app, &path_owned, "extracting_images", 70, "Image extraction disabled");
        Vec::new()
    };

    insert_images_into_content(&mut content, &images);

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

fn insert_images_into_content(
    content: &mut Vec<structure_detector::StructuredContent>,
    images: &[image_extractor::ExtractedImage],
) {
    if images.is_empty() {
        return;
    }

    let mut page_images: std::collections::HashMap<u32, Vec<&image_extractor::ExtractedImage>> =
        std::collections::HashMap::new();

    for img in images {
        let page_num = img
            .id
            .strip_prefix("img_p")
            .and_then(|rest| rest.split('_').next())
            .and_then(|n| n.parse::<u32>().ok())
            .unwrap_or(0);
        page_images.entry(page_num).or_default().push(img);
    }

    let mut current_page: u32 = 1;
    let mut i = 0;

    while i < content.len() {
        if matches!(content[i], structure_detector::StructuredContent::PageBreak) {
            if let Some(imgs) = page_images.remove(&current_page) {
                let nodes: Vec<_> = imgs
                    .iter()
                    .map(|img| {
                        let ext = match img.mime_type.as_str() {
                            "image/jpeg" => "jpg",
                            "image/png" => "png",
                            "image/webp" => "webp",
                            _ => "bin",
                        };
                        structure_detector::StructuredContent::Image {
                            resource_path: format!("images/{}_{}.{}", img.id, img.width, ext),
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
            let ext = match img.mime_type.as_str() {
                "image/jpeg" => "jpg",
                "image/png" => "png",
                "image/webp" => "webp",
                _ => "bin",
            };
            content.push(structure_detector::StructuredContent::Image {
                resource_path: format!("images/{}_{}.{}", img.id, img.width, ext),
                alt: format!("Image from page {}", current_page),
                display_width_pct: img.display_width_pct,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
