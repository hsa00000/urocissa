#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use crate::migration::v2_schema::{Album as V2Album, Database as V2Database};
use crate::migration::v3_schema::{
    AbstractData, AlbumCombined, AlbumMetadata, FileModify, ImageCombined, ImageMetadata,
    ObjectSchema, ObjectType, Share as V3Share, VideoCombined, VideoMetadata,
};
use crate::migration::v3_v4;
use crate::public::constant::redb::DATA_TABLE;
use anyhow::{Context, Result};
use arrayvec::ArrayString;
use rayon::prelude::*;
use redb_old::{
    ReadableTable as OldReadableTable, ReadableTableMetadata as OldReadableTableMetadata,
};
use std::collections::HashMap;

const USER_DEFINED_DESCRIPTION: &str = "_user_defined_description";

/// Executes the migration from V2 (redb 2.6) to V4 (redb 3.1)
pub fn migrate_v2_to_v4(old_db_path: &str, write_txn: &redb::WriteTransaction) -> Result<()> {
    let old_db =
        redb_old::Database::open(old_db_path).context("Failed to open old database file.")?;
    let read_txn = old_db.begin_read()?;

    let old_data_table = read_txn.open_table(redb_old::TableDefinition::<&str, V2Database>::new(
        "database",
    ))?;
    let old_album_table =
        read_txn.open_table(redb_old::TableDefinition::<&str, V2Album>::new("album"))?;

    let batch_size = 5000;

    // Migrating DATA (Images/Videos)
    {
        let mut data_table = write_txn.open_table(DATA_TABLE)?;
        let total_items = old_data_table.len()?;
        println!("Found {total_items} items to migrate.");

        let mut processed_count = 0;
        let mut batch_buffer: Vec<V2Database> = Vec::with_capacity(batch_size);

        let mut commit_batch = |batch: Vec<V2Database>| -> Result<()> {
            let transformed_batch: Vec<crate::public::structure::abstract_data::AbstractData> =
                batch
                    .into_par_iter()
                    .map(|v2| {
                        let v3 = transform_v2_to_v3(v2);
                        v3_v4::transform_v3_to_v4(v3)
                    })
                    .collect();

            for abstract_data in transformed_batch {
                data_table.insert(abstract_data.hash().as_str(), &abstract_data)?;
            }
            Ok(())
        };

        for result in old_data_table.iter()? {
            let (_, value) = result?;
            batch_buffer.push(value.value());

            if batch_buffer.len() >= batch_size {
                commit_batch(std::mem::take(&mut batch_buffer))?;
                processed_count += batch_size;
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
            let v3 = transform_v2_album_to_v3(value.value());
            let abstract_data = v3_v4::transform_v3_to_v4(v3);
            data_table.insert(abstract_data.hash().as_str(), &abstract_data)?;

            processed_count += 1;
            if processed_count % 100 == 0 {
                println!("Migrated {processed_count} / {total_albums} albums...");
            }
        }
        println!("Album migration completed. Total: {processed_count}");
    }
    drop(read_txn);
    drop(old_db);

    Ok(())
}

pub fn transform_v2_to_v3(old_data: V2Database) -> AbstractData {
    let description = old_data.exif_vec.get(USER_DEFINED_DESCRIPTION).cloned();

    // Legacy Business Logic:
    // Tags starting with "_" (e.g., _favorite) were previously used as boolean flags.
    let mut tags = old_data.tag.clone();
    let is_favorite = tags.remove("_favorite");
    let is_archived = tags.remove("_archived");
    let is_trashed = tags.remove("_trashed");

    let alias: Vec<FileModify> = old_data
        .alias
        .iter()
        .map(|a| FileModify {
            file: a.file.clone(),
            modified: a.modified,
            scan_time: a.scan_time,
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

pub fn transform_v2_album_to_v3(old_album: V2Album) -> AbstractData {
    let description = old_album
        .user_defined_metadata
        .get(USER_DEFINED_DESCRIPTION)
        .and_then(|v| v.first())
        .cloned();

    let mut tags = old_album.tag.clone();
    let is_favorite = tags.remove("_favorite");
    let is_archived = tags.remove("_archived");
    let is_trashed = tags.remove("_trashed");

    let share_list: HashMap<ArrayString<64>, V3Share> = old_album
        .share_list
        .into_iter()
        .map(|(key, old_share)| {
            (
                key,
                V3Share {
                    url: old_share.url,
                    description: old_share.description,
                    password: old_share.password,
                    show_metadata: old_share.show_metadata,
                    show_download: old_share.show_download,
                    show_upload: old_share.show_upload,
                    exp: old_share.exp,
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
