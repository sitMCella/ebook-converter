use super::ImageOptions;
use lopdf::{Document, Object};

#[allow(dead_code)]
pub struct ExtractedImage {
    pub id: String,
    pub data: Vec<u8>,
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
}

pub fn extract_cover_image(
    path: &str,
    cover_mode: &str,
) -> Result<Option<ExtractedImage>, String> {
    if cover_mode == "none" {
        return Ok(None);
    }

    let doc = Document::load(path)
        .map_err(|e| format!("Failed to load PDF for cover extraction: {}", e))?;

    let pages = doc.get_pages();
    let first_page_id = match pages.get(&1) {
        Some(id) => *id,
        None => return Ok(None),
    };

    let resources = match get_page_resources(&doc, first_page_id) {
        Some(r) => r,
        None => return Ok(None),
    };

    let xobjects = match get_xobjects(&doc, &resources) {
        Some(x) => x,
        None => return Ok(None),
    };

    let mut best_image: Option<ExtractedImage> = None;
    let mut best_area: u64 = 0;
    let mut image_counter = 0u32;

    let cover_options = ImageOptions {
        extract_images: true,
        image_quality: "high".to_string(),
        max_image_width: u32::MAX,
        convert_to_webp: false,
    };

    for (name, obj_ref) in &xobjects {
        let object = match resolve_object(&doc, obj_ref) {
            Some(o) => o,
            None => continue,
        };

        if !is_image_xobject(&doc, &object) {
            continue;
        }

        let stream = match object.as_stream() {
            Ok(s) => s,
            Err(_) => continue,
        };

        image_counter += 1;
        let name_str = String::from_utf8_lossy(name).to_string();
        let image_id = format!("cover_{}", name_str);

        match extract_single_image(stream, &image_id, image_counter, &cover_options) {
            Ok(img) => {
                let area = img.width as u64 * img.height as u64;
                if area > best_area {
                    best_area = area;
                    best_image = Some(img);
                }
            }
            Err(e) => {
                log::warn!("Skipping potential cover image {}: {}", name_str, e);
            }
        }
    }

    match cover_mode {
        "firstPage" => Ok(best_image),
        "auto" => {
            if let Some(ref img) = best_image {
                if img.width >= 300 && img.height >= 400 {
                    Ok(best_image)
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        }
        _ => Ok(None),
    }
}

pub fn extract_images(
    path: &str,
    options: &ImageOptions,
) -> Result<Vec<ExtractedImage>, String> {
    if !options.extract_images {
        return Ok(Vec::new());
    }

    let doc = Document::load(path)
        .map_err(|e| format!("Failed to load PDF for image extraction: {}", e))?;

    let mut images = Vec::new();
    let mut image_counter = 0u32;

    for (page_num, page_id) in doc.get_pages() {
        let resources = match get_page_resources(&doc, page_id) {
            Some(r) => r,
            None => continue,
        };

        let xobjects = match get_xobjects(&doc, &resources) {
            Some(x) => x,
            None => continue,
        };

        for (name, obj_ref) in &xobjects {
            let object = match resolve_object(&doc, obj_ref) {
                Some(o) => o,
                None => continue,
            };

            if !is_image_xobject(&doc, &object) {
                continue;
            }

            let stream = match object.as_stream() {
                Ok(s) => s,
                Err(_) => continue,
            };

            image_counter += 1;
            let name_str = String::from_utf8_lossy(name).to_string();
            let image_id = format!("img_p{}_{}", page_num, name_str);

            match extract_single_image(stream, &image_id, image_counter, options) {
                Ok(img) => images.push(img),
                Err(e) => {
                    log::warn!("Skipping image {} on page {}: {}", name_str, page_num, e);
                }
            }
        }
    }

    Ok(images)
}

pub fn render_cover_page(path: &str) -> Result<Option<image::DynamicImage>, String> {
    let doc = Document::load(path)
        .map_err(|e| format!("Failed to load PDF for cover rendering: {}", e))?;

    let pages = doc.get_pages();
    let first_page_id = match pages.get(&1) {
        Some(id) => *id,
        None => return Ok(None),
    };

    let page = doc
        .get_object(first_page_id)
        .map_err(|e| format!("Failed to get page: {}", e))?;
    let page_dict = page
        .as_dict()
        .map_err(|e| format!("Page is not a dictionary: {}", e))?;

    let (x0, y0, x1, y1) = page_media_box(&doc, page_dict)?;
    let page_w = x1 - x0;
    let page_h = y1 - y0;
    if page_w <= 0.0 || page_h <= 0.0 {
        return Ok(None);
    }

    let content_bytes = match page_content_bytes(&doc, page_dict) {
        Ok(b) => b,
        Err(_) => return Ok(None),
    };
    let content = match lopdf::content::Content::decode(&content_bytes) {
        Ok(c) => c,
        Err(_) => return Ok(None),
    };

    let resources = match get_page_resources(&doc, first_page_id) {
        Some(r) => r,
        None => return Ok(None),
    };
    let xobjects = match get_xobjects(&doc, &resources) {
        Some(x) => x,
        None => return Ok(None),
    };

    let placements = find_image_placements(&content, &doc, &xobjects);
    if placements.len() < 2 {
        return Ok(None);
    }

    let cover_options = ImageOptions {
        extract_images: true,
        image_quality: "high".to_string(),
        max_image_width: u32::MAX,
        convert_to_webp: false,
    };

    let mut scale = 2.0_f64;
    for (_, ctm, stream) in &placements {
        let img_w = get_dict_int(&stream.dict, b"Width").unwrap_or(0) as f64;
        let img_h = get_dict_int(&stream.dict, b"Height").unwrap_or(0) as f64;
        let a = ctm[0].abs();
        let d = ctm[3].abs();
        if a > 0.0 && img_w > 0.0 {
            scale = scale.max(img_w / a);
        }
        if d > 0.0 && img_h > 0.0 {
            scale = scale.max(img_h / d);
        }
    }
    scale = scale.min(4.0);

    let canvas_w = (page_w * scale).round() as u32;
    let canvas_h = (page_h * scale).round() as u32;
    if canvas_w == 0 || canvas_h == 0 || canvas_w > 8000 || canvas_h > 12000 {
        return Ok(None);
    }

    let mut canvas = image::RgbaImage::new(canvas_w, canvas_h);
    for pixel in canvas.pixels_mut() {
        *pixel = image::Rgba([255, 255, 255, 255]);
    }

    let mut painted = false;
    let mut counter = 0u32;
    for (name, ctm, stream) in &placements {
        counter += 1;
        let name_str = String::from_utf8_lossy(name).to_string();
        let image_id = format!("comp_{}", name_str);

        let extracted = match extract_single_image(stream, &image_id, counter, &cover_options) {
            Ok(img) => img,
            Err(_) => continue,
        };

        let src_img = match image::load_from_memory(&extracted.data) {
            Ok(img) => img,
            Err(_) => continue,
        };

        let [a, _b, _c, d, e, f] = *ctm;
        let dest_w = (a.abs() * scale).round() as u32;
        let dest_h = (d.abs() * scale).round() as u32;
        if dest_w == 0 || dest_h == 0 {
            continue;
        }

        let dest_x = ((e - x0) * scale).round() as i64;
        let dest_y = ((page_h - (f - y0) - d.abs()) * scale).round() as i64;

        let resized = src_img.resize_exact(
            dest_w,
            dest_h,
            image::imageops::FilterType::Lanczos3,
        );
        image::imageops::overlay(&mut canvas, &resized.to_rgba8(), dest_x, dest_y);
        painted = true;
    }

    if painted {
        Ok(Some(image::DynamicImage::ImageRgba8(canvas)))
    } else {
        Ok(None)
    }
}

fn page_media_box(
    doc: &Document,
    page_dict: &lopdf::Dictionary,
) -> Result<(f64, f64, f64, f64), String> {
    let mb = page_dict
        .get(b"MediaBox")
        .map_err(|_| "No MediaBox".to_string())?;
    let resolved = match mb {
        Object::Reference(r) => doc.get_object(*r).map_err(|e| format!("{}", e))?,
        other => other,
    };
    let arr = resolved
        .as_array()
        .map_err(|_| "MediaBox not an array".to_string())?;
    if arr.len() < 4 {
        return Err("MediaBox has fewer than 4 elements".to_string());
    }
    let vals: Vec<f64> = arr
        .iter()
        .filter_map(|v| match v {
            Object::Integer(i) => Some(*i as f64),
            Object::Real(r) => Some(*r as f64),
            _ => None,
        })
        .collect();
    if vals.len() < 4 {
        return Err("MediaBox values not numeric".to_string());
    }
    Ok((vals[0], vals[1], vals[2], vals[3]))
}

fn stream_bytes(stream: &lopdf::Stream) -> Vec<u8> {
    stream
        .decompressed_content()
        .unwrap_or_else(|_| stream.content.clone())
}

fn page_content_bytes(
    doc: &Document,
    page_dict: &lopdf::Dictionary,
) -> Result<Vec<u8>, String> {
    let contents = page_dict
        .get(b"Contents")
        .map_err(|_| "No Contents".to_string())?;
    match contents {
        Object::Reference(r) => {
            let obj = doc.get_object(*r).map_err(|e| format!("{}", e))?;
            let stream = obj.as_stream().map_err(|e| format!("{}", e))?;
            Ok(stream_bytes(stream))
        }
        Object::Array(arr) => {
            let mut bytes = Vec::new();
            for item in arr {
                let r = match item {
                    Object::Reference(r) => r,
                    _ => continue,
                };
                if let Ok(obj) = doc.get_object(*r) {
                    if let Ok(stream) = obj.as_stream() {
                        bytes.extend_from_slice(&stream_bytes(stream));
                        bytes.push(b' ');
                    }
                }
            }
            Ok(bytes)
        }
        _ => Err("Contents is neither Reference nor Array".to_string()),
    }
}

fn find_image_placements<'a>(
    content: &lopdf::content::Content,
    doc: &'a Document,
    xobjects: &'a lopdf::Dictionary,
) -> Vec<(Vec<u8>, [f64; 6], &'a lopdf::Stream)> {
    let mut ctm_stack: Vec<[f64; 6]> = Vec::new();
    let mut ctm: [f64; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    let mut placements = Vec::new();

    for op in &content.operations {
        match op.operator.as_str() {
            "q" => ctm_stack.push(ctm),
            "Q" => {
                if let Some(saved) = ctm_stack.pop() {
                    ctm = saved;
                }
            }
            "cm" => {
                if let Some(m) = extract_ctm_operands(&op.operands) {
                    ctm = concat_matrix(m, ctm);
                }
            }
            "Do" => {
                let name = match op.operands.first() {
                    Some(Object::Name(n)) => n.clone(),
                    _ => continue,
                };
                let xobj_ref = match xobjects.get(&name) {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                let obj = match resolve_object(doc, xobj_ref) {
                    Some(o) => o,
                    None => continue,
                };
                if !is_image_xobject(doc, obj) {
                    continue;
                }
                if let Ok(stream) = obj.as_stream() {
                    placements.push((name, ctm, stream));
                }
            }
            _ => {}
        }
    }

    placements
}

fn extract_ctm_operands(operands: &[Object]) -> Option<[f64; 6]> {
    if operands.len() < 6 {
        return None;
    }
    let mut m = [0.0_f64; 6];
    for (i, op) in operands[..6].iter().enumerate() {
        m[i] = match op {
            Object::Integer(n) => *n as f64,
            Object::Real(n) => *n as f64,
            _ => return None,
        };
    }
    Some(m)
}

fn concat_matrix(m1: [f64; 6], m2: [f64; 6]) -> [f64; 6] {
    let [a1, b1, c1, d1, e1, f1] = m1;
    let [a2, b2, c2, d2, e2, f2] = m2;
    [
        a1 * a2 + b1 * c2,
        a1 * b2 + b1 * d2,
        c1 * a2 + d1 * c2,
        c1 * b2 + d1 * d2,
        e1 * a2 + f1 * c2 + e2,
        e1 * b2 + f1 * d2 + f2,
    ]
}

fn get_page_resources(
    doc: &Document,
    page_id: lopdf::ObjectId,
) -> Option<lopdf::Dictionary> {
    let page = doc.get_object(page_id).ok()?;
    let page_dict = page.as_dict().ok()?;

    if let Ok(resources_obj) = page_dict.get(b"Resources") {
        let resolved = match resources_obj {
            Object::Reference(r) => doc.get_object(*r).ok()?,
            other => other,
        };
        return resolved.as_dict().ok().cloned();
    }

    None
}

fn get_xobjects(
    doc: &Document,
    resources: &lopdf::Dictionary,
) -> Option<lopdf::Dictionary> {
    let xobj = resources.get(b"XObject").ok()?;
    let resolved = match xobj {
        Object::Reference(r) => doc.get_object(*r).ok()?,
        other => other,
    };
    resolved.as_dict().ok().cloned()
}

fn resolve_object<'a>(doc: &'a Document, obj: &'a Object) -> Option<&'a Object> {
    match obj {
        Object::Reference(r) => doc.get_object(*r).ok(),
        other => Some(other),
    }
}

fn is_image_xobject(doc: &Document, object: &Object) -> bool {
    let stream = match object.as_stream() {
        Ok(s) => s,
        Err(_) => return false,
    };

    let subtype = stream.dict.get(b"Subtype").ok().and_then(|s| {
        match s {
            Object::Name(n) => Some(n.clone()),
            Object::Reference(r) => doc
                .get_object(*r)
                .ok()
                .and_then(|o| o.as_name().ok().map(|n| n.to_vec())),
            _ => None,
        }
    });

    subtype.as_deref() == Some(b"Image")
}

fn extract_single_image(
    stream: &lopdf::Stream,
    image_id: &str,
    counter: u32,
    options: &ImageOptions,
) -> Result<ExtractedImage, String> {
    let filter = stream
        .dict
        .get(b"Filter")
        .ok()
        .and_then(|f| match f {
            Object::Name(n) => Some(String::from_utf8_lossy(n).to_string()),
            Object::Array(arr) => arr.first().and_then(|item| {
                if let Object::Name(n) = item {
                    Some(String::from_utf8_lossy(n).to_string())
                } else {
                    None
                }
            }),
            _ => None,
        })
        .unwrap_or_default();

    let width = get_dict_int(&stream.dict, b"Width").unwrap_or(0) as u32;
    let height = get_dict_int(&stream.dict, b"Height").unwrap_or(0) as u32;

    if width == 0 || height == 0 {
        return Err("Image has zero dimensions".to_string());
    }

    if filter == "DCTDecode" {
        let data = stream.content.clone();
        return process_image_data(data, "image/jpeg", width, height, image_id, counter, options);
    }

    if filter == "JPXDecode" || filter == "JBIG2Decode" {
        return Err(format!("Unsupported image format: {}", filter));
    }

    let data = decompress_stream(stream, &filter)?;

    let color_space = stream
        .dict
        .get(b"ColorSpace")
        .ok()
        .and_then(|cs| match cs {
            Object::Name(n) => Some(String::from_utf8_lossy(n).to_string()),
            _ => None,
        })
        .unwrap_or_else(|| "DeviceRGB".to_string());

    let bits_per_component = get_dict_int(&stream.dict, b"BitsPerComponent").unwrap_or(8) as u32;

    let rgb_data = convert_to_rgb(&data, width, height, &color_space, bits_per_component)?;

    let img = image::RgbImage::from_raw(width, height, rgb_data)
        .ok_or_else(|| "Failed to create image from raw data".to_string())?;

    let dynamic_img = image::DynamicImage::ImageRgb8(img);

    process_dynamic_image(dynamic_img, image_id, counter, options)
}

fn convert_to_rgb(
    data: &[u8],
    width: u32,
    height: u32,
    color_space: &str,
    bits_per_component: u32,
) -> Result<Vec<u8>, String> {
    let pixel_count = (width * height) as usize;

    match color_space {
        "DeviceRGB" | "CalRGB" => {
            if bits_per_component == 8 {
                if data.len() >= pixel_count * 3 {
                    Ok(data[..pixel_count * 3].to_vec())
                } else {
                    Err("Insufficient RGB data".to_string())
                }
            } else {
                Err(format!(
                    "Unsupported bits per component: {}",
                    bits_per_component
                ))
            }
        }
        "DeviceGray" | "CalGray" => {
            if data.len() >= pixel_count {
                let mut rgb = Vec::with_capacity(pixel_count * 3);
                for &gray in &data[..pixel_count] {
                    rgb.push(gray);
                    rgb.push(gray);
                    rgb.push(gray);
                }
                Ok(rgb)
            } else {
                Err("Insufficient grayscale data".to_string())
            }
        }
        "DeviceCMYK" => {
            if data.len() >= pixel_count * 4 {
                let mut rgb = Vec::with_capacity(pixel_count * 3);
                for i in 0..pixel_count {
                    let c = data[i * 4] as f32 / 255.0;
                    let m = data[i * 4 + 1] as f32 / 255.0;
                    let y = data[i * 4 + 2] as f32 / 255.0;
                    let k = data[i * 4 + 3] as f32 / 255.0;
                    rgb.push(((1.0 - c) * (1.0 - k) * 255.0) as u8);
                    rgb.push(((1.0 - m) * (1.0 - k) * 255.0) as u8);
                    rgb.push(((1.0 - y) * (1.0 - k) * 255.0) as u8);
                }
                Ok(rgb)
            } else {
                Err("Insufficient CMYK data".to_string())
            }
        }
        _ => Err(format!("Unsupported color space: {}", color_space)),
    }
}

fn process_image_data(
    data: Vec<u8>,
    mime: &str,
    width: u32,
    height: u32,
    image_id: &str,
    counter: u32,
    options: &ImageOptions,
) -> Result<ExtractedImage, String> {
    let (actual_w, actual_h) = if mime == "image/jpeg" {
        jpeg_dimensions(&data).unwrap_or((width, height))
    } else {
        (width, height)
    };

    if actual_w <= options.max_image_width && !options.convert_to_webp {
        return Ok(ExtractedImage {
            id: format!("{}_{}", image_id, counter),
            data,
            mime_type: mime.to_string(),
            width: actual_w,
            height: actual_h,
        });
    }

    let img = image::load_from_memory(&data)
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    process_dynamic_image(img, image_id, counter, options)
}

fn process_dynamic_image(
    mut img: image::DynamicImage,
    image_id: &str,
    counter: u32,
    options: &ImageOptions,
) -> Result<ExtractedImage, String> {
    let (mut w, mut h) = (img.width(), img.height());

    if w > options.max_image_width {
        let ratio = options.max_image_width as f32 / w as f32;
        let new_h = (h as f32 * ratio) as u32;
        img = img.resize(options.max_image_width, new_h, image::imageops::FilterType::Lanczos3);
        w = img.width();
        h = img.height();
    }

    let quality = match options.image_quality.as_str() {
        "high" => 90,
        "low" => 50,
        _ => 75,
    };

    let (data, mime) = if options.convert_to_webp {
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::WebP)
            .map_err(|e| format!("Failed to encode WebP: {}", e))?;
        (buf.into_inner(), "image/webp".to_string())
    } else {
        let mut buf = std::io::Cursor::new(Vec::new());
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality);
        img.write_with_encoder(encoder)
            .map_err(|e| format!("Failed to encode JPEG: {}", e))?;
        (buf.into_inner(), "image/jpeg".to_string())
    };

    Ok(ExtractedImage {
        id: format!("{}_{}", image_id, counter),
        data,
        mime_type: mime,
        width: w,
        height: h,
    })
}

fn decompress_stream(stream: &lopdf::Stream, filter: &str) -> Result<Vec<u8>, String> {
    // Try lopdf's built-in decompression first
    if let Ok(data) = stream.decompressed_content() {
        return Ok(data);
    }

    // Fallback: manual decompression for FlateDecode when lopdf fails
    // (lopdf can choke on streams with indirect-ref DecodeParms)
    if filter == "FlateDecode" {
        use flate2::read::ZlibDecoder;
        use std::io::Read;

        let mut decoder = ZlibDecoder::new(stream.content.as_slice());
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| format!("Manual FlateDecode failed: {}", e))?;
        return Ok(decompressed);
    }

    // No filter — raw bytes are the image data
    if filter.is_empty() {
        return Ok(stream.content.clone());
    }

    Err(format!(
        "Cannot decompress stream with filter '{}' (lopdf failed and no manual fallback available)",
        filter
    ))
}

fn jpeg_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    if data.len() < 2 || data[0] != 0xFF || data[1] != 0xD8 {
        return None;
    }
    let mut i = 2;
    while i + 1 < data.len() {
        if data[i] != 0xFF {
            return None;
        }
        while i + 1 < data.len() && data[i + 1] == 0xFF {
            i += 1;
        }
        if i + 1 >= data.len() {
            return None;
        }
        let marker = data[i + 1];
        i += 2;

        if marker == 0xD8 || marker == 0xD9 || (0xD0..=0xD7).contains(&marker) || marker == 0x01 {
            continue;
        }

        if matches!(marker, 0xC0..=0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF) {
            if i + 7 <= data.len() {
                let height = u16::from_be_bytes([data[i + 3], data[i + 4]]) as u32;
                let width = u16::from_be_bytes([data[i + 5], data[i + 6]]) as u32;
                return Some((width, height));
            }
            return None;
        }

        if i + 1 < data.len() {
            let len = u16::from_be_bytes([data[i], data[i + 1]]) as usize;
            if len < 2 {
                return None;
            }
            i += len;
        } else {
            return None;
        }
    }
    None
}

fn get_dict_int(dict: &lopdf::Dictionary, key: &[u8]) -> Option<i64> {
    dict.get(key).ok().and_then(|v| match v {
        Object::Integer(i) => Some(*i),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::{dictionary, Stream};
    use tempfile::NamedTempFile;

    fn create_jpeg_data(width: u32, height: u32) -> Vec<u8> {
        let img = image::RgbImage::from_pixel(width, height, image::Rgb([100, 150, 200]));
        let dynamic = image::DynamicImage::ImageRgb8(img);
        let mut buf = std::io::Cursor::new(Vec::new());
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 50);
        dynamic.write_with_encoder(encoder).unwrap();
        buf.into_inner()
    }

    fn create_pdf_with_image(img_width: u32, img_height: u32) -> NamedTempFile {
        let mut doc = Document::with_version("1.7");
        let pages_id = doc.new_object_id();
        let page_id = doc.new_object_id();

        let jpeg_data = create_jpeg_data(img_width, img_height);
        let img_stream = Stream::new(
            dictionary! {
                "Type" => "XObject",
                "Subtype" => "Image",
                "Width" => Object::Integer(img_width as i64),
                "Height" => Object::Integer(img_height as i64),
                "ColorSpace" => "DeviceRGB",
                "BitsPerComponent" => Object::Integer(8),
                "Filter" => "DCTDecode",
            },
            jpeg_data,
        );
        let img_id = doc.add_object(Object::Stream(img_stream));

        let xobjects = dictionary! {
            "Im0" => img_id,
        };
        let resources = dictionary! {
            "XObject" => xobjects,
        };

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

        let file = NamedTempFile::new().unwrap();
        doc.save(file.path()).unwrap();
        file
    }

    fn create_pdf_with_two_images(
        w1: u32, h1: u32,
        w2: u32, h2: u32,
    ) -> NamedTempFile {
        let mut doc = Document::with_version("1.7");
        let pages_id = doc.new_object_id();
        let page_id = doc.new_object_id();

        let jpeg1 = create_jpeg_data(w1, h1);
        let img1 = Stream::new(
            dictionary! {
                "Type" => "XObject",
                "Subtype" => "Image",
                "Width" => Object::Integer(w1 as i64),
                "Height" => Object::Integer(h1 as i64),
                "ColorSpace" => "DeviceRGB",
                "BitsPerComponent" => Object::Integer(8),
                "Filter" => "DCTDecode",
            },
            jpeg1,
        );
        let img1_id = doc.add_object(Object::Stream(img1));

        let jpeg2 = create_jpeg_data(w2, h2);
        let img2 = Stream::new(
            dictionary! {
                "Type" => "XObject",
                "Subtype" => "Image",
                "Width" => Object::Integer(w2 as i64),
                "Height" => Object::Integer(h2 as i64),
                "ColorSpace" => "DeviceRGB",
                "BitsPerComponent" => Object::Integer(8),
                "Filter" => "DCTDecode",
            },
            jpeg2,
        );
        let img2_id = doc.add_object(Object::Stream(img2));

        let xobjects = dictionary! {
            "Im0" => img1_id,
            "Im1" => img2_id,
        };
        let resources = dictionary! {
            "XObject" => xobjects,
        };

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

        let file = NamedTempFile::new().unwrap();
        doc.save(file.path()).unwrap();
        file
    }

    fn create_pdf_no_images() -> NamedTempFile {
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

        let root_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", root_id);

        let file = NamedTempFile::new().unwrap();
        doc.save(file.path()).unwrap();
        file
    }

    fn create_pdf_with_image_on_page_2_only(w: u32, h: u32) -> NamedTempFile {
        let mut doc = Document::with_version("1.7");
        let pages_id = doc.new_object_id();
        let page1_id = doc.new_object_id();
        let page2_id = doc.new_object_id();

        let page1 = dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
        };
        doc.objects.insert(page1_id, Object::Dictionary(page1));

        let jpeg = create_jpeg_data(w, h);
        let img_stream = Stream::new(
            dictionary! {
                "Type" => "XObject",
                "Subtype" => "Image",
                "Width" => Object::Integer(w as i64),
                "Height" => Object::Integer(h as i64),
                "ColorSpace" => "DeviceRGB",
                "BitsPerComponent" => Object::Integer(8),
                "Filter" => "DCTDecode",
            },
            jpeg,
        );
        let img_id = doc.add_object(Object::Stream(img_stream));

        let xobjects = dictionary! { "Im0" => img_id };
        let resources = dictionary! { "XObject" => xobjects };

        let page2 = dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Resources" => resources,
        };
        doc.objects.insert(page2_id, Object::Dictionary(page2));

        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page1_id.into(), page2_id.into()],
            "Count" => Object::Integer(2),
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

    // -- cover_mode = "none" --

    #[test]
    fn cover_none_returns_none() {
        let result = extract_cover_image("/nonexistent.pdf", "none").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn cover_none_skips_pdf_with_images() {
        let file = create_pdf_with_image(600, 800);
        let result = extract_cover_image(file.path().to_str().unwrap(), "none").unwrap();
        assert!(result.is_none());
    }

    // -- cover_mode = "auto" --

    #[test]
    fn cover_auto_returns_large_image() {
        let file = create_pdf_with_image(600, 800);
        let result = extract_cover_image(file.path().to_str().unwrap(), "auto").unwrap();
        assert!(result.is_some());
        let img = result.unwrap();
        assert!(img.width >= 300);
        assert!(img.height >= 400);
    }

    #[test]
    fn cover_auto_rejects_small_image() {
        let file = create_pdf_with_image(100, 100);
        let result = extract_cover_image(file.path().to_str().unwrap(), "auto").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn cover_auto_rejects_narrow_image() {
        let file = create_pdf_with_image(200, 800);
        let result = extract_cover_image(file.path().to_str().unwrap(), "auto").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn cover_auto_rejects_short_image() {
        let file = create_pdf_with_image(600, 200);
        let result = extract_cover_image(file.path().to_str().unwrap(), "auto").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn cover_auto_at_exact_threshold() {
        let file = create_pdf_with_image(300, 400);
        let result = extract_cover_image(file.path().to_str().unwrap(), "auto").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn cover_auto_just_below_threshold_width() {
        let file = create_pdf_with_image(299, 400);
        let result = extract_cover_image(file.path().to_str().unwrap(), "auto").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn cover_auto_just_below_threshold_height() {
        let file = create_pdf_with_image(300, 399);
        let result = extract_cover_image(file.path().to_str().unwrap(), "auto").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn cover_auto_returns_none_for_no_images() {
        let file = create_pdf_no_images();
        let result = extract_cover_image(file.path().to_str().unwrap(), "auto").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn cover_auto_only_checks_page_1() {
        let file = create_pdf_with_image_on_page_2_only(600, 800);
        let result = extract_cover_image(file.path().to_str().unwrap(), "auto").unwrap();
        assert!(result.is_none());
    }

    // -- cover_mode = "firstPage" --

    #[test]
    fn cover_firstpage_returns_any_size_image() {
        let file = create_pdf_with_image(50, 50);
        let result = extract_cover_image(file.path().to_str().unwrap(), "firstPage").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn cover_firstpage_returns_none_for_no_images() {
        let file = create_pdf_no_images();
        let result = extract_cover_image(file.path().to_str().unwrap(), "firstPage").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn cover_firstpage_only_checks_page_1() {
        let file = create_pdf_with_image_on_page_2_only(600, 800);
        let result = extract_cover_image(file.path().to_str().unwrap(), "firstPage").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn cover_firstpage_picks_largest_image() {
        let file = create_pdf_with_two_images(100, 100, 400, 500);
        let result = extract_cover_image(file.path().to_str().unwrap(), "firstPage").unwrap();
        let img = result.unwrap();
        assert!(img.width >= 400);
        assert!(img.height >= 500);
    }

    // -- invalid inputs --

    #[test]
    fn cover_invalid_path_returns_error() {
        let result = extract_cover_image("/nonexistent.pdf", "auto");
        assert!(result.is_err());
    }

    #[test]
    fn cover_firstpage_invalid_path_returns_error() {
        let result = extract_cover_image("/nonexistent.pdf", "firstPage");
        assert!(result.is_err());
    }

    #[test]
    fn cover_unknown_mode_returns_none() {
        let file = create_pdf_with_image(600, 800);
        let result = extract_cover_image(file.path().to_str().unwrap(), "unknown").unwrap();
        assert!(result.is_none());
    }

    // -- cover image properties --

    #[test]
    fn cover_image_has_jpeg_mime_type() {
        let file = create_pdf_with_image(400, 500);
        let result = extract_cover_image(file.path().to_str().unwrap(), "firstPage").unwrap();
        let img = result.unwrap();
        assert_eq!(img.mime_type, "image/jpeg");
    }

    #[test]
    fn cover_image_id_contains_cover_prefix() {
        let file = create_pdf_with_image(400, 500);
        let result = extract_cover_image(file.path().to_str().unwrap(), "firstPage").unwrap();
        let img = result.unwrap();
        assert!(img.id.contains("cover_"));
    }

    #[test]
    fn cover_image_has_nonempty_data() {
        let file = create_pdf_with_image(400, 500);
        let result = extract_cover_image(file.path().to_str().unwrap(), "firstPage").unwrap();
        let img = result.unwrap();
        assert!(!img.data.is_empty());
    }

    #[test]
    fn cover_image_preserves_full_resolution_above_1600() {
        let file = create_pdf_with_image(2400, 3200);
        let result = extract_cover_image(file.path().to_str().unwrap(), "firstPage").unwrap();
        let img = result.unwrap();
        assert_eq!(img.width, 2400);
        assert_eq!(img.height, 3200);
    }

    #[test]
    fn cover_uses_actual_jpeg_dimensions_not_dictionary() {
        let mut doc = Document::with_version("1.7");
        let pages_id = doc.new_object_id();
        let page_id = doc.new_object_id();

        let jpeg_data = create_jpeg_data(800, 1000);
        let img_stream = Stream::new(
            dictionary! {
                "Type" => "XObject",
                "Subtype" => "Image",
                "Width" => Object::Integer(400),
                "Height" => Object::Integer(500),
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

        let file = NamedTempFile::new().unwrap();
        doc.save(file.path()).unwrap();

        let result = extract_cover_image(file.path().to_str().unwrap(), "firstPage").unwrap();
        let img = result.unwrap();
        assert_eq!(img.width, 800);
        assert_eq!(img.height, 1000);
    }

    #[test]
    fn cover_selects_largest_by_actual_dimensions() {
        let mut doc = Document::with_version("1.7");
        let pages_id = doc.new_object_id();
        let page_id = doc.new_object_id();

        // Small JPEG with inflated dictionary dimensions
        let small_jpeg = create_jpeg_data(100, 100);
        let small_stream = Stream::new(
            dictionary! {
                "Type" => "XObject",
                "Subtype" => "Image",
                "Width" => Object::Integer(2000),
                "Height" => Object::Integer(2000),
                "ColorSpace" => "DeviceRGB",
                "BitsPerComponent" => Object::Integer(8),
                "Filter" => "DCTDecode",
            },
            small_jpeg,
        );
        let small_id = doc.add_object(Object::Stream(small_stream));

        // Large JPEG with accurate dictionary dimensions
        let large_jpeg = create_jpeg_data(600, 800);
        let large_stream = Stream::new(
            dictionary! {
                "Type" => "XObject",
                "Subtype" => "Image",
                "Width" => Object::Integer(600),
                "Height" => Object::Integer(800),
                "ColorSpace" => "DeviceRGB",
                "BitsPerComponent" => Object::Integer(8),
                "Filter" => "DCTDecode",
            },
            large_jpeg,
        );
        let large_id = doc.add_object(Object::Stream(large_stream));

        let xobjects = dictionary! {
            "Im0" => small_id,
            "Im1" => large_id,
        };
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

        let file = NamedTempFile::new().unwrap();
        doc.save(file.path()).unwrap();

        let result = extract_cover_image(file.path().to_str().unwrap(), "firstPage").unwrap();
        let img = result.unwrap();
        assert_eq!(img.width, 600);
        assert_eq!(img.height, 800);
    }

    // -- jpeg_dimensions tests --

    #[test]
    fn jpeg_dimensions_parses_valid_jpeg() {
        let data = create_jpeg_data(640, 480);
        let dims = jpeg_dimensions(&data);
        assert_eq!(dims, Some((640, 480)));
    }

    #[test]
    fn jpeg_dimensions_returns_none_for_non_jpeg() {
        assert_eq!(jpeg_dimensions(&[0x89, b'P', b'N', b'G']), None);
    }

    #[test]
    fn jpeg_dimensions_returns_none_for_empty() {
        assert_eq!(jpeg_dimensions(&[]), None);
    }

    #[test]
    fn jpeg_dimensions_returns_none_for_truncated() {
        assert_eq!(jpeg_dimensions(&[0xFF, 0xD8, 0xFF]), None);
    }

    // -- render_cover_page tests --

    fn create_tiled_pdf(
        w1: u32, h1: u32,
        w2: u32, h2: u32,
    ) -> NamedTempFile {
        let mut doc = Document::with_version("1.7");
        let pages_id = doc.new_object_id();
        let page_id = doc.new_object_id();

        let jpeg1 = create_jpeg_data(w1, h1);
        let img1 = Stream::new(
            dictionary! {
                "Type" => "XObject",
                "Subtype" => "Image",
                "Width" => Object::Integer(w1 as i64),
                "Height" => Object::Integer(h1 as i64),
                "ColorSpace" => "DeviceRGB",
                "BitsPerComponent" => Object::Integer(8),
                "Filter" => "DCTDecode",
            },
            jpeg1,
        );
        let img1_id = doc.add_object(Object::Stream(img1));

        let jpeg2 = create_jpeg_data(w2, h2);
        let img2 = Stream::new(
            dictionary! {
                "Type" => "XObject",
                "Subtype" => "Image",
                "Width" => Object::Integer(w2 as i64),
                "Height" => Object::Integer(h2 as i64),
                "ColorSpace" => "DeviceRGB",
                "BitsPerComponent" => Object::Integer(8),
                "Filter" => "DCTDecode",
            },
            jpeg2,
        );
        let img2_id = doc.add_object(Object::Stream(img2));

        let xobjects = dictionary! {
            "Im0" => img1_id,
            "Im1" => img2_id,
        };
        let resources = dictionary! { "XObject" => xobjects };

        // Content stream: two images tiled vertically (top half + bottom half)
        let content_str = b"q 612 0 0 396 0 396 cm /Im0 Do Q q 612 0 0 396 0 0 cm /Im1 Do Q";
        let content_stream = Stream::new(dictionary! {}, content_str.to_vec());
        let content_id = doc.add_object(Object::Stream(content_stream));

        let page = dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Resources" => resources,
            "Contents" => content_id,
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

    #[test]
    fn render_cover_composites_tiled_images() {
        let file = create_tiled_pdf(400, 300, 400, 300);
        let result = render_cover_page(file.path().to_str().unwrap()).unwrap();
        assert!(result.is_some());
        let img = result.unwrap();
        assert!(img.width() > 0);
        assert!(img.height() > 0);
        // Canvas should have page aspect ratio (612:792 ≈ 0.77)
        let ratio = img.width() as f64 / img.height() as f64;
        assert!((ratio - 612.0 / 792.0).abs() < 0.05);
    }

    #[test]
    fn render_cover_returns_none_for_single_image() {
        let file = create_pdf_with_image(600, 800);
        let result = render_cover_page(file.path().to_str().unwrap()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn render_cover_returns_none_for_no_images() {
        let file = create_pdf_no_images();
        let result = render_cover_page(file.path().to_str().unwrap()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn render_cover_returns_none_for_invalid_path() {
        let result = render_cover_page("/nonexistent.pdf");
        assert!(result.is_err());
    }
}
