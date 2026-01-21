use blake3::Hasher;
use exif::{In, Reader};
use image::ImageFormat;
use serde::Serialize;
use std::collections::BTreeMap;
use std::io::Cursor;
use thumbhash::rgba_to_thumb_hash;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
pub struct ProcessedImage {
    pub hash: String,
    pub width: u32,
    pub height: u32,
    pub size: u64,
    pub thumbhash: Option<Vec<u8>>,
    pub phash: Option<Vec<u8>>,
    pub exif: BTreeMap<String, String>,
    pub compressed_image: Vec<u8>,
}

#[wasm_bindgen]
pub fn process_image(data: &[u8], filename: &str) -> Result<JsValue, JsValue> {
    let cursor = Cursor::new(data);
    let mut reader = image::ImageReader::new(cursor);
    reader.set_format(image::ImageFormat::from_path(filename).map_err(|e| e.to_string())?);

    // Check if the image format is supported for WASM processing (exclude GIF as requested)
    let format = image::ImageFormat::from_path(filename).map_err(|e| e.to_string())?;
    if format == ImageFormat::Gif {
        // Return null or specific error/code to indicate skipping
        return Ok(JsValue::NULL);
    }

    let dynamic_image = reader.decode().map_err(|e| e.to_string())?;
    let (width, height) = (dynamic_image.width(), dynamic_image.height());

    // 1. Calculate Hash (BLAKE3)
    let mut hasher = Hasher::new();
    hasher.update(data);
    let hash = hasher.finalize().to_hex().to_string();

    // 2. EXIF
    let mut exif_map = BTreeMap::new();
    let cursor_exif = Cursor::new(data);
    let mut bufreader = std::io::BufReader::new(cursor_exif);
    let exif_reader = Reader::new();
    if let Ok(exif) = exif_reader.read_from_container(&mut bufreader) {
        for field in exif.fields() {
            if field.ifd_num == In::PRIMARY {
                exif_map.insert(
                    field.tag.to_string(),
                    field.display_value().with_unit(&exif).to_string(),
                );
            }
        }
    }

    // 3. Thumbhash
    let thumbhash_vec = if width > 100 && height > 100 {
        let thumb = dynamic_image.thumbnail(100, 100);
        let (w, h) = (thumb.width(), thumb.height());
        let rgba = thumb.to_rgba8().into_raw();
        Some(rgba_to_thumb_hash(w as usize, h as usize, &rgba))
    } else {
        let rgba = dynamic_image.to_rgba8().into_raw();
        Some(rgba_to_thumb_hash(width as usize, height as usize, &rgba))
    };

    // 4. Compressed Image (JPEG)
    let mut compressed_bytes: Vec<u8> = Vec::new();
    dynamic_image
        .write_to(
            &mut Cursor::new(&mut compressed_bytes),
            image::ImageFormat::Jpeg,
        )
        .map_err(|e| e.to_string())?;

    let result = ProcessedImage {
        hash,
        width,
        height,
        size: data.len() as u64,
        thumbhash: thumbhash_vec,
        phash: None, // pHash usually requires another crate, skipping for MVP
        exif: exif_map,
        compressed_image: compressed_bytes,
    };

    Ok(serde_wasm_bindgen::to_value(&result).map_err(|e| e.to_string())?)
}
