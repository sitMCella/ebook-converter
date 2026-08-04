pub mod text_extractor;
pub mod structure_detector;
pub mod image_extractor;
pub mod chapter_splitter;
pub mod epub_generator;
pub mod css;
pub mod pipeline;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tauri::State;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionOptions {
    pub structure: StructureOptions,
    pub images: ImageOptions,
    pub output: OutputOptions,
    pub page_handling: PageHandlingOptions,
    pub output_folder: String,
    pub book_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct StructureOptions {
    pub detect_headings: bool,
    pub detect_toc: bool,
    pub detect_footnotes: bool,
    pub heading_level_threshold: u8,
    pub paragraph_detection: bool,
    pub list_detection: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageOptions {
    pub extract_images: bool,
    pub image_quality: String,
    pub max_image_width: u32,
    #[serde(alias = "convertToWebP")]
    pub convert_to_webp: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct OutputOptions {
    pub epub_version: String,
    pub embed_fonts: bool,
    pub font_family: String,
    pub base_font_size: u8,
    pub line_height: f32,
    pub margins: f32,
    pub text_alignment: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct PageHandlingOptions {
    pub skip_blank_pages: bool,
    pub page_range: String,
    pub page_range_from: Option<u32>,
    pub page_range_to: Option<u32>,
    pub split_chapters_by: String,
    pub keep_page_breaks: bool,
    pub remove_page_numbers: bool,
    pub cover_page: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConversionResult {
    pub output_path: String,
    pub chapters: usize,
    pub images: usize,
    pub file_size: u64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConversionProgress {
    pub path: String,
    pub stage: String,
    pub percent: u8,
    pub message: String,
}

pub struct ConversionState {
    pub cancel_tokens: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

#[tauri::command]
pub async fn convert_pdf(
    app: tauri::AppHandle,
    state: State<'_, ConversionState>,
    path: String,
    options: ConversionOptions,
) -> Result<ConversionResult, String> {
    let cancel_token = Arc::new(AtomicBool::new(false));
    {
        let mut tokens = state
            .cancel_tokens
            .lock()
            .map_err(|e| format!("Failed to lock state: {}", e))?;
        tokens.insert(path.clone(), cancel_token.clone());
    }

    let result = pipeline::run_conversion(&app, &path, &options, cancel_token.clone()).await;

    {
        let mut tokens = state
            .cancel_tokens
            .lock()
            .map_err(|e| format!("Failed to lock state: {}", e))?;
        tokens.remove(&path);
    }

    result
}

#[tauri::command]
pub fn cancel_conversion(
    state: State<'_, ConversionState>,
    path: String,
) -> Result<(), String> {
    let tokens = state
        .cancel_tokens
        .lock()
        .map_err(|e| format!("Failed to lock state: {}", e))?;
    if let Some(token) = tokens.get(&path) {
        token.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    Ok(())
}
