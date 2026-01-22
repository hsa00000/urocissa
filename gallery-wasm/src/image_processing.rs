use blake3::Hasher;
use exif::{In, Reader};
use image::ImageFormat;
use serde::Serialize;
use std::collections::BTreeMap;
use std::io::Cursor;
use thumbhash::rgba_to_thumb_hash;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessedImage {
    pub hash: String,
    pub width: u32,
    pub height: u32,
    pub size: u64,
    pub extension: String,
    pub thumbhash: Option<Vec<u8>>,
    pub phash: Option<Vec<u8>>,
    pub exif: BTreeMap<String, String>,
    pub compressed_image: Vec<u8>,
    pub last_modified: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadLocalObject {
    pub id: String,
    pub obj_type: String,
    pub pending: bool,
    pub thumbhash: Option<Vec<u8>>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub is_favorite: bool,
    pub is_archived: bool,
    pub is_trashed: bool,
    pub update_at: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadLocalFileModify {
    pub file: String,
    pub modified: i64,
    pub scan_time: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadLocalMetadata {
    pub id: String,
    pub size: u64,
    pub width: u32,
    pub height: u32,
    pub ext: String,
    pub phash: Option<Vec<u8>>,
    pub albums: Vec<String>,
    pub exif_vec: BTreeMap<String, String>,
    pub alias: Vec<UploadLocalFileModify>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadLocalPayload {
    pub object: UploadLocalObject,
    pub metadata: UploadLocalMetadata,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessImageForUploadResult {
    /// A JSON string matching backend `UploadLocalPayload`.
    pub metadata_json: String,
    /// JPEG bytes (optimized) for multipart field `compressed`.
    pub compressed_jpeg: Vec<u8>,
    /// The computed object id (hash).
    pub hash: String,
}

#[wasm_bindgen]
pub fn process_image(
    data: &[u8],
    filename: &str,
    last_modified_ms: f64, // JS number passed as f64
    album_id_opt: Option<String>,
) -> Result<JsValue, JsValue> {
    let cursor = Cursor::new(data);
    let mut reader = image::ImageReader::new(cursor);

    // 1. Handle extension and format
    // Try to get format from filename, which includes extension logic
    let format = image::ImageFormat::from_path(filename).map_err(|e| e.to_string())?;

    // Reject GIF (as per original requirement)
    if format == ImageFormat::Gif {
        return Ok(JsValue::NULL);
    }

    reader.set_format(format);
    let dynamic_image = reader.decode().map_err(|e| e.to_string())?;
    let (width, height) = (dynamic_image.width(), dynamic_image.height());

    // Get standardized extension (e.g., jpeg -> jpg)
    let extension = format.extensions_str().get(0).unwrap_or(&"jpg").to_string();

    // 2. Calculate Hash (BLAKE3)
    let mut hasher = Hasher::new();
    hasher.update(data);
    let hash = hasher.finalize().to_hex().to_string();

    // 3. EXIF
    let mut exif_map = BTreeMap::new();
    let cursor_exif = Cursor::new(data);
    let mut bufreader = std::io::BufReader::new(cursor_exif);
    let exif_reader = Reader::new();

    // Try to read EXIF, ignore if failed
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

    // 4. Thumbhash
    let thumbhash_vec = if width > 100 && height > 100 {
        let thumb = dynamic_image.thumbnail(100, 100);
        let (w, h) = (thumb.width(), thumb.height());
        let rgba = thumb.to_rgba8().into_raw();
        Some(rgba_to_thumb_hash(w as usize, h as usize, &rgba))
    } else {
        let rgba = dynamic_image.to_rgba8().into_raw();
        Some(rgba_to_thumb_hash(width as usize, height as usize, &rgba))
    };

    // 5. Compressed Image (JPEG)
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
        extension,
        thumbhash: thumbhash_vec,
        phash: None,
        exif: exif_map,
        compressed_image: compressed_bytes,
        last_modified: last_modified_ms as i64,
    };

    // Build an upload-ready payload that matches backend `UploadLocalPayload` (camelCase).
    // We serialize it to a JSON string inside wasm to avoid JS-side mapping and Uint8Array->number[] conversions.
    let now_ms = js_sys::Date::now() as i64;
    let payload = UploadLocalPayload {
        object: UploadLocalObject {
            id: result.hash.clone(),
            obj_type: "image".to_string(),
            pending: false,
            thumbhash: result.thumbhash.clone(),
            description: None,
            tags: Vec::new(),
            is_favorite: false,
            is_archived: false,
            is_trashed: false,
            update_at: now_ms,
        },
        metadata: UploadLocalMetadata {
            id: result.hash.clone(),
            size: result.size,
            width: result.width,
            height: result.height,
            ext: result.extension.clone(),
            phash: result.phash.clone(),
            albums: album_id_opt.into_iter().collect(),
            exif_vec: result.exif.clone(),
            alias: vec![UploadLocalFileModify {
                file: filename.to_string(),
                modified: result.last_modified,
                scan_time: now_ms,
            }],
        },
    };

    let payload_js = payload
        .serialize(
            &serde_wasm_bindgen::Serializer::new()
                .serialize_maps_as_objects(true)
                .serialize_bytes_as_arrays(true),
        )
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let metadata_json = js_sys::JSON::stringify(&payload_js)
        .map_err(|e| {
            JsValue::from_str(
                &e.as_string()
                    .unwrap_or_else(|| "Failed to stringify upload metadata".to_string()),
            )
        })?
        .as_string()
        .ok_or_else(|| JsValue::from_str("JSON.stringify did not return a string"))?;

    let upload_result = ProcessImageForUploadResult {
        metadata_json,
        compressed_jpeg: result.compressed_image,
        hash: result.hash,
    };

    Ok(serde_wasm_bindgen::to_value(&upload_result).map_err(|e| e.to_string())?)
}
