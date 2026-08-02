use super::PageHandlingOptions;

pub fn extract_text(
    path: &str,
    options: &PageHandlingOptions,
) -> Result<Vec<String>, String> {
    let text = pdf_extract::extract_text(path)
        .map_err(|e| format!("Failed to extract text: {}", e))?;

    let mut pages: Vec<String> = text.split('\x0C').map(|s| s.to_string()).collect();

    if pages.last().map_or(false, |p| p.trim().is_empty()) {
        pages.pop();
    }

    if options.page_range == "custom" {
        let from = options.page_range_from.unwrap_or(1).max(1) as usize;
        let to = options
            .page_range_to
            .map(|t| t as usize)
            .unwrap_or(pages.len())
            .min(pages.len());

        if from > to || from > pages.len() {
            return Ok(Vec::new());
        }

        pages = pages[(from - 1)..to].to_vec();
    }

    if options.skip_blank_pages {
        pages.retain(|p| !p.trim().is_empty());
    }

    Ok(pages)
}
