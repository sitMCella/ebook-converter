use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::Manager;
use uuid::Uuid;

use crate::conversion::image_extractor::extract_cover_image;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredBook {
    pub book_id: String,
    pub stored_pdf_path: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BookMetadata {
    pub book_id: String,
    pub stored_pdf_path: String,
    pub original_path: String,
    pub original_name: String,
    pub file_size: u64,
    pub title: Option<String>,
    pub author: Option<String>,
    pub page_count: u32,
    pub pdf_version: Option<String>,
    pub created_date: Option<String>,
    pub modified_date: Option<String>,
    pub producer: Option<String>,
    pub status: String,
    #[serde(default)]
    pub output_path: Option<String>,
    #[serde(default)]
    pub images: Option<usize>,
    #[serde(default)]
    pub epub_file_size: Option<u64>,
    #[serde(default)]
    pub conversion_settings: Option<serde_json::Value>,
}

pub fn get_books_dir_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data directory: {}", e))?;
    let books_dir = app_data.join("books");
    if !books_dir.exists() {
        std::fs::create_dir_all(&books_dir)
            .map_err(|e| format!("Failed to create books directory: {}", e))?;
    }
    Ok(books_dir)
}

pub fn validate_book_id(book_id: &str) -> Result<(), String> {
    Uuid::parse_str(book_id).map_err(|_| format!("Invalid book ID: {}", book_id))?;
    Ok(())
}

pub fn create_book_dir(app: &tauri::AppHandle) -> Result<(String, PathBuf), String> {
    let books_dir = get_books_dir_path(app)?;
    let book_id = Uuid::new_v4().to_string();
    let book_dir = books_dir.join(&book_id);
    std::fs::create_dir_all(&book_dir)
        .map_err(|e| format!("Failed to create book directory: {}", e))?;
    Ok((book_id, book_dir))
}

fn save_cover_image(book_dir: &Path, pdf_path: &str) {
    let cover = match extract_cover_image(pdf_path, "firstPage") {
        Ok(Some(img)) => img,
        _ => return,
    };

    let dynamic_img = match image::load_from_memory(&cover.data) {
        Ok(img) => img,
        Err(_) => return,
    };

    let cover_path = book_dir.join("cover.png");
    let _ = dynamic_img.save_with_format(&cover_path, image::ImageFormat::Png);
}

pub fn copy_pdf_to_storage(
    app: &tauri::AppHandle,
    source_path: &str,
) -> Result<StoredBook, String> {
    let (book_id, book_dir) = create_book_dir(app)?;
    let filename = std::path::Path::new(source_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("source.pdf");
    let dest = book_dir.join(filename);

    if let Err(e) = std::fs::copy(source_path, &dest) {
        let _ = std::fs::remove_dir_all(&book_dir);
        return Err(format!("Failed to copy PDF to storage: {}", e));
    }

    let stored_path = dest
        .to_str()
        .ok_or_else(|| "Invalid stored path".to_string())?
        .to_string();

    save_cover_image(&book_dir, &stored_path);

    Ok(StoredBook {
        book_id,
        stored_pdf_path: stored_path,
    })
}

pub fn get_epub_output_path(app: &tauri::AppHandle, book_id: &str, pdf_path: &str) -> Result<PathBuf, String> {
    validate_book_id(book_id)?;
    let books_dir = get_books_dir_path(app)?;
    let stem = std::path::Path::new(pdf_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    Ok(books_dir.join(book_id).join(format!("{}.epub", stem)))
}

pub fn delete_book_dir(app: &tauri::AppHandle, book_id: &str) -> Result<(), String> {
    validate_book_id(book_id)?;
    let books_dir = get_books_dir_path(app)?;
    let book_dir = books_dir.join(book_id);
    if book_dir.exists() {
        std::fs::remove_dir_all(&book_dir)
            .map_err(|e| format!("Failed to delete book directory: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn import_pdf(
    app: tauri::AppHandle,
    source_path: String,
) -> Result<StoredBook, String> {
    copy_pdf_to_storage(&app, &source_path)
}

#[tauri::command]
pub fn delete_book(app: tauri::AppHandle, book_id: String) -> Result<(), String> {
    delete_book_dir(&app, &book_id)
}

#[tauri::command]
pub fn get_books_dir(app: tauri::AppHandle) -> Result<String, String> {
    let path = get_books_dir_path(&app)?;
    path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid books directory path".to_string())
}

#[tauri::command]
pub fn get_book_dir(app: tauri::AppHandle, book_id: String) -> Result<String, String> {
    validate_book_id(&book_id)?;
    let books_dir = get_books_dir_path(&app)?;
    let book_dir = books_dir.join(&book_id);
    if !book_dir.exists() {
        return Err(format!("Book directory does not exist: {}", book_id));
    }
    book_dir
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid book directory path".to_string())
}

pub fn write_book_metadata(books_dir: &PathBuf, metadata: &BookMetadata) -> Result<(), String> {
    validate_book_id(&metadata.book_id)?;
    let metadata_path = books_dir.join(&metadata.book_id).join("metadata.json");
    let json = serde_json::to_string_pretty(metadata)
        .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
    std::fs::write(&metadata_path, json)
        .map_err(|e| format!("Failed to write metadata file: {}", e))?;
    Ok(())
}

pub fn read_all_book_metadata(books_dir: &PathBuf) -> Result<Vec<BookMetadata>, String> {
    if !books_dir.exists() {
        return Ok(vec![]);
    }
    let entries = std::fs::read_dir(books_dir)
        .map_err(|e| format!("Failed to read books directory: {}", e))?;

    let mut books = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let metadata_path = path.join("metadata.json");
        if !metadata_path.exists() {
            continue;
        }
        let content = match std::fs::read_to_string(&metadata_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let metadata: BookMetadata = match serde_json::from_str(&content) {
            Ok(m) => m,
            Err(_) => continue,
        };
        books.push(metadata);
    }
    Ok(books)
}

#[tauri::command]
pub fn save_book_metadata(app: tauri::AppHandle, metadata: BookMetadata) -> Result<(), String> {
    let books_dir = get_books_dir_path(&app)?;
    write_book_metadata(&books_dir, &metadata)
}

#[tauri::command]
pub fn list_books(app: tauri::AppHandle) -> Result<Vec<BookMetadata>, String> {
    let books_dir = get_books_dir_path(&app)?;
    read_all_book_metadata(&books_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::{dictionary, Document, Object, Stream};

    fn create_jpeg_data(width: u32, height: u32) -> Vec<u8> {
        let img = image::RgbImage::from_pixel(width, height, image::Rgb([100, 150, 200]));
        let dynamic = image::DynamicImage::ImageRgb8(img);
        let mut buf = std::io::Cursor::new(Vec::new());
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 50);
        dynamic.write_with_encoder(encoder).unwrap();
        buf.into_inner()
    }

    fn create_pdf_with_image(dir: &Path, filename: &str, width: u32, height: u32) -> PathBuf {
        let mut doc = Document::with_version("1.7");
        let pages_id = doc.new_object_id();
        let page_id = doc.new_object_id();

        let jpeg_data = create_jpeg_data(width, height);
        let img_stream = Stream::new(
            dictionary! {
                "Type" => "XObject",
                "Subtype" => "Image",
                "Width" => Object::Integer(width as i64),
                "Height" => Object::Integer(height as i64),
                "ColorSpace" => "DeviceRGB",
                "BitsPerComponent" => Object::Integer(8),
                "Filter" => "DCTDecode",
            },
            jpeg_data,
        );
        let img_id = doc.add_object(Object::Stream(img_stream));

        let xobjects = dictionary! { "Im0" => img_id };
        let resources = dictionary! { "XObject" => xobjects };

        let page = dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Resources" => resources,
        };
        doc.objects.insert(page_id, Object::Dictionary(page));

        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => Object::Integer(1),
        };
        doc.objects.insert(pages_id, Object::Dictionary(pages));

        let root_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", root_id);

        let path = dir.join(filename);
        doc.save(&path).unwrap();
        path
    }

    fn create_pdf_no_images(dir: &Path, filename: &str) -> PathBuf {
        let mut doc = Document::with_version("1.7");
        let pages_id = doc.new_object_id();
        let page_id = doc.new_object_id();

        let page = dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
        };
        doc.objects.insert(page_id, Object::Dictionary(page));

        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => Object::Integer(1),
        };
        doc.objects.insert(pages_id, Object::Dictionary(pages));

        let root_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", root_id);

        let path = dir.join(filename);
        doc.save(&path).unwrap();
        path
    }

    // -- save_cover_image tests --

    #[test]
    fn save_cover_image_creates_png_when_pdf_has_image() {
        let dir = tempfile::tempdir().unwrap();
        let book_dir = dir.path().join("book1");
        std::fs::create_dir_all(&book_dir).unwrap();
        let pdf_path = create_pdf_with_image(&book_dir, "source.pdf", 400, 500);

        save_cover_image(&book_dir, pdf_path.to_str().unwrap());

        let cover_path = book_dir.join("cover.png");
        assert!(cover_path.exists());

        let data = std::fs::read(&cover_path).unwrap();
        assert!(data.starts_with(&[0x89, b'P', b'N', b'G']));
    }

    #[test]
    fn save_cover_image_does_not_create_file_when_pdf_has_no_images() {
        let dir = tempfile::tempdir().unwrap();
        let book_dir = dir.path().join("book1");
        std::fs::create_dir_all(&book_dir).unwrap();
        let pdf_path = create_pdf_no_images(&book_dir, "source.pdf");

        save_cover_image(&book_dir, pdf_path.to_str().unwrap());

        let cover_path = book_dir.join("cover.png");
        assert!(!cover_path.exists());
    }

    #[test]
    fn save_cover_image_does_not_error_on_invalid_pdf() {
        let dir = tempfile::tempdir().unwrap();
        let book_dir = dir.path().join("book1");
        std::fs::create_dir_all(&book_dir).unwrap();

        save_cover_image(&book_dir, "/nonexistent/file.pdf");

        let cover_path = book_dir.join("cover.png");
        assert!(!cover_path.exists());
    }

    #[test]
    fn save_cover_image_produces_valid_png() {
        let dir = tempfile::tempdir().unwrap();
        let book_dir = dir.path().join("book1");
        std::fs::create_dir_all(&book_dir).unwrap();
        let pdf_path = create_pdf_with_image(&book_dir, "source.pdf", 300, 400);

        save_cover_image(&book_dir, pdf_path.to_str().unwrap());

        let cover_path = book_dir.join("cover.png");
        let img = image::open(&cover_path).unwrap();
        assert!(img.width() > 0);
        assert!(img.height() > 0);
    }

    // -- existing tests --

    #[test]
    fn validate_book_id_accepts_valid_uuid() {
        let id = Uuid::new_v4().to_string();
        assert!(validate_book_id(&id).is_ok());
    }

    #[test]
    fn validate_book_id_rejects_invalid_string() {
        assert!(validate_book_id("not-a-uuid").is_err());
        assert!(validate_book_id("../../../etc/passwd").is_err());
        assert!(validate_book_id("").is_err());
    }

    fn make_test_metadata(book_id: &str) -> BookMetadata {
        BookMetadata {
            book_id: book_id.to_string(),
            stored_pdf_path: format!("/books/{}/source.pdf", book_id),
            original_path: "/docs/test.pdf".to_string(),
            original_name: "test.pdf".to_string(),
            file_size: 1024,
            title: Some("Test Book".to_string()),
            author: Some("Author".to_string()),
            page_count: 10,
            pdf_version: Some("1.7".to_string()),
            created_date: None,
            modified_date: None,
            producer: None,
            status: "ready".to_string(),
            output_path: None,
            images: None,
            epub_file_size: None,
            conversion_settings: None,
        }
    }

    #[test]
    fn write_and_read_book_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let books_dir = dir.path().to_path_buf();
        let book_id = Uuid::new_v4().to_string();
        let book_dir = books_dir.join(&book_id);
        std::fs::create_dir_all(&book_dir).unwrap();

        let metadata = make_test_metadata(&book_id);
        write_book_metadata(&books_dir, &metadata).unwrap();

        let books = read_all_book_metadata(&books_dir).unwrap();
        assert_eq!(books.len(), 1);
        assert_eq!(books[0].book_id, book_id);
        assert_eq!(books[0].title, Some("Test Book".to_string()));
        assert_eq!(books[0].original_name, "test.pdf");
        assert_eq!(books[0].page_count, 10);
    }

    #[test]
    fn read_all_book_metadata_returns_empty_for_nonexistent_dir() {
        let dir = tempfile::tempdir().unwrap();
        let nonexistent = dir.path().join("nonexistent");
        let books = read_all_book_metadata(&nonexistent).unwrap();
        assert!(books.is_empty());
    }

    #[test]
    fn read_all_book_metadata_skips_dirs_without_metadata_json() {
        let dir = tempfile::tempdir().unwrap();
        let books_dir = dir.path().to_path_buf();
        let book_id = Uuid::new_v4().to_string();
        std::fs::create_dir_all(books_dir.join(&book_id)).unwrap();

        let books = read_all_book_metadata(&books_dir).unwrap();
        assert!(books.is_empty());
    }

    #[test]
    fn read_all_book_metadata_skips_invalid_json() {
        let dir = tempfile::tempdir().unwrap();
        let books_dir = dir.path().to_path_buf();
        let book_id = Uuid::new_v4().to_string();
        let book_dir = books_dir.join(&book_id);
        std::fs::create_dir_all(&book_dir).unwrap();
        std::fs::write(book_dir.join("metadata.json"), "not valid json").unwrap();

        let books = read_all_book_metadata(&books_dir).unwrap();
        assert!(books.is_empty());
    }

    #[test]
    fn write_book_metadata_rejects_invalid_book_id() {
        let dir = tempfile::tempdir().unwrap();
        let books_dir = dir.path().to_path_buf();
        let mut metadata = make_test_metadata("not-a-uuid");
        metadata.book_id = "not-a-uuid".to_string();
        assert!(write_book_metadata(&books_dir, &metadata).is_err());
    }

    #[test]
    fn read_all_book_metadata_reads_multiple_books() {
        let dir = tempfile::tempdir().unwrap();
        let books_dir = dir.path().to_path_buf();

        for _ in 0..3 {
            let book_id = Uuid::new_v4().to_string();
            let book_dir = books_dir.join(&book_id);
            std::fs::create_dir_all(&book_dir).unwrap();
            let metadata = make_test_metadata(&book_id);
            write_book_metadata(&books_dir, &metadata).unwrap();
        }

        let books = read_all_book_metadata(&books_dir).unwrap();
        assert_eq!(books.len(), 3);
    }

    #[test]
    fn write_and_read_conversion_settings() {
        let dir = tempfile::tempdir().unwrap();
        let books_dir = dir.path().to_path_buf();
        let book_id = Uuid::new_v4().to_string();
        let book_dir = books_dir.join(&book_id);
        std::fs::create_dir_all(&book_dir).unwrap();

        let mut metadata = make_test_metadata(&book_id);
        metadata.conversion_settings = Some(serde_json::json!({
            "structure": { "headingLevelThreshold": 3 },
            "images": { "imageQuality": "low" }
        }));
        write_book_metadata(&books_dir, &metadata).unwrap();

        let books = read_all_book_metadata(&books_dir).unwrap();
        assert_eq!(books.len(), 1);
        let settings = books[0].conversion_settings.as_ref().unwrap();
        assert_eq!(settings["structure"]["headingLevelThreshold"], 3);
        assert_eq!(settings["images"]["imageQuality"], "low");
    }

    #[test]
    fn read_metadata_without_conversion_settings_defaults_to_none() {
        let dir = tempfile::tempdir().unwrap();
        let books_dir = dir.path().to_path_buf();
        let book_id = Uuid::new_v4().to_string();
        let book_dir = books_dir.join(&book_id);
        std::fs::create_dir_all(&book_dir).unwrap();

        let metadata = make_test_metadata(&book_id);
        write_book_metadata(&books_dir, &metadata).unwrap();

        let metadata_path = book_dir.join("metadata.json");
        let content = std::fs::read_to_string(&metadata_path).unwrap();
        let mut json: serde_json::Value = serde_json::from_str(&content).unwrap();
        json.as_object_mut().unwrap().remove("conversionSettings");
        std::fs::write(&metadata_path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let books = read_all_book_metadata(&books_dir).unwrap();
        assert_eq!(books.len(), 1);
        assert!(books[0].conversion_settings.is_none());
    }
}
