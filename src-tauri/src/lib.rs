mod pdf;
mod conversion;
mod epub_reader;
mod storage;

use std::collections::HashMap;
use std::sync::Mutex;

#[tauri::command]
fn open_path(path: String) -> Result<(), String> {
    open::that_detached(&path)
        .map_err(|e| format!("Failed to open path: {e}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_fs::init())
    .manage(conversion::ConversionState {
        cancel_tokens: Mutex::new(HashMap::new()),
    })
    .invoke_handler(tauri::generate_handler![
      pdf::validate_pdf,
      pdf::get_pdf_metadata,
      conversion::convert_pdf,
      conversion::cancel_conversion,
      storage::import_pdf,
      storage::delete_book,
      storage::get_books_dir,
      storage::save_book_metadata,
      storage::list_books,
      epub_reader::read_epub_preview,
      open_path,
    ])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
