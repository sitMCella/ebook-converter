use super::ImageOptions;
use lopdf::{Document, Object};

#[allow(dead_code)]
pub struct ExtractedImage {
    pub id: String,
    pub data: Vec<u8>,
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
    pub display_width_pct: Option<u32>,
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

    let doc = mupdf::Document::open(path)
        .map_err(|e| format!("Failed to open PDF for image extraction: {}", e))?;

    let page_count = doc
        .page_count()
        .map_err(|e| format!("Failed to get page count: {}", e))?;

    let mut images = Vec::new();
    let mut image_counter = 0u32;

    for page_idx in 0..page_count {
        let page = match doc.load_page(page_idx) {
            Ok(p) => p,
            Err(e) => {
                log::warn!("Skipping page {}: {}", page_idx + 1, e);
                continue;
            }
        };

        let page_width = page
            .bounds()
            .map(|b| b.x1 - b.x0)
            .unwrap_or(612.0);

        let collector = std::rc::Rc::new(std::cell::RefCell::new(ImageCollector {
            images: Vec::new(),
        }));
        let device = mupdf::Device::from_native(collector.clone())
            .map_err(|e| format!("Failed to create device: {}", e))?;
        let identity = mupdf::Matrix::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        if let Err(e) = page.run(&device, &identity) {
            log::warn!("Failed to run page {} through device: {}", page_idx + 1, e);
            continue;
        }
        drop(device);

        let page_num = page_idx + 1;
        let collected = collector.borrow();

        for (img_idx, (mupdf_img, ctm)) in collected.images.iter().enumerate() {
            image_counter += 1;
            let image_id = format!("img_p{}_{}", page_num, img_idx);

            let rendered_w = (ctm.a * ctm.a + ctm.b * ctm.b).sqrt();
            let pct_of_page = rendered_w / page_width;
            let display_width_pct = if pct_of_page > 0.0 && pct_of_page < 0.15 {
                Some(((pct_of_page * 100.0).round() as u32).max(1))
            } else {
                None
            };

            match mupdf_image_to_extracted(mupdf_img, &image_id, image_counter, display_width_pct, options) {
                Ok(img) => images.push(img),
                Err(e) => {
                    log::warn!("Skipping image {} on page {}: {}", img_idx, page_num, e);
                }
            }
        }
    }

    Ok(images)
}

struct ImageCollector {
    images: Vec<(mupdf::Image, mupdf::Matrix)>,
}

impl mupdf::device::NativeDevice for ImageCollector {
    fn fill_image(
        &mut self,
        img: &mupdf::Image,
        ctm: mupdf::Matrix,
        _alpha: f32,
        _cp: mupdf::ColorParams,
    ) {
        if img.width() >= 10 && img.height() >= 10 {
            self.images.push((img.clone(), ctm));
        }
    }
}

fn mupdf_image_to_extracted(
    img: &mupdf::Image,
    image_id: &str,
    counter: u32,
    display_width_pct: Option<u32>,
    options: &ImageOptions,
) -> Result<ExtractedImage, String> {
    let pixmap = img
        .to_pixmap()
        .map_err(|e| format!("Failed to convert image to pixmap: {}", e))?;

    let width = pixmap.width();
    let height = pixmap.height();
    if width == 0 || height == 0 {
        return Err("Image has zero dimensions".to_string());
    }

    let n = pixmap.n() as usize;
    let samples = pixmap.samples();
    let stride = pixmap.stride() as usize;
    let row_bytes = width as usize * n;

    let mut packed = Vec::with_capacity(row_bytes * height as usize);
    for row in 0..height as usize {
        let start = row * stride;
        let end = start + row_bytes;
        if end <= samples.len() {
            packed.extend_from_slice(&samples[start..end]);
        }
    }

    let dynamic_img = if n == 1 {
        let gray = image::GrayImage::from_raw(width, height, packed)
            .ok_or_else(|| "Failed to create grayscale image".to_string())?;
        image::DynamicImage::ImageLuma8(gray)
    } else if n == 3 {
        let rgb = image::RgbImage::from_raw(width, height, packed)
            .ok_or_else(|| "Failed to create RGB image".to_string())?;
        image::DynamicImage::ImageRgb8(rgb)
    } else if n == 4 {
        let rgba = image::RgbaImage::from_raw(width, height, packed)
            .ok_or_else(|| "Failed to create RGBA image".to_string())?;
        image::DynamicImage::ImageRgba8(rgba)
    } else if n == 2 {
        let mut rgb = Vec::with_capacity(width as usize * height as usize * 3);
        for chunk in packed.chunks(2) {
            let gray = chunk[0];
            rgb.push(gray);
            rgb.push(gray);
            rgb.push(gray);
        }
        let img = image::RgbImage::from_raw(width, height, rgb)
            .ok_or_else(|| "Failed to create image from gray+alpha".to_string())?;
        image::DynamicImage::ImageRgb8(img)
    } else {
        return Err(format!("Unsupported channel count: {}", n));
    };

    let mut result = process_dynamic_image(dynamic_img, image_id, counter, options)?;
    result.display_width_pct = display_width_pct;
    Ok(result)
}

pub fn render_cover_page(path: &str) -> Result<Option<image::DynamicImage>, String> {
    let doc = mupdf::Document::open(path)
        .map_err(|e| format!("Failed to open PDF with mupdf: {}", e))?;

    let page = doc
        .load_page(0)
        .map_err(|e| format!("Failed to load page: {}", e))?;

    let bounds = page
        .bounds()
        .map_err(|e| format!("Failed to get page bounds: {}", e))?;

    let page_w = bounds.x1 - bounds.x0;
    let page_h = bounds.y1 - bounds.y0;
    if page_w <= 0.0 || page_h <= 0.0 {
        return Ok(None);
    }

    let long_side = page_w.max(page_h);
    let scale = (1200.0 / long_side).clamp(1.0, 4.0);
    let matrix = mupdf::Matrix::new_scale(scale, scale);

    let pixmap = page
        .to_pixmap(
            &matrix,
            &mupdf::Colorspace::device_rgb(),
            false,
            true,
        )
        .map_err(|e| format!("Failed to render page: {}", e))?;

    let width = pixmap.width();
    let height = pixmap.height();
    if width == 0 || height == 0 {
        return Ok(None);
    }

    let n = pixmap.n() as usize;
    let samples = pixmap.samples();
    let stride = pixmap.stride() as usize;
    let row_bytes = width as usize * n;

    let mut packed = Vec::with_capacity(row_bytes * height as usize);
    for row in 0..height as usize {
        let start = row * stride;
        let end = start + row_bytes;
        if end <= samples.len() {
            packed.extend_from_slice(&samples[start..end]);
        }
    }

    let img = image::RgbImage::from_raw(width, height, packed)
        .ok_or_else(|| "Failed to create image from rendered page".to_string())?;

    Ok(Some(image::DynamicImage::ImageRgb8(img)))
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
            display_width_pct: None,
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
        display_width_pct: None,
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

    #[test]
    fn render_cover_renders_page_with_image() {
        let file = create_pdf_with_image(600, 800);
        let result = render_cover_page(file.path().to_str().unwrap()).unwrap();
        assert!(result.is_some());
        let img = result.unwrap();
        assert!(img.width() > 0);
        assert!(img.height() > 0);
    }

    #[test]
    fn render_cover_renders_page_without_images() {
        let file = create_pdf_no_images();
        let result = render_cover_page(file.path().to_str().unwrap()).unwrap();
        assert!(result.is_some());
        let img = result.unwrap();
        assert!(img.width() > 0);
        assert!(img.height() > 0);
    }

    #[test]
    fn render_cover_preserves_page_aspect_ratio() {
        let file = create_pdf_with_image(400, 500);
        let result = render_cover_page(file.path().to_str().unwrap()).unwrap();
        let img = result.unwrap();
        let ratio = img.width() as f64 / img.height() as f64;
        assert!((ratio - 612.0 / 792.0).abs() < 0.05);
    }

    #[test]
    fn render_cover_returns_error_for_invalid_path() {
        let result = render_cover_page("/nonexistent.pdf");
        assert!(result.is_err());
    }
}
