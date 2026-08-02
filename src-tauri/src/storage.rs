use serde::Serialize;
use std::path::PathBuf;
use tauri::Manager;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredBook {
    pub book_id: String,
    pub stored_pdf_path: String,
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
    let dest = book_dir.join("source.pdf");

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

pub fn get_epub_output_path(app: &tauri::AppHandle, book_id: &str) -> Result<PathBuf, String> {
    validate_book_id(book_id)?;
    let books_dir = get_books_dir_path(app)?;
    Ok(books_dir.join(book_id).join("output.epub"))
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
}
