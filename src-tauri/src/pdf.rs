use lopdf::Document;
use serde::Serialize;
use std::fs;
use std::path::Path;

use crate::conversion::image_extractor::extract_cover_image;

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
pub struct PdfCoverData {
    pub cover_image: Option<String>,
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

#[tauri::command]
pub fn get_pdf_cover(path: String) -> Result<PdfCoverData, String> {
    if let Some(dir) = Path::new(&path).parent() {
        let cached = dir.join("cover.png");
        if cached.exists() {
            let data = fs::read(&cached)
                .map_err(|e| format!("Failed to read cached cover: {}", e))?;
            use base64::Engine;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
            return Ok(PdfCoverData {
                cover_image: Some(format!("data:image/png;base64,{}", b64)),
            });
        }
    }

    let cover = extract_cover_image(&path, "firstPage")?;

    let cover_image = cover.map(|img| {
        use base64::Engine;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&img.data);
        format!("data:{};base64,{}", img.mime_type, b64)
    });

    Ok(PdfCoverData { cover_image })
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

    if let Ok(info_val) = doc.trailer.get(b"Info") {
        let info_dict = resolve_dictionary(&doc, info_val);
        if let Some(dict) = info_dict {
            title = get_string_from_dict(dict, b"Title");
            author = get_string_from_dict(dict, b"Author");
            producer = get_string_from_dict(dict, b"Producer");
            created_date =
                get_string_from_dict(dict, b"CreationDate").and_then(|s| parse_pdf_date(&s));
            modified_date =
                get_string_from_dict(dict, b"ModDate").and_then(|s| parse_pdf_date(&s));
        }
    }

    if title.is_none() {
        if let Some(xmp_title) = extract_xmp_title(&doc) {
            title = Some(xmp_title);
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

fn resolve_dictionary<'a>(doc: &'a Document, obj: &'a lopdf::Object) -> Option<&'a lopdf::Dictionary> {
    match obj {
        lopdf::Object::Dictionary(dict) => Some(dict),
        lopdf::Object::Reference(ref_id) => {
            if let Ok(resolved) = doc.get_object(*ref_id) {
                if let lopdf::Object::Dictionary(dict) = resolved {
                    return Some(dict);
                }
            }
            None
        }
        _ => None,
    }
}

fn extract_xmp_title(doc: &Document) -> Option<String> {
    let root_ref = doc.trailer.get(b"Root").ok()?;
    let catalog = resolve_dictionary(doc, root_ref)?;
    let meta_ref = catalog.get(b"Metadata").ok()?;

    let meta_obj = match meta_ref {
        lopdf::Object::Reference(ref_id) => doc.get_object(*ref_id).ok()?,
        other => other,
    };

    let xmp_bytes = match meta_obj {
        lopdf::Object::Stream(stream) => {
            stream.decompressed_content().unwrap_or_else(|_| stream.content.clone())
        }
        _ => return None,
    };

    let xmp_str = String::from_utf8_lossy(&xmp_bytes);

    extract_xmp_tag(&xmp_str, "dc:title")
}

fn extract_xmp_tag(xmp: &str, tag: &str) -> Option<String> {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);
    let start = xmp.find(&open)?;
    let end = xmp[start..].find(&close)? + start;
    let inner = &xmp[start..end];

    // Look for <rdf:li> content inside the tag
    if let Some(li_start) = inner.find("<rdf:li") {
        let content_start = inner[li_start..].find('>')? + li_start + 1;
        let content_end = inner[content_start..].find("</rdf:li")? + content_start;
        let value = inner[content_start..content_end].trim();
        if value.is_empty() { None } else { Some(value.to_string()) }
    } else {
        // Fallback: content directly inside the tag
        let content_start = inner.find('>')? + 1;
        let value = inner[content_start..].trim();
        if value.is_empty() { None } else { Some(value.to_string()) }
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

    // -- extract_xmp_tag tests --

    #[test]
    fn extract_xmp_tag_with_rdf_li() {
        let xmp = r#"<dc:title><rdf:Alt><rdf:li xml:lang="x-default">My Book Title</rdf:li></rdf:Alt></dc:title>"#;
        assert_eq!(extract_xmp_tag(xmp, "dc:title"), Some("My Book Title".to_string()));
    }

    #[test]
    fn extract_xmp_tag_direct_content() {
        let xmp = r#"<dc:title>Direct Title</dc:title>"#;
        assert_eq!(extract_xmp_tag(xmp, "dc:title"), Some("Direct Title".to_string()));
    }

    #[test]
    fn extract_xmp_tag_missing_tag() {
        let xmp = r#"<dc:creator>Some Author</dc:creator>"#;
        assert_eq!(extract_xmp_tag(xmp, "dc:title"), None);
    }

    #[test]
    fn extract_xmp_tag_empty_content() {
        let xmp = r#"<dc:title><rdf:Alt><rdf:li xml:lang="x-default">   </rdf:li></rdf:Alt></dc:title>"#;
        assert_eq!(extract_xmp_tag(xmp, "dc:title"), None);
    }

    #[test]
    fn extract_xmp_tag_multiline() {
        let xmp = r#"
            <rdf:Description>
                <dc:title>
                    <rdf:Alt>
                        <rdf:li xml:lang="x-default">Multiline Title</rdf:li>
                    </rdf:Alt>
                </dc:title>
            </rdf:Description>
        "#;
        assert_eq!(extract_xmp_tag(xmp, "dc:title"), Some("Multiline Title".to_string()));
    }

    // -- resolve_dictionary tests --

    #[test]
    fn resolve_dictionary_direct() {
        let doc = Document::with_version("1.7");
        let dict = dictionary! { "Key" => "Value" };
        let obj = Object::Dictionary(dict);
        let result = resolve_dictionary(&doc, &obj);
        assert!(result.is_some());
    }

    #[test]
    fn resolve_dictionary_indirect() {
        let mut doc = Document::with_version("1.7");
        let dict = dictionary! { "Key" => "Value" };
        let id = doc.add_object(Object::Dictionary(dict));
        let ref_obj = Object::Reference(id);
        let result = resolve_dictionary(&doc, &ref_obj);
        assert!(result.is_some());
    }

    #[test]
    fn resolve_dictionary_non_dict() {
        let doc = Document::with_version("1.7");
        let obj = Object::Integer(42);
        let result = resolve_dictionary(&doc, &obj);
        assert!(result.is_none());
    }

    // -- XMP fallback integration test --

    #[test]
    fn get_pdf_metadata_falls_back_to_xmp_title() {
        let mut doc = Document::with_version("2.0");
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

        let xmp_content = br#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    <rdf:Description xmlns:dc="http://purl.org/dc/elements/1.1/">
      <dc:title>
        <rdf:Alt>
          <rdf:li xml:lang="x-default">XMP Only Title</rdf:li>
        </rdf:Alt>
      </dc:title>
    </rdf:Description>
  </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#;

        let xmp_stream = Stream::new(
            dictionary! { "Type" => "Metadata", "Subtype" => "XML" },
            xmp_content.to_vec(),
        );
        let xmp_id = doc.add_object(xmp_stream);

        let root_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
            "Metadata" => xmp_id,
        });
        doc.trailer.set("Root", root_id);

        let file = NamedTempFile::new().unwrap();
        doc.save(file.path()).unwrap();

        let metadata = get_pdf_metadata(file.path().to_str().unwrap().to_string()).unwrap();
        assert_eq!(metadata.title.as_deref(), Some("XMP Only Title"));
    }

    #[test]
    fn get_pdf_metadata_prefers_info_title_over_xmp() {
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

        let info = dictionary! {
            "Title" => Object::String(b"Info Title".to_vec(), lopdf::StringFormat::Literal),
        };
        let info_id = doc.add_object(info);
        doc.trailer.set("Info", info_id);

        let xmp_content = b"<dc:title><rdf:Alt><rdf:li xml:lang=\"x-default\">XMP Title</rdf:li></rdf:Alt></dc:title>";
        let xmp_stream = Stream::new(
            dictionary! { "Type" => "Metadata", "Subtype" => "XML" },
            xmp_content.to_vec(),
        );
        let xmp_id = doc.add_object(xmp_stream);

        let root_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
            "Metadata" => xmp_id,
        });
        doc.trailer.set("Root", root_id);

        let file = NamedTempFile::new().unwrap();
        doc.save(file.path()).unwrap();

        let metadata = get_pdf_metadata(file.path().to_str().unwrap().to_string()).unwrap();
        assert_eq!(metadata.title.as_deref(), Some("Info Title"));
    }

    // -- get_pdf_cover caching tests --

    fn create_pdf_with_image(dir: &std::path::Path, filename: &str, width: u32, height: u32) -> std::path::PathBuf {
        let mut doc = Document::with_version("1.7");
        let pages_id = doc.new_object_id();
        let page_id = doc.new_object_id();

        let img = image::RgbImage::from_pixel(width, height, image::Rgb([100, 150, 200]));
        let dynamic = image::DynamicImage::ImageRgb8(img);
        let mut buf = std::io::Cursor::new(Vec::new());
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 50);
        dynamic.write_with_encoder(encoder).unwrap();
        let jpeg_data = buf.into_inner();

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

    #[test]
    fn get_pdf_cover_returns_cached_png_when_present() {
        let dir = tempfile::tempdir().unwrap();
        let pdf_path = create_pdf_with_image(dir.path(), "source.pdf", 400, 500);

        let img = image::RgbImage::from_pixel(10, 10, image::Rgb([255, 0, 0]));
        let cover_path = dir.path().join("cover.png");
        img.save(&cover_path).unwrap();

        let result = get_pdf_cover(pdf_path.to_str().unwrap().to_string()).unwrap();
        assert!(result.cover_image.is_some());
        let data_uri = result.cover_image.unwrap();
        assert!(data_uri.starts_with("data:image/png;base64,"));
    }

    #[test]
    fn get_pdf_cover_falls_back_to_extraction_without_cache() {
        let dir = tempfile::tempdir().unwrap();
        let pdf_path = create_pdf_with_image(dir.path(), "source.pdf", 400, 500);

        let cached = dir.path().join("cover.png");
        assert!(!cached.exists());

        let result = get_pdf_cover(pdf_path.to_str().unwrap().to_string()).unwrap();
        assert!(result.cover_image.is_some());
        let data_uri = result.cover_image.unwrap();
        assert!(data_uri.starts_with("data:image/jpeg;base64,"));
    }

    #[test]
    fn get_pdf_cover_returns_none_when_no_cache_and_no_images() {
        let file = create_valid_pdf();
        let result = get_pdf_cover(file.path().to_str().unwrap().to_string()).unwrap();
        assert!(result.cover_image.is_none());
    }

    #[test]
    fn get_pdf_cover_prefers_cache_over_extraction() {
        let dir = tempfile::tempdir().unwrap();
        let pdf_path = create_pdf_with_image(dir.path(), "source.pdf", 400, 500);

        let sentinel = image::RgbImage::from_pixel(1, 1, image::Rgb([0, 0, 0]));
        let cover_path = dir.path().join("cover.png");
        sentinel.save(&cover_path).unwrap();
        let cached_size = std::fs::metadata(&cover_path).unwrap().len();

        let result = get_pdf_cover(pdf_path.to_str().unwrap().to_string()).unwrap();
        let data_uri = result.cover_image.unwrap();

        use base64::Engine;
        let b64_part = data_uri.strip_prefix("data:image/png;base64,").unwrap();
        let decoded = base64::engine::general_purpose::STANDARD.decode(b64_part).unwrap();
        assert_eq!(decoded.len() as u64, cached_size);
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

}
