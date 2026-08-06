use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;
use uuid::Uuid;

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
}
