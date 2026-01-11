#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

//! Database Migration Module
//!
//! Handles the migration from redb 2.6.x (Old Schema) to redb 3.1.x (New `AbstractData` Schema).
//!

use anyhow::{Context, Result};
use arrayvec::ArrayString;
use bitcode::{Decode, Encode};
use chrono::Utc;
use dotenv::dotenv;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use redb::{
    ReadableDatabase, ReadableTable as NewReadableTable,
    ReadableTableMetadata as NewReadableTableMetadata,
};
use redb_old::{
    ReadableTable as OldReadableTable, ReadableTableMetadata as OldReadableTableMetadata,
};

pub mod v3_0_schema;
use v3_0_schema::AbstractData as AbstractDataOld;

use crate::public::constant::redb::DATA_TABLE;
use crate::public::structure::abstract_data::AbstractData;
use crate::public::structure::album::{
    combined::AlbumCombined, metadata::AlbumMetadata, share::Share as NewShare,
};
use crate::public::structure::common::FileModify;
use crate::public::structure::config::AppConfig;
use crate::public::structure::image::{combined::ImageCombined, metadata::ImageMetadata};
use crate::public::structure::object::{ObjectSchema, ObjectType};
use crate::public::structure::video::{combined::VideoCombined, metadata::VideoMetadata};

// ==================================================================================
// Old Data Structures (Snapshot for redb 2.6.x)
// ==================================================================================

mod old_structure {
    #[allow(clippy::wildcard_imports)]
    use super::*;
    use redb_old::{TypeName, Value};

    #[derive(Debug, Clone, Deserialize, Default, Serialize, Decode, Encode, PartialEq, Eq)]
    pub struct Database {
        pub hash: ArrayString<64>,
        pub size: u64,
        pub width: u32,
        pub height: u32,
        pub thumbhash: Vec<u8>,
        pub phash: Vec<u8>,
        pub ext: String,
        pub exif_vec: BTreeMap<String, String>,
        pub tag: HashSet<String>,
        pub album: HashSet<ArrayString<64>>,
        pub alias: Vec<OldFileModify>,
        pub ext_type: String,
        pub pending: bool,
    }

    impl Value for Database {
        type SelfType<'a> = Self;
        type AsBytes<'a> = Vec<u8>;

        fn fixed_width() -> Option<usize> {
            None
        }

        fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
        where
            Self: 'a,
        {
            bitcode::decode(data).expect("Corrupt Data: Failed to decode OldDatabase via bitcode")
        }

        fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
            bitcode::encode(value)
        }

        fn type_name() -> TypeName {
            TypeName::new("Database")
        }
    }

    #[derive(
        Debug,
        Default,
        Clone,
        Deserialize,
        Serialize,
        Decode,
        Encode,
        Hash,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
    )]
    #[serde(rename_all = "camelCase")]
    pub struct OldFileModify {
        pub file: String,
        pub modified: u128,
        pub scan_time: u128,
    }

    #[derive(Debug, Clone, Deserialize, Default, Serialize, Decode, Encode, PartialEq, Eq)]
    #[serde(rename_all = "camelCase")]
    pub struct Album {
        pub id: ArrayString<64>,
        pub title: Option<String>,
        pub created_time: u128,
        pub start_time: Option<u128>,
        pub end_time: Option<u128>,
        pub last_modified_time: u128,
        pub cover: Option<ArrayString<64>>,
        pub thumbhash: Option<Vec<u8>>,
        pub user_defined_metadata: HashMap<String, Vec<String>>,
        pub share_list: HashMap<ArrayString<64>, OldShare>,
        pub tag: HashSet<String>,
        pub width: u32,
        pub height: u32,
        pub item_count: usize,
        pub item_size: u64,
        pub pending: bool,
    }

    impl Value for Album {
        type SelfType<'a> = Self;
        type AsBytes<'a> = Vec<u8>;

        fn fixed_width() -> Option<usize> {
            None
        }

        fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
        where
            Self: 'a,
        {
            bitcode::decode(data).expect("Corrupt Data: Failed to decode OldAlbum via bitcode")
        }

        fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a> {
            bitcode::encode(value)
        }

        fn type_name() -> TypeName {
            TypeName::new("Album")
        }
    }

    #[derive(
        Debug, Clone, Deserialize, Default, Serialize, Decode, Encode, PartialEq, Eq, Hash,
    )]
    #[serde(rename_all = "camelCase")]
    pub struct OldShare {
        pub url: ArrayString<64>,
        pub description: String,
        pub password: Option<String>,
        pub show_metadata: bool,
        pub show_download: bool,
        pub show_upload: bool,
        pub exp: u64,
    }
}

use old_structure::{Album as OldAlbum, Database as OldDatabase};

// ==================================================================================
// Migration Logic
// ==================================================================================

const OLD_DB_PATH: &str = "./db/index.redb";
const NEW_DB_PATH: &str = "./db/index_v4.redb";
const BATCH_SIZE: usize = 5000;
const USER_DEFINED_DESCRIPTION: &str = "_user_defined_description";

enum MigrationType {
    None,
    V2ToV3,
    V3ToV4,
}

fn transform_database_to_abstract_data(old_data: OldDatabase) -> AbstractData {
    let description = old_data.exif_vec.get(USER_DEFINED_DESCRIPTION).cloned();
    let timestamp = Utc::now().timestamp_millis();

    // Legacy Business Logic:
    // Tags starting with "_" (e.g., _favorite) were previously used as boolean flags.
    // We must extract them to explicit fields in the new schema.
    let mut tags = old_data.tag.clone();
    let is_favorite = tags.remove("_favorite");
    let is_archived = tags.remove("_archived");
    let is_trashed = tags.remove("_trashed");

    let alias: Vec<FileModify> = old_data
        .alias
        .iter()
        .map(|a| FileModify {
            file: a.file.clone(),
            modified: a.modified as i64,
            scan_time: a.scan_time as i64,
        })
        .collect();

    let mut exif_vec = old_data.exif_vec.clone();
    exif_vec.remove(USER_DEFINED_DESCRIPTION);

    let thumbhash = if old_data.thumbhash.is_empty() {
        None
    } else {
        Some(old_data.thumbhash)
    };

    if old_data.ext_type == "video" {
        let duration = old_data
            .exif_vec
            .get("duration")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let object = ObjectSchema {
            id: old_data.hash,
            obj_type: ObjectType::Video,
            pending: old_data.pending,
            thumbhash,
            description,
            tags,
            is_favorite,
            is_archived,
            is_trashed,
            update_at: timestamp,
        };

        let metadata = VideoMetadata {
            id: old_data.hash,
            size: old_data.size,
            width: old_data.width,
            height: old_data.height,
            ext: old_data.ext,
            duration,
            albums: old_data.album,
            exif_vec,
            alias,
        };

        AbstractData::Video(VideoCombined { object, metadata })
    } else {
        let phash = if old_data.phash.is_empty() {
            None
        } else {
            Some(old_data.phash)
        };

        let object = ObjectSchema {
            id: old_data.hash,
            obj_type: ObjectType::Image,
            pending: old_data.pending,
            thumbhash,
            description,
            tags,
            is_favorite,
            is_archived,
            is_trashed,
            update_at: timestamp,
        };
        let metadata = ImageMetadata {
            id: old_data.hash,
            size: old_data.size,
            width: old_data.width,
            height: old_data.height,
            ext: old_data.ext,
            phash,
            albums: old_data.album,
            exif_vec,
            alias,
        };

        AbstractData::Image(ImageCombined { object, metadata })
    }
}

fn transform_album_to_abstract_data(old_album: OldAlbum) -> AbstractData {
    let description = old_album
        .user_defined_metadata
        .get(USER_DEFINED_DESCRIPTION)
        .and_then(|v| v.first())
        .cloned();

    // Legacy Business Logic: Handle pseudo-flags in tags
    let mut tags = old_album.tag.clone();
    let is_favorite = tags.remove("_favorite");
    let is_archived = tags.remove("_archived");
    let is_trashed = tags.remove("_trashed");

    let share_list: HashMap<ArrayString<64>, NewShare> = old_album
        .share_list
        .into_iter()
        .map(|(key, old_share)| {
            (
                key,
                NewShare {
                    url: old_share.url,
                    description: old_share.description,
                    password: old_share.password,
                    show_metadata: old_share.show_metadata,
                    show_download: old_share.show_download,
                    show_upload: old_share.show_upload,
                    exp: old_share.exp as i64,
                },
            )
        })
        .collect();

    let object = ObjectSchema {
        id: old_album.id,
        obj_type: ObjectType::Album,
        pending: old_album.pending,
        thumbhash: old_album.thumbhash,
        description,
        tags,
        is_favorite,
        is_archived,
        is_trashed,
        update_at: old_album.last_modified_time as i64,
    };

    let metadata = AlbumMetadata {
        id: old_album.id,
        title: old_album.title,
        created_time: old_album.created_time as i64,
        start_time: old_album.start_time.map(|t| t as i64),
        end_time: old_album.end_time.map(|t| t as i64),
        last_modified_time: old_album.last_modified_time as i64,
        cover: old_album.cover,
        item_count: old_album.item_count,
        item_size: old_album.item_size,
        share_list,
    };

    AbstractData::Album(AlbumCombined { object, metadata })
}

fn transform_v30_to_v31(old_data: AbstractDataOld) -> AbstractData {
    match old_data {
        AbstractDataOld::Image(img) => {
            let update_at = Utc::now().timestamp_millis();
            let object = ObjectSchema {
                id: img.object.id,
                obj_type: ObjectType::Image,
                pending: img.object.pending,
                thumbhash: img.object.thumbhash,
                description: img.object.description,
                tags: img.object.tags,
                is_favorite: img.object.is_favorite,
                is_archived: img.object.is_archived,
                is_trashed: img.object.is_trashed,
                update_at,
            };
            let metadata = ImageMetadata {
                id: img.metadata.id,
                size: img.metadata.size,
                width: img.metadata.width,
                height: img.metadata.height,
                ext: img.metadata.ext,
                phash: img.metadata.phash,
                albums: img.metadata.albums,
                exif_vec: img.metadata.exif_vec,
                alias: img
                    .metadata
                    .alias
                    .into_iter()
                    .map(|a| FileModify {
                        file: a.file,
                        modified: a.modified as i64,
                        scan_time: a.scan_time as i64,
                    })
                    .collect(),
            };
            AbstractData::Image(ImageCombined { object, metadata })
        }
        AbstractDataOld::Video(vid) => {
            let update_at = Utc::now().timestamp_millis();
            let object = ObjectSchema {
                id: vid.object.id,
                obj_type: ObjectType::Video,
                pending: vid.object.pending,
                thumbhash: vid.object.thumbhash,
                description: vid.object.description,
                tags: vid.object.tags,
                is_favorite: vid.object.is_favorite,
                is_archived: vid.object.is_archived,
                is_trashed: vid.object.is_trashed,
                update_at,
            };
            let metadata = VideoMetadata {
                id: vid.metadata.id,
                size: vid.metadata.size,
                width: vid.metadata.width,
                height: vid.metadata.height,
                ext: vid.metadata.ext,
                duration: vid.metadata.duration,
                albums: vid.metadata.albums,
                exif_vec: vid.metadata.exif_vec,
                alias: vid
                    .metadata
                    .alias
                    .into_iter()
                    .map(|a| FileModify {
                        file: a.file,
                        modified: a.modified as i64,
                        scan_time: a.scan_time as i64,
                    })
                    .collect(),
            };
            AbstractData::Video(VideoCombined { object, metadata })
        }
        AbstractDataOld::Album(alb) => {
            let object = ObjectSchema {
                id: alb.object.id,
                obj_type: ObjectType::Album,
                pending: alb.object.pending,
                thumbhash: alb.object.thumbhash,
                description: alb.object.description,
                tags: alb.object.tags,
                is_favorite: alb.object.is_favorite,
                is_archived: alb.object.is_archived,
                is_trashed: alb.object.is_trashed,
                #[allow(clippy::unnecessary_cast)]
                update_at: alb.metadata.last_modified_time as i64,
            };
            let metadata = AlbumMetadata {
                id: alb.metadata.id,
                title: alb.metadata.title,
                created_time: alb.metadata.created_time,
                start_time: alb.metadata.start_time,
                end_time: alb.metadata.end_time,
                last_modified_time: alb.metadata.last_modified_time,
                cover: alb.metadata.cover,
                item_count: alb.metadata.item_count,
                item_size: alb.metadata.item_size,
                share_list: alb
                    .metadata
                    .share_list
                    .into_iter()
                    .map(|(k, v)| {
                        (
                            k,
                            crate::public::structure::album::share::Share {
                                url: v.url,
                                description: v.description,
                                password: v.password,
                                show_metadata: v.show_metadata,
                                show_download: v.show_download,
                                show_upload: v.show_upload,
                                exp: v.exp as i64,
                            },
                        )
                    })
                    .collect(),
            };
            AbstractData::Album(AlbumCombined { object, metadata })
        }
    }
}

/// Checks if a migration is required by attempting to open the DB with the new and old drivers.
fn needs_migration() -> MigrationType {
    // 1. Check if V4 DB already exists
    if Path::new(NEW_DB_PATH).exists() {
        // Assume if it exists, it's correct (or at least we've already migrated).
        // You could add a schema check here if V4->V5 migrations happen later.
        println!(
            "[INFO] Found V4 database at {NEW_DB_PATH}. No migration needed."
        );
        return MigrationType::None;
    }

    // 2. If V4 doesn't exist, check for legacy V3/V2 at OLD_DB_PATH
    if !Path::new(OLD_DB_PATH).exists() {
        return MigrationType::None;
    }

    // Check V3/V4 (Redb 3.x)
    // We need to use catch_unwind because accessing a table with wrong schema panics in redb/bitcode.

    // 1. Check if it is already V4 (Current Schema) but just in the wrong location?
    // Or if we are re-running on a V4 DB.
    let is_v4 = std::panic::catch_unwind(|| {
        if let Ok(db) = redb::Database::open(OLD_DB_PATH) {
            let read_txn = db.begin_read().unwrap();
            if let Ok(table) = read_txn.open_table(DATA_TABLE) {
                if table.len().unwrap() > 0 {
                    // Attempt to read/decode to verify schema
                    if let Some((_k, v)) = table.first().unwrap() {
                        let _ = v.value();
                    }
                }
                return true;
            }
        }
        false
    });

    if let Ok(true) = is_v4 {
        println!(
            "[WARN] Found V4-compatible database at {OLD_DB_PATH}. Moving to {NEW_DB_PATH}."
        );
        if let Err(e) = std::fs::rename(OLD_DB_PATH, NEW_DB_PATH) {
            eprintln!("[ERROR] Failed to move V4 database: {e}");
        }
        return MigrationType::None;
    }

    // 2. Check if it is V3 (Old Schema - Missing update_at)
    let is_v3 = std::panic::catch_unwind(|| {
        if let Ok(db) = redb::Database::open(OLD_DB_PATH) {
            let read_txn = db.begin_read().unwrap();
            let table_def = redb::TableDefinition::<&str, AbstractDataOld>::new("database");
            if let Ok(table) = read_txn.open_table(table_def) {
                if table.len().unwrap() > 0 {
                    // Attempt to read/decode
                    if let Some((_k, v)) = table.first().unwrap() {
                        let _ = v.value();
                    }
                }
                return true;
            }
        }
        false
    });

    if let Ok(true) = is_v3 {
        return MigrationType::V3ToV4;
    }

    // Check V2
    // Defensive: Use catch_unwind as a safety net.
    let result = std::panic::catch_unwind(|| redb_old::Database::open(OLD_DB_PATH));

    match result {
        Ok(Ok(db)) => {
            if let Ok(txn) = db.begin_read() {
                let has_db_table = txn
                    .open_table(redb_old::TableDefinition::<&str, OldDatabase>::new(
                        "database",
                    ))
                    .is_ok();
                let has_album_table = txn
                    .open_table(redb_old::TableDefinition::<&str, OldAlbum>::new("album"))
                    .is_ok();

                if has_db_table || has_album_table {
                    return MigrationType::V2ToV3;
                }
                MigrationType::None
            } else {
                MigrationType::None
            }
        }
        _ => MigrationType::None,
    }
}

/// Executes the migration from v2 to v4
///
/// This requires user confirmation via stdin. It backs up the old database to `.bak`,
/// transforms all records, and creates a new database at the original path.
pub fn migrate() -> Result<()> {
    let migration_type = needs_migration();
    if let MigrationType::None = migration_type {
        return Ok(());
    }

    println!("========================================================");
    match migration_type {
        MigrationType::V2ToV3 => {
            println!(
                " DETECTED OLD DATABASE (V2 / redb 2.6.x) at {OLD_DB_PATH}"
            );
            println!(" A MIGRATION IS REQUIRED TO UPGRADE TO V4 (redb 3.1, schema V4)");
        }
        MigrationType::V3ToV4 => {
            println!(" DETECTED OLD SCHEMA (V3) at {OLD_DB_PATH}");
            println!(" A MIGRATION IS REQUIRED TO UPGRADE TO V4 (schema with update_at)");
        }
        MigrationType::None => {}
    }
    println!("========================================================");
    println!(" Please ensure you have BACKED UP your './db' folder.");
    println!("Type 'yes' to start migration:");

    let mut input = String::new();
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;

    if input.trim() != "yes" {
        println!("Migration cancelled.");
        std::process::exit(0);
    }

    println!("Starting migration...");

    let new_db =
        redb::Database::create(NEW_DB_PATH).context("Failed to create new database file.")?;
    let write_txn = new_db.begin_write()?;

    match migration_type {
        MigrationType::V2ToV3 => {
            let old_db = redb_old::Database::open(OLD_DB_PATH)
                .context("Failed to open old database file.")?;
            let read_txn = old_db.begin_read()?;

            let old_data_table = read_txn
                .open_table(redb_old::TableDefinition::<&str, OldDatabase>::new(
                    "database",
                ))?;
            let old_album_table =
                read_txn.open_table(redb_old::TableDefinition::<&str, OldAlbum>::new("album"))?;

            // Migrating DATA (Images/Videos)
            {
                let mut data_table = write_txn.open_table(DATA_TABLE)?;
                let total_items = old_data_table.len()?;
                println!("Found {total_items} items to migrate.");

                let mut processed_count = 0;
                let mut batch_buffer: Vec<OldDatabase> = Vec::with_capacity(BATCH_SIZE);

                let mut commit_batch = |batch: Vec<OldDatabase>| -> Result<()> {
                    // Optimization: Struct transformation and bitcode decoding are CPU-bound.
                    let transformed_batch: Vec<AbstractData> = batch
                        .into_par_iter()
                        .map(transform_database_to_abstract_data)
                        .collect();

                    for abstract_data in transformed_batch {
                        data_table.insert(abstract_data.hash().as_str(), &abstract_data)?;
                    }
                    Ok(())
                };

                for result in old_data_table.iter()? {
                    let (_, value) = result?;
                    batch_buffer.push(value.value());

                    if batch_buffer.len() >= BATCH_SIZE {
                        commit_batch(std::mem::take(&mut batch_buffer))?;
                        processed_count += BATCH_SIZE;
                        println!("Migrated {processed_count} / {total_items} items...");
                    }
                }

                if !batch_buffer.is_empty() {
                    let len = batch_buffer.len();
                    commit_batch(batch_buffer)?;
                    processed_count += len;
                }
                println!("Data migration completed. Total: {processed_count}");
            }

            // Migrating ALBUMS
            {
                let mut data_table = write_txn.open_table(DATA_TABLE)?;
                let total_albums = old_album_table.len()?;
                println!("Found {total_albums} albums to migrate.");

                let mut processed_count = 0;

                for result in old_album_table.iter()? {
                    let (_, value) = result?;
                    let abstract_data = transform_album_to_abstract_data(value.value());
                    data_table.insert(abstract_data.hash().as_str(), &abstract_data)?;

                    processed_count += 1;
                    if processed_count % 100 == 0 {
                        println!("Migrated {processed_count} / {total_albums} albums...");
                    }
                }
                println!("Album migration completed. Total: {processed_count}");
            }
            // Drop handle for rename
            drop(read_txn);
            drop(old_db);
        }
        MigrationType::V3ToV4 => {
            let old_db =
                redb::Database::open(OLD_DB_PATH).context("Failed to open old database file.")?;
            let read_txn = old_db.begin_read()?;

            let table_def = redb::TableDefinition::<&str, AbstractDataOld>::new("database");
            let old_data_table = read_txn.open_table(table_def)?;

            let mut data_table = write_txn.open_table(DATA_TABLE)?;
            let total_items = old_data_table.len()?;
            println!("Found {total_items} items to migrate (V3 -> V4).");

            let mut processed_count = 0;
            let mut batch_buffer: Vec<AbstractDataOld> = Vec::with_capacity(BATCH_SIZE);

            let mut commit_batch = |batch: Vec<AbstractDataOld>| -> Result<()> {
                let transformed_batch: Vec<AbstractData> =
                    batch.into_par_iter().map(transform_v30_to_v31).collect();

                for abstract_data in transformed_batch {
                    data_table.insert(abstract_data.hash().as_str(), &abstract_data)?;
                }
                Ok(())
            };

            for result in old_data_table.iter()? {
                let (_, value) = result?;
                batch_buffer.push(value.value());

                if batch_buffer.len() >= BATCH_SIZE {
                    commit_batch(std::mem::take(&mut batch_buffer))?;
                    processed_count += BATCH_SIZE;
                    println!("Migrated {processed_count} / {total_items} items...");
                }
            }

            if !batch_buffer.is_empty() {
                let len = batch_buffer.len();
                commit_batch(batch_buffer)?;
                processed_count += len;
            }
            println!("V3 -> V4 Migration completed. Total: {processed_count}");

            drop(read_txn);
            drop(old_db);
        }
        MigrationType::None => unreachable!(),
    }

    write_txn.commit()?;
    println!("Migration completed successfully.");

    // Rename OLD DB to .bak
    let backup_path = format!("{OLD_DB_PATH}.bak");
    std::fs::rename(OLD_DB_PATH, &backup_path)
        .context(format!("Failed to rename old DB to {backup_path}"))?;
    println!("Old database renamed to {backup_path}");

    // DO NOT RENAME NEW DB BACK TO OLD_DB_PATH.
    // We are keeping index_v4.redb as the new standard.
    println!("New database created at {NEW_DB_PATH}");

    Ok(())
}

// ==================================================================================
// Config Migration Logic
// ==================================================================================

/// Reads legacy configuration sources (config.json, .env, env vars) and constructs
/// the new nested `AppConfig` (`PublicConfig` + `PrivateConfig`).
pub fn construct_migrated_config() -> AppConfig {
    let mut config = AppConfig::default();

    // 1. Migrate from old `config.json`
    // Legacy files were flat. We map recognized fields to `config.public`.
    if let Ok(file) = File::open("config.json") {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct LegacyConfigJson {
            #[serde(default)]
            read_only_mode: bool,
            #[serde(default)]
            disable_img: bool,
            // Add other legacy fields here if they existed in the old json
        }

        if let Ok(old) = serde_json::from_reader::<_, LegacyConfigJson>(file) {
            config.public.read_only_mode = old.read_only_mode;
            config.public.disable_img = old.disable_img;
            println!("Migrated settings from legacy config.json into PublicConfig");
        }
    }

    // 1b. Migrate from Rocket.toml (legacy Rocket configuration)
    if let Ok(toml_content) = fs::read_to_string("Rocket.toml")
        && let Ok(toml_value) = toml_content.parse::<toml::Table>()
    {
        // Try to read from [default] section first, then root level
        let default_section = toml_value.get("default").and_then(|v| v.as_table());

        // Read port
        let port = default_section
            .and_then(|d| d.get("port"))
            .or_else(|| toml_value.get("port"))
            .and_then(toml::Value::as_integer)
            .map(|p| u16::try_from(p).unwrap_or(8000));

        if let Some(p) = port {
            config.public.port = p;
            println!("Migrated port {p} from Rocket.toml into PublicConfig");
        }

        // Read address
        let address = default_section
            .and_then(|d| d.get("address"))
            .or_else(|| toml_value.get("address"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        if let Some(addr) = address {
            config.public.address.clone_from(&addr);
            println!(
                "Migrated address {addr} from Rocket.toml into PublicConfig"
            );
        }
    }

    // 2. Migrate from Environment Variables (.env)
    dotenv().ok();

    if let Ok(pwd) = std::env::var("PASSWORD")
        && !pwd.trim().is_empty()
    {
        // Sensitive -> PrivateConfig
        config.private.password = pwd;
        println!("Migrated PASSWORD from environment into PrivateConfig");
    }

    if let Ok(key) = std::env::var("AUTH_KEY")
        && !key.trim().is_empty()
    {
        // Sensitive -> PrivateConfig
        config.private.auth_key = Some(key);
        println!("Migrated AUTH_KEY from environment into PrivateConfig");
    }

    if let Ok(hook) = std::env::var("DISCORD_HOOK_URL")
        && !hook.trim().is_empty()
    {
        // Non-sensitive -> PublicConfig
        config.public.discord_hook_url = Some(hook);
        println!("Migrated DISCORD_HOOK_URL from environment into PublicConfig");
    }

    // 3. Migrate Sync Paths
    if let Ok(sync_paths_str) = std::env::var("SYNC_PATH") {
        let mut count = 0;
        for path_str in sync_paths_str.split(',') {
            let path_str = path_str.trim();
            if !path_str.is_empty() {
                config.public.sync_paths.insert(PathBuf::from(path_str));
                count += 1;
            }
        }
        if count > 0 {
            println!(
                "Migrated {count} sync paths from SYNC_PATH into PublicConfig"
            );
        }
    }

    // Constraint: Prevent recursive syncing.
    if let Ok(upload_path) = fs::canonicalize(PathBuf::from("./upload")) {
        config
            .public
            .sync_paths
            .retain(|p| match fs::canonicalize(p) {
                Ok(c) => c != upload_path,
                Err(_) => p != &upload_path,
            });
    }

    config
}

/// Removes legacy configuration files (.env, Rocket.toml) to finalize migration.
pub fn cleanup_legacy_config_files() {
    if Path::new(".env").exists() {
        if let Err(e) = fs::remove_file(".env") {
            eprintln!("Failed to remove legacy .env file: {e}");
        } else {
            println!("Removed legacy .env file");
        }
    }

    if Path::new("Rocket.toml").exists() {
        if let Err(e) = fs::remove_file("Rocket.toml") {
            eprintln!("Failed to remove legacy Rocket.toml file: {e}");
        } else {
            println!("Removed legacy Rocket.toml file");
        }
    }
}
