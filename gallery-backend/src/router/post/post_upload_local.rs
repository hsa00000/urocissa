use crate::operations::open_db::open_data_table;
use crate::public::constant::storage::get_data_path;
use crate::public::error::{AppError, ErrorKind};
use crate::public::structure::abstract_data::AbstractData;
use crate::public::structure::common::FileModify;
use crate::public::structure::image::{ImageCombined, ImageMetadata};
use crate::public::structure::object::{ObjectSchema, ObjectType};
use crate::router::fairing::guard_read_only_mode::GuardReadOnlyMode;
use crate::router::fairing::guard_upload::GuardUpload;
use crate::router::{AppResult, GuardResult};
use crate::tasks::BATCH_COORDINATOR;
use crate::tasks::batcher::flush_tree::FlushTreeTask;
use anyhow::Result;
use arrayvec::ArrayString;
use chrono::Utc;
use rocket::form::{Errors, Form};
use rocket::fs::TempFile;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::mem;

/// Form for local-first upload with processed metadata
#[derive(FromForm, Debug)]
pub struct UploadLocalForm<'r> {
    /// Serialized JSON metadata
    #[field(name = "metadata")]
    pub metadata: String,

    /// Compressed image (JPEG)
    #[field(name = "compressed")]
    pub compressed: TempFile<'r>,

    /// Original file
    #[field(name = "original")]
    pub original: TempFile<'r>,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProcessedMetadata {
    pub hash: String,
    pub width: u32,
    pub height: u32,
    pub size: u64,
    pub thumbhash: Option<Vec<u8>>,
    pub phash: Option<Vec<u8>>,
    pub exif: BTreeMap<String, String>,
    #[serde(default)]
    pub last_modified: Option<i64>,
}

#[post("/upload-local?<presigned_album_id_opt>", data = "<form>")]
pub async fn upload_local(
    auth: GuardResult<GuardUpload>,
    read_only_mode: GuardResult<GuardReadOnlyMode>,
    presigned_album_id_opt: Option<String>,
    form: Result<Form<UploadLocalForm<'_>>, Errors<'_>>,
) -> AppResult<()> {
    let _ = auth?;
    let _ = read_only_mode?;

    let mut inner_form = match form {
        Ok(f) => f.into_inner(),
        Err(e) => {
            return Err(AppError::new(
                ErrorKind::InvalidInput,
                format!("Form error: {:?}", e),
            ));
        }
    };

    let meta: ProcessedMetadata = serde_json::from_str(&inner_form.metadata).map_err(|e| {
        AppError::new(
            ErrorKind::InvalidInput,
            format!("Invalid metadata JSON: {}", e),
        )
    })?;

    let hash_str = &meta.hash;
    let hash = ArrayString::from(hash_str)
        .map_err(|_| AppError::new(ErrorKind::InvalidInput, "Hash too long"))?;

    // 1. Determine paths
    let root = get_data_path();
    let original_ext = inner_form
        .original
        .name()
        .and_then(|n| std::path::Path::new(n).extension())
        .and_then(|e| e.to_str())
        .unwrap_or("jpg")
        .to_lowercase();

    // Use a temporary AbstractData (or logic) to get standard paths
    // We construct the "intended" new data to use its path methods
    let mut albums = HashSet::new();
    if let Some(aid) = presigned_album_id_opt {
        if let Ok(aid_array) = ArrayString::from(&aid) {
            albums.insert(aid_array);
        }
    }

    let timestamp = Utc::now().timestamp_millis();
    let modified_time = meta.last_modified.unwrap_or(timestamp);

    // Construct Object Schema
    let object = ObjectSchema {
        id: hash,
        obj_type: ObjectType::Image,
        pending: false,
        thumbhash: meta.thumbhash,
        description: None,
        tags: HashSet::new(),
        is_favorite: false,
        is_archived: false,
        is_trashed: false,
        update_at: timestamp,
    };

    let mut metadata = ImageMetadata {
        id: hash,
        size: meta.size,
        width: meta.width,
        height: meta.height,
        ext: original_ext.clone(),
        phash: meta.phash,
        albums,
        exif_vec: meta.exif,
        alias: vec![], // Will populate
    };

    // We create a temporary combined object to resolve paths
    // But we can just compute paths manually to be safe and avoid "self" issues if alias is empty
    let hash_prefix = &hash_str[0..2];

    // Imported Path: object/imported/xx/hash.ext
    let imported_dir = root.join(format!("object/imported/{}", hash_prefix));
    let imported_path = imported_dir.join(format!("{}.{}", hash_str, original_ext));

    // Compressed Path: object/compressed/xx/hash.jpg
    let compressed_dir = root.join(format!("object/compressed/{}", hash_prefix));
    let compressed_path = compressed_dir.join(format!("{}.jpg", hash_str));

    // 2. Save Files
    fs::create_dir_all(&imported_dir).map_err(|e| AppError::new(ErrorKind::IO, e.to_string()))?;
    inner_form
        .original
        .move_copy_to(&imported_path)
        .await
        .map_err(|e| AppError::new(ErrorKind::IO, e.to_string()))?;

    // Set modification time on the imported file
    let mtime = filetime::FileTime::from_unix_time((modified_time / 1000) as i64, 0);
    let _ = filetime::set_file_mtime(&imported_path, mtime);

    fs::create_dir_all(&compressed_dir).map_err(|e| AppError::new(ErrorKind::IO, e.to_string()))?;
    // We use the 'compressed' file from WASM (high res jpeg)
    inner_form
        .compressed
        .move_copy_to(&compressed_path)
        .await
        .map_err(|e| AppError::new(ErrorKind::IO, e.to_string()))?;

    // 3. Construct FileModify with correct path
    let file_modify = FileModify {
        file: imported_path.to_string_lossy().into_owned(),
        modified: modified_time,
        scan_time: timestamp,
    };
    metadata.alias.push(file_modify);

    let mut abstract_data = AbstractData::Image(ImageCombined { object, metadata });

    // 4. Deduplication / Insertion Logic
    let data_table = open_data_table();

    // Check if hash exists
    if let Some(guard) = data_table.get(&*hash).unwrap() {
        // Exists: Merge logic
        let mut data_exist = guard.value();

        // Merge Alias
        if let Some(new_alias_mut) = abstract_data.alias_mut() {
            // We take the one alias we just created
            if !new_alias_mut.is_empty() {
                let new_fm = mem::take(&mut new_alias_mut[0]);
                if let Some(exist_alias) = data_exist.alias_mut() {
                    // Avoid strict duplicates if path is identical?
                    // But FileModify equality is scan_time.
                    // Let's just push it, consistent with deduplicate.rs
                    exist_alias.push(new_fm);
                }
            }
        }

        // Merge Albums
        if let Some(new_albums) = abstract_data.albums() {
            if let Some(exist_albums) = data_exist.albums_mut() {
                for alb in new_albums {
                    exist_albums.insert(*alb);
                }
            }
        }

        // Update/Insert the merged data
        BATCH_COORDINATOR.execute_batch_detached(FlushTreeTask::insert(vec![data_exist]));
    } else {
        // New: Insert directly
        BATCH_COORDINATOR.execute_batch_detached(FlushTreeTask::insert(vec![abstract_data]));
    }

    Ok(())
}
