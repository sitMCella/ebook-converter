use serde::Serialize;
use std::io::Read;
use zip::ZipArchive;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpubPreviewData {
    pub cover_image: Option<String>,
}

#[tauri::command]
pub fn read_epub_preview(path: String) -> Result<EpubPreviewData, String> {
    let file = std::fs::File::open(&path)
        .map_err(|e| format!("Failed to open EPUB: {}", e))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("Failed to read EPUB archive: {}", e))?;

    let cover_image = extract_cover_image_data(&mut archive);

    Ok(EpubPreviewData { cover_image })
}

fn extract_cover_image_data(archive: &mut ZipArchive<std::fs::File>) -> Option<String> {
    let image_names: Vec<String> = (0..archive.len())
        .filter_map(|i| {
            let entry = archive.by_index(i).ok()?;
            let name = entry.name().to_string();
            if name.contains("cover") && is_image_ext(&name) {
                Some(name)
            } else {
                None
            }
        })
        .collect();

    let name = image_names.first()?;

    let mut file = archive.by_name(name).ok()?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).ok()?;

    let mime = if name.ends_with(".png") {
        "image/png"
    } else if name.ends_with(".webp") {
        "image/webp"
    } else {
        "image/jpeg"
    };

    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
    Some(format!("data:{};base64,{}", mime, b64))
}

fn is_image_ext(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".jpg") || lower.ends_with(".jpeg") || lower.ends_with(".png") || lower.ends_with(".webp")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_image_ext_matches_images() {
        assert!(is_image_ext("cover.jpg"));
        assert!(is_image_ext("img/COVER.PNG"));
        assert!(is_image_ext("photo.webp"));
        assert!(!is_image_ext("chapter.xhtml"));
    }
}
