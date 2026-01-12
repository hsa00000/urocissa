#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use crate::migration::v3_schema::AbstractData as AbstractDataOld;
use crate::public::constant::redb::DATA_TABLE;
use crate::public::structure::abstract_data::AbstractData;
use crate::public::structure::album::{
    combined::AlbumCombined, metadata::AlbumMetadata, share::Share as NewShare,
};
use crate::public::structure::common::FileModify;
use crate::public::structure::image::{combined::ImageCombined, metadata::ImageMetadata};
use crate::public::structure::object::{ObjectSchema, ObjectType};
use crate::public::structure::video::{combined::VideoCombined, metadata::VideoMetadata};
use anyhow::{Context, Result};
use chrono::Utc;
use rayon::prelude::*;
use redb::{ReadableDatabase, ReadableTable, ReadableTableMetadata};

/// Executes the migration from V3 to V4
pub fn migrate_v3_to_v4(old_db_path: &str, write_txn: &redb::WriteTransaction) -> Result<()> {
    let old_db = redb::Database::open(old_db_path).context("Failed to open old database file.")?;
    let read_txn = old_db.begin_read()?;

    let table_def = redb::TableDefinition::<&str, AbstractDataOld>::new("database");
    let old_data_table = read_txn.open_table(table_def)?;

    let batch_size = 5000;

    let mut data_table = write_txn.open_table(DATA_TABLE)?;
    let total_items = old_data_table.len()?;
    println!("Found {total_items} items to migrate (V3 -> V4).");

    let mut processed_count = 0;
    let mut batch_buffer: Vec<AbstractDataOld> = Vec::with_capacity(batch_size);

    let mut commit_batch = |batch: Vec<AbstractDataOld>| -> Result<()> {
        let transformed_batch: Vec<AbstractData> =
            batch.into_par_iter().map(transform_v3_to_v4).collect();

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
    println!("V3 -> V4 Migration completed. Total: {processed_count}");

    drop(read_txn);
    drop(old_db);

    Ok(())
}

pub fn transform_v3_to_v4(old_data: AbstractDataOld) -> AbstractData {
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
                            NewShare {
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
