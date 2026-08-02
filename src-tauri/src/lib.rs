mod pdf;
mod conversion;

use std::collections::HashMap;
use std::sync::Mutex;

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
