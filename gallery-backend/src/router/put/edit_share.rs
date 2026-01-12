use crate::public::db::tree::TREE;
use crate::public::error::{AppError, ErrorKind, ResultExt};
use crate::public::structure::abstract_data::AbstractData;
use crate::public::structure::album::Share;
use crate::router::GuardResult;
use crate::router::fairing::guard_auth::GuardAuth;
use crate::router::fairing::guard_read_only_mode::GuardReadOnlyMode;
use crate::tasks::BATCH_COORDINATOR;
use crate::tasks::batcher::update_tree::UpdateTreeTask;
use crate::{public::constant::redb::DATA_TABLE, router::AppResult};

use arrayvec::ArrayString;
use redb::ReadableTable;
use rocket::serde::{Deserialize, json::Json};
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditShare {
    album_id: ArrayString<64>,
    share: Share,
}

#[put("/put/edit_share", format = "json", data = "<json_data>")]
pub async fn edit_share(
    auth: GuardResult<GuardAuth>,
    read_only_mode: GuardResult<GuardReadOnlyMode>,
    json_data: Json<EditShare>,
) -> AppResult<()> {
    let _ = auth?;
    let _ = read_only_mode?;
    tokio::task::spawn_blocking(move || -> Result<(), AppError> {
        let txn = TREE
            .in_disk
            .begin_write()
            .or_raise(|| (ErrorKind::Database, "Failed to begin transaction"))?;
        {
            let mut data_table = txn
                .open_table(DATA_TABLE)
                .or_raise(|| (ErrorKind::Database, "Failed to open data table"))?;

            let album_opt = data_table
                .get(json_data.album_id.as_str())
                .or_raise(|| (ErrorKind::Database, "Failed to get album"))?
                .and_then(|guard| {
                    let abstract_data = guard.value();
                    match abstract_data {
                        AbstractData::Album(album) => Some(album),
                        _ => None,
                    }
                });

            if let Some(mut album) = album_opt {
                album
                    .metadata
                    .share_list
                    .insert(json_data.share.url, json_data.share.clone());
                data_table
                    .insert(json_data.album_id.as_str(), AbstractData::Album(album))
                    .or_raise(|| (ErrorKind::Database, "Failed to update album"))?;
            }
        }
        txn.commit()
            .or_raise(|| (ErrorKind::Database, "Failed to commit transaction"))?;
        Ok(())
    })
    .await
    .or_raise(|| (ErrorKind::Internal, "Failed to join blocking task"))??;
    BATCH_COORDINATOR
        .execute_batch_waiting(UpdateTreeTask)
        .await
        .or_raise(|| (ErrorKind::Internal, "Failed to update tree"))?;
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteShare {
    album_id: ArrayString<64>,
    share_id: ArrayString<64>,
}

#[put("/put/delete_share", format = "json", data = "<json_data>")]
pub async fn delete_share(
    auth: GuardResult<GuardAuth>,
    read_only_mode: GuardResult<GuardReadOnlyMode>,
    json_data: Json<DeleteShare>,
) -> AppResult<()> {
    let _ = auth?;
    let _ = read_only_mode?;
    tokio::task::spawn_blocking(move || -> Result<(), AppError> {
        let txn = TREE
            .in_disk
            .begin_write()
            .or_raise(|| (ErrorKind::Database, "Failed to begin transaction"))?;
        {
            let mut data_table = txn
                .open_table(DATA_TABLE)
                .or_raise(|| (ErrorKind::Database, "Failed to open data table"))?;

            let album_opt = data_table
                .get(json_data.album_id.as_str())
                .or_raise(|| (ErrorKind::Database, "Failed to get album"))?
                .and_then(|guard| {
                    let abstract_data = guard.value();
                    match abstract_data {
                        AbstractData::Album(album) => Some(album),
                        _ => None,
                    }
                });

            if let Some(mut album) = album_opt {
                album.metadata.share_list.remove(&json_data.share_id);
                data_table
                    .insert(json_data.album_id.as_str(), AbstractData::Album(album))
                    .or_raise(|| (ErrorKind::Database, "Failed to update album"))?;
            }
        }
        txn.commit()
            .or_raise(|| (ErrorKind::Database, "Failed to commit transaction"))?;
        Ok(())
    })
    .await
    .or_raise(|| (ErrorKind::Internal, "Failed to join blocking task"))??;
    BATCH_COORDINATOR
        .execute_batch_waiting(UpdateTreeTask)
        .await
        .or_raise(|| (ErrorKind::Internal, "Failed to update tree"))?;
    Ok(())
}
