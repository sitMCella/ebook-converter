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

#[derive(Debug, Serialize)]
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

pub fn get_pdf_metadata_internal(path: &str) -> Result<PdfMetadata, String> {
    get_pdf_metadata(path.to_string())
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

pub fn extract_outline(path: &str) -> Vec<crate::conversion::TocEntry> {
    let doc = match Document::load(path) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    let catalog = match doc.catalog() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let outlines_ref = match catalog.get(b"Outlines") {
        Ok(obj) => match doc.dereference(obj) {
            Ok((_, obj)) => obj.clone(),
            Err(_) => return Vec::new(),
        },
        Err(_) => return Vec::new(),
    };

    let outlines_dict = match outlines_ref {
        lopdf::Object::Dictionary(d) => d,
        _ => return Vec::new(),
    };

    let mut entries = Vec::new();
    if let Ok(first) = outlines_dict.get(b"First") {
        if let Ok(first_id) = first.as_reference() {
            collect_outline_items(&doc, first_id, 1, &mut entries);
        }
    }

    entries
}

fn collect_outline_items(
    doc: &Document,
    item_id: lopdf::ObjectId,
    level: u8,
    entries: &mut Vec<crate::conversion::TocEntry>,
) {
    let dict = match doc.get_dictionary(item_id) {
        Ok(d) => d,
        Err(_) => return,
    };

    if let Some(title) = get_string_from_dict(dict, b"Title") {
        entries.push(crate::conversion::TocEntry { title, level });
    }

    if let Ok(first_child) = dict.get(b"First") {
        if let Ok(child_id) = first_child.as_reference() {
            collect_outline_items(doc, child_id, level + 1, entries);
        }
    }

    if let Ok(next) = dict.get(b"Next") {
        if let Ok(next_id) = next.as_reference() {
            collect_outline_items(doc, next_id, level, entries);
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::dictionary;
    use lopdf::{Document, Object, Stream};
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_valid_pdf() -> NamedTempFile {
        let mut doc = Document::with_version("1.7");
        let pages_id = doc.new_object_id();
        let page_id = doc.new_object_id();

        let content = Stream::new(dictionary! {}, b"BT /F1 12 Tf 100 700 Td (Hello) Tj ET".to_vec());
        let content_id = doc.add_object(content);

        let font_dict = dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
        };
        let font_id = doc.add_object(font_dict);

        let resources = dictionary! {
            "Font" => dictionary! {
                "F1" => font_id,
            },
        };

        let page = dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Contents" => content_id,
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

        let file = NamedTempFile::new().unwrap();
        doc.save(file.path()).unwrap();
        file
    }

    fn create_valid_pdf_with_metadata() -> NamedTempFile {
        let mut doc = Document::with_version("1.7");
        let pages_id = doc.new_object_id();
        let page_id = doc.new_object_id();
        let page2_id = doc.new_object_id();

        let content = Stream::new(dictionary! {}, b"BT /F1 12 Tf 100 700 Td (Hello) Tj ET".to_vec());
        let content_id = doc.add_object(content);

        let font_dict = dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
        };
        let font_id = doc.add_object(font_dict);

        let resources = dictionary! {
            "Font" => dictionary! {
                "F1" => font_id,
            },
        };

        let page = dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Contents" => content_id,
            "Resources" => resources,
        };
        doc.objects.insert(page_id, Object::Dictionary(page));

        let page2 = dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
        };
        doc.objects.insert(page2_id, Object::Dictionary(page2));

        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into(), page2_id.into()],
            "Count" => Object::Integer(2),
        };
        doc.objects.insert(pages_id, Object::Dictionary(pages));

        let root_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", root_id);

        let info = dictionary! {
            "Title" => Object::String(b"Test Document".to_vec(), lopdf::StringFormat::Literal),
            "Author" => Object::String(b"Test Author".to_vec(), lopdf::StringFormat::Literal),
            "Producer" => Object::String(b"Test Producer".to_vec(), lopdf::StringFormat::Literal),
            "CreationDate" => Object::String(b"D:20240115120000".to_vec(), lopdf::StringFormat::Literal),
            "ModDate" => Object::String(b"D:20240620153045".to_vec(), lopdf::StringFormat::Literal),
        };
        let info_id = doc.add_object(info);
        doc.trailer.set("Info", info_id);

        let file = NamedTempFile::new().unwrap();
        doc.save(file.path()).unwrap();
        file
    }

    fn create_corrupted_file() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"This is not a valid PDF file at all").unwrap();
        file.flush().unwrap();
        file
    }

    // -- validate_pdf tests --

    #[test]
    fn validate_pdf_with_valid_file() {
        let file = create_valid_pdf();
        let result = validate_pdf(file.path().to_str().unwrap().to_string()).unwrap();
        match result {
            PdfValidation::Valid => {}
            _ => panic!("Expected Valid, got {:?}", serde_json::to_string(&result).unwrap()),
        }
    }

    #[test]
    fn validate_pdf_with_corrupted_file() {
        let file = create_corrupted_file();
        let result = validate_pdf(file.path().to_str().unwrap().to_string()).unwrap();
        match result {
            PdfValidation::Error { ref message } => {
                assert!(message.contains("could not be read"), "Expected error message, got: {}", message);
            }
            _ => panic!("Expected Error, got {:?}", serde_json::to_string(&result).unwrap()),
        }
    }

    #[test]
    fn validate_pdf_with_nonexistent_file() {
        let result = validate_pdf("/nonexistent/path/fake.pdf".to_string()).unwrap();
        match result {
            PdfValidation::Error { ref message } => {
                assert_eq!(message, "File not found");
            }
            _ => panic!("Expected Error for nonexistent file"),
        }
    }

    // -- get_pdf_metadata tests --

    #[test]
    fn get_pdf_metadata_extracts_page_count() {
        let file = create_valid_pdf();
        let metadata = get_pdf_metadata(file.path().to_str().unwrap().to_string()).unwrap();
        assert_eq!(metadata.page_count, 1);
    }

    #[test]
    fn get_pdf_metadata_extracts_version() {
        let file = create_valid_pdf();
        let metadata = get_pdf_metadata(file.path().to_str().unwrap().to_string()).unwrap();
        assert_eq!(metadata.pdf_version, "1.7");
    }

    #[test]
    fn get_pdf_metadata_extracts_file_size() {
        let file = create_valid_pdf();
        let metadata = get_pdf_metadata(file.path().to_str().unwrap().to_string()).unwrap();
        assert!(metadata.file_size > 0);
    }

    #[test]
    fn get_pdf_metadata_with_full_metadata() {
        let file = create_valid_pdf_with_metadata();
        let metadata = get_pdf_metadata(file.path().to_str().unwrap().to_string()).unwrap();
        assert_eq!(metadata.title.as_deref(), Some("Test Document"));
        assert_eq!(metadata.author.as_deref(), Some("Test Author"));
        assert_eq!(metadata.producer.as_deref(), Some("Test Producer"));
        assert_eq!(metadata.page_count, 2);
    }

    #[test]
    fn get_pdf_metadata_with_missing_metadata() {
        let file = create_valid_pdf();
        let metadata = get_pdf_metadata(file.path().to_str().unwrap().to_string()).unwrap();
        assert!(metadata.title.is_none());
        assert!(metadata.author.is_none());
        assert!(metadata.producer.is_none());
        assert!(metadata.created_date.is_none());
        assert!(metadata.modified_date.is_none());
    }

    #[test]
    fn get_pdf_metadata_parses_dates() {
        let file = create_valid_pdf_with_metadata();
        let metadata = get_pdf_metadata(file.path().to_str().unwrap().to_string()).unwrap();
        assert_eq!(metadata.created_date.as_deref(), Some("2024-01-15T12:00:00Z"));
        assert_eq!(metadata.modified_date.as_deref(), Some("2024-06-20T15:30:45Z"));
    }

    #[test]
    fn get_pdf_metadata_errors_on_invalid_file() {
        let result = get_pdf_metadata("/nonexistent/path/fake.pdf".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to load PDF"));
    }

    // -- parse_pdf_date tests --

    #[test]
    fn parse_pdf_date_full_format() {
        assert_eq!(
            parse_pdf_date("D:20240115120000"),
            Some("2024-01-15T12:00:00Z".to_string())
        );
    }

    #[test]
    fn parse_pdf_date_without_prefix() {
        assert_eq!(
            parse_pdf_date("20240115120000"),
            Some("2024-01-15T12:00:00Z".to_string())
        );
    }

    #[test]
    fn parse_pdf_date_year_only() {
        assert_eq!(
            parse_pdf_date("D:2024"),
            Some("2024-01-01T00:00:00Z".to_string())
        );
    }

    #[test]
    fn parse_pdf_date_year_and_month() {
        assert_eq!(
            parse_pdf_date("D:202406"),
            Some("2024-06-01T00:00:00Z".to_string())
        );
    }

    #[test]
    fn parse_pdf_date_too_short() {
        assert_eq!(parse_pdf_date("D:20"), None);
        assert_eq!(parse_pdf_date(""), None);
    }

    #[test]
    fn parse_pdf_date_with_timezone() {
        assert_eq!(
            parse_pdf_date("D:20240115120000+05'30'"),
            Some("2024-01-15T12:00:00Z".to_string())
        );
    }

    // -- get_string_from_dict tests --

    #[test]
    fn get_string_from_dict_returns_string() {
        let dict = dictionary! {
            "Title" => Object::String(b"Hello World".to_vec(), lopdf::StringFormat::Literal),
        };
        assert_eq!(get_string_from_dict(&dict, b"Title"), Some("Hello World".to_string()));
    }

    #[test]
    fn get_string_from_dict_returns_none_for_missing_key() {
        let dict = dictionary! {};
        assert_eq!(get_string_from_dict(&dict, b"Title"), None);
    }

    #[test]
    fn get_string_from_dict_returns_none_for_empty_string() {
        let dict = dictionary! {
            "Title" => Object::String(b"".to_vec(), lopdf::StringFormat::Literal),
        };
        assert_eq!(get_string_from_dict(&dict, b"Title"), None);
    }

    #[test]
    fn get_string_from_dict_returns_none_for_whitespace_string() {
        let dict = dictionary! {
            "Title" => Object::String(b"   ".to_vec(), lopdf::StringFormat::Literal),
        };
        assert_eq!(get_string_from_dict(&dict, b"Title"), None);
    }

    #[test]
    fn get_string_from_dict_trims_whitespace() {
        let dict = dictionary! {
            "Title" => Object::String(b"  Hello  ".to_vec(), lopdf::StringFormat::Literal),
        };
        assert_eq!(get_string_from_dict(&dict, b"Title"), Some("Hello".to_string()));
    }

    #[test]
    fn get_string_from_dict_returns_none_for_non_string_object() {
        let dict = dictionary! {
            "Count" => Object::Integer(42),
        };
        assert_eq!(get_string_from_dict(&dict, b"Count"), None);
    }

    #[test]
    fn get_string_from_dict_handles_utf16_bom() {
        let mut bytes = vec![0xFE, 0xFF]; // UTF-16 BE BOM
        bytes.extend_from_slice(&[0x00, 0x48, 0x00, 0x69]); // "Hi" in UTF-16 BE
        let dict = dictionary! {
            "Title" => Object::String(bytes, lopdf::StringFormat::Literal),
        };
        assert_eq!(get_string_from_dict(&dict, b"Title"), Some("Hi".to_string()));
    }

    // -- PdfValidation serialization tests --

    #[test]
    fn pdf_validation_serializes_valid() {
        let json = serde_json::to_string(&PdfValidation::Valid).unwrap();
        assert_eq!(json, r#"{"status":"valid"}"#);
    }

    #[test]
    fn pdf_validation_serializes_encrypted() {
        let json = serde_json::to_string(&PdfValidation::Encrypted).unwrap();
        assert_eq!(json, r#"{"status":"encrypted"}"#);
    }

    #[test]
    fn pdf_validation_serializes_error() {
        let json = serde_json::to_string(&PdfValidation::Error { message: "bad".to_string() }).unwrap();
        assert_eq!(json, r#"{"status":"error","message":"bad"}"#);
    }

    // -- PdfMetadata serialization tests --

    #[test]
    fn pdf_metadata_serializes_to_camel_case() {
        let metadata = PdfMetadata {
            title: Some("Test".to_string()),
            author: None,
            page_count: 5,
            pdf_version: "1.7".to_string(),
            created_date: None,
            modified_date: None,
            producer: None,
            file_size: 1024,
        };
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"pageCount\":5"));
        assert!(json.contains("\"pdfVersion\":\"1.7\""));
        assert!(json.contains("\"fileSize\":1024"));
    }

    // -- extract_outline tests --

    fn create_pdf_with_outlines() -> NamedTempFile {
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

        let item2_id = doc.new_object_id();
        let item1_id = doc.new_object_id();

        let item2 = dictionary! {
            "Title" => Object::String(b"Chapter 2: Advanced Topics".to_vec(), lopdf::StringFormat::Literal),
            "Parent" => item1_id, // just needs a parent ref, not accurate but enough for traversal
        };
        doc.objects.insert(item2_id, Object::Dictionary(item2));

        let item1 = dictionary! {
            "Title" => Object::String(b"Chapter 1: Getting Started".to_vec(), lopdf::StringFormat::Literal),
            "Next" => Object::Reference(item2_id),
        };
        doc.objects.insert(item1_id, Object::Dictionary(item1));

        let outlines_id = doc.add_object(dictionary! {
            "Type" => "Outlines",
            "First" => Object::Reference(item1_id),
            "Last" => Object::Reference(item2_id),
            "Count" => Object::Integer(2),
        });

        let root_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
            "Outlines" => Object::Reference(outlines_id),
        });
        doc.trailer.set("Root", root_id);

        let file = NamedTempFile::new().unwrap();
        doc.save(file.path()).unwrap();
        file
    }

    fn create_pdf_with_nested_outlines() -> NamedTempFile {
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

        let child_id = doc.new_object_id();
        let parent_item_id = doc.new_object_id();

        let child = dictionary! {
            "Title" => Object::String(b"Section 1.1".to_vec(), lopdf::StringFormat::Literal),
            "Parent" => parent_item_id,
        };
        doc.objects.insert(child_id, Object::Dictionary(child));

        let parent_item = dictionary! {
            "Title" => Object::String(b"Chapter 1".to_vec(), lopdf::StringFormat::Literal),
            "First" => Object::Reference(child_id),
            "Last" => Object::Reference(child_id),
        };
        doc.objects.insert(parent_item_id, Object::Dictionary(parent_item));

        let outlines_id = doc.add_object(dictionary! {
            "Type" => "Outlines",
            "First" => Object::Reference(parent_item_id),
            "Last" => Object::Reference(parent_item_id),
            "Count" => Object::Integer(2),
        });

        let root_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
            "Outlines" => Object::Reference(outlines_id),
        });
        doc.trailer.set("Root", root_id);

        let file = NamedTempFile::new().unwrap();
        doc.save(file.path()).unwrap();
        file
    }

    #[test]
    fn extract_outline_from_pdf_with_bookmarks() {
        let file = create_pdf_with_outlines();
        let entries = extract_outline(file.path().to_str().unwrap());
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].title, "Chapter 1: Getting Started");
        assert_eq!(entries[0].level, 1);
        assert_eq!(entries[1].title, "Chapter 2: Advanced Topics");
        assert_eq!(entries[1].level, 1);
    }

    #[test]
    fn extract_outline_returns_empty_for_no_outlines() {
        let file = create_valid_pdf();
        let entries = extract_outline(file.path().to_str().unwrap());
        assert!(entries.is_empty());
    }

    #[test]
    fn extract_outline_returns_empty_for_invalid_path() {
        let entries = extract_outline("/nonexistent/path.pdf");
        assert!(entries.is_empty());
    }

    #[test]
    fn extract_outline_handles_nested_entries() {
        let file = create_pdf_with_nested_outlines();
        let entries = extract_outline(file.path().to_str().unwrap());
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].title, "Chapter 1");
        assert_eq!(entries[0].level, 1);
        assert_eq!(entries[1].title, "Section 1.1");
        assert_eq!(entries[1].level, 2);
    }
}
