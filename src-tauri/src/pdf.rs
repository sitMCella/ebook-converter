use lopdf::Document;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
#[serde(tag = "status")]
pub enum PdfValidation {
    #[serde(rename = "valid")]
    Valid,
    #[serde(rename = "encrypted")]
    Encrypted,
    #[serde(rename = "error")]
    Error { message: String },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub page_count: usize,
    pub pdf_version: String,
    pub created_date: Option<String>,
    pub modified_date: Option<String>,
    pub producer: Option<String>,
    pub file_size: u64,
}

#[tauri::command]
pub fn validate_pdf(path: String) -> Result<PdfValidation, String> {
    let file_path = Path::new(&path);
    if !file_path.exists() {
        return Ok(PdfValidation::Error {
            message: "File not found".to_string(),
        });
    }

    match Document::load(&path) {
        Ok(doc) => {
            if doc.is_encrypted() {
                Ok(PdfValidation::Encrypted)
            } else {
                Ok(PdfValidation::Valid)
            }
        }
        Err(e) => {
            let err_str = e.to_string().to_lowercase();
            if err_str.contains("encrypt") || err_str.contains("password") {
                Ok(PdfValidation::Encrypted)
            } else {
                Ok(PdfValidation::Error {
                    message: format!("This file could not be read: {}", e),
                })
            }
        }
    }
}

#[tauri::command]
pub fn get_pdf_metadata(path: String) -> Result<PdfMetadata, String> {
    let doc = Document::load(&path).map_err(|e| format!("Failed to load PDF: {}", e))?;

    let page_count = doc.get_pages().len();
    let pdf_version = doc.version.clone();

    let file_size = fs::metadata(&path)
        .map(|m| m.len())
        .unwrap_or(0);

    let mut title = None;
    let mut author = None;
    let mut created_date = None;
    let mut modified_date = None;
    let mut producer = None;

    if let Ok(info_ref) = doc.trailer.get(b"Info") {
        if let Ok(info_obj) = doc.get_object(info_ref.as_reference().unwrap_or_default()) {
            if let lopdf::Object::Dictionary(ref dict) = *info_obj {
                title = get_string_from_dict(dict, b"Title");
                author = get_string_from_dict(dict, b"Author");
                producer = get_string_from_dict(dict, b"Producer");
                created_date =
                    get_string_from_dict(dict, b"CreationDate").and_then(|s| parse_pdf_date(&s));
                modified_date =
                    get_string_from_dict(dict, b"ModDate").and_then(|s| parse_pdf_date(&s));
            }
        }
    }

    Ok(PdfMetadata {
        title,
        author,
        page_count,
        pdf_version,
        created_date,
        modified_date,
        producer,
        file_size,
    })
}

fn get_string_from_dict(dict: &lopdf::Dictionary, key: &[u8]) -> Option<String> {
    dict.get(key).ok().and_then(|obj| match obj {
        lopdf::Object::String(bytes, _) => {
            let s = if bytes.starts_with(&[0xFE, 0xFF]) {
                // UTF-16 BE with BOM
                let chars: Vec<u16> = bytes[2..]
                    .chunks(2)
                    .map(|chunk| u16::from_be_bytes([chunk[0], *chunk.get(1).unwrap_or(&0)]))
                    .collect();
                String::from_utf16_lossy(&chars)
            } else {
                String::from_utf8_lossy(bytes).into_owned()
            };
            let trimmed = s.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        }
        _ => None,
    })
}

fn parse_pdf_date(date_str: &str) -> Option<String> {
    // PDF date format: D:YYYYMMDDHHmmSSOHH'mm'
    let s = date_str.strip_prefix("D:").unwrap_or(date_str);
    if s.len() < 4 {
        return None;
    }

    let year = s.get(0..4)?;
    let month = s.get(4..6).unwrap_or("01");
    let day = s.get(6..8).unwrap_or("01");
    let hour = s.get(8..10).unwrap_or("00");
    let minute = s.get(10..12).unwrap_or("00");
    let second = s.get(12..14).unwrap_or("00");

    Some(format!(
        "{}-{}-{}T{}:{}:{}Z",
        year, month, day, hour, minute, second
    ))
}
