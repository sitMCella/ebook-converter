use super::ImageOptions;
use lopdf::{Document, Object};

pub struct ExtractedImage {
    pub id: String,
    pub data: Vec<u8>,
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
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
    if width <= options.max_image_width && !options.convert_to_webp {
        return Ok(ExtractedImage {
            id: format!("{}_{}", image_id, counter),
            data,
            mime_type: mime.to_string(),
            width,
            height,
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

fn get_dict_int(dict: &lopdf::Dictionary, key: &[u8]) -> Option<i64> {
    dict.get(key).ok().and_then(|v| match v {
        Object::Integer(i) => Some(*i),
        _ => None,
    })
}
