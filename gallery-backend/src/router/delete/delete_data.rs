// src/router/delete/delete_data.rs
use crate::operations::open_db::{open_data_table, open_tree_snapshot_table};
use crate::process::transitor::index_to_abstract_data;
use crate::public::error::{AppError, ErrorKind, ResultExt};
use crate::public::structure::abstract_data::AbstractData;
use crate::router::fairing::guard_auth::GuardAuth;
use crate::router::fairing::guard_read_only_mode::GuardReadOnlyMode;
use crate::router::{AppResult, GuardResult};
use crate::tasks::actor::album::AlbumSelfUpdateTask;
use crate::tasks::batcher::flush_tree::FlushTreeTask;
use crate::tasks::batcher::update_tree::UpdateTreeTask;
use crate::tasks::{BATCH_COORDINATOR, INDEX_COORDINATOR};
use anyhow::Result;
use arrayvec::ArrayString;
use futures::future::try_join_all;
use rocket::serde::{Deserialize, json::Json};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteList {
    delete_list: Vec<usize>,
    timestamp: i64,
}

#[delete("/delete/delete-data", format = "json", data = "<json_data>")]
pub async fn delete_data(
    auth: GuardResult<GuardAuth>,
    read_only_mode: GuardResult<GuardReadOnlyMode>,
    json_data: Json<DeleteList>,
) -> AppResult<()> {
    let _ = auth?;
    let _ = read_only_mode?;
    let (abstract_data_to_remove, all_affected_album_ids) = tokio::task::spawn_blocking({
        let delete_list = json_data.delete_list.clone();
        let timestamp = json_data.timestamp;
        move || process_deletes(delete_list, timestamp)
    })
    .await
    .or_raise(|| (ErrorKind::Internal, "Failed to join blocking task"))??;

    BATCH_COORDINATOR
        .execute_batch_waiting(FlushTreeTask::remove(abstract_data_to_remove))
        .await
        .or_raise(|| (ErrorKind::Internal, "Failed to execute flush tree task"))?;

    BATCH_COORDINATOR
        .execute_batch_waiting(UpdateTreeTask)
        .await
        .or_raise(|| (ErrorKind::Internal, "Failed to execute update tree task"))?;

    try_join_all(
        all_affected_album_ids
            .into_iter()
            .map(|album_id| async move {
                INDEX_COORDINATOR
                    .execute_waiting(AlbumSelfUpdateTask::new(album_id))
                    .await
            }),
    )
    .await
    .or_raise(|| (ErrorKind::Internal, "Failed to update affected albums"))?;
    Ok(())
}

fn process_deletes(
    delete_list: Vec<usize>,
    timestamp: i64,
) -> Result<(Vec<AbstractData>, Vec<ArrayString<64>>), AppError> {
    let data_table = open_data_table();
    let tree_snapshot = open_tree_snapshot_table(timestamp)
        .or_raise(|| (ErrorKind::Database, "Failed to open tree snapshot"))?;

    let mut all_affected_album_ids = Vec::new();
    let mut abstract_data_to_remove = Vec::new();

    for index in delete_list {
        let abstract_data =
            index_to_abstract_data(&tree_snapshot, &data_table, index).or_raise(|| {
                (
                    ErrorKind::Database,
                    format!("Failed to retrieve data at index {index}"),
                )
            })?;

        let affected_albums = match &abstract_data {
            AbstractData::Image(img) => img.metadata.albums.iter().copied().collect(),
            AbstractData::Video(vid) => vid.metadata.albums.iter().copied().collect(),
            AbstractData::Album(alb) => vec![alb.object.id],
        };

        all_affected_album_ids.extend(affected_albums);
        abstract_data_to_remove.push(abstract_data);
    }

    Ok((abstract_data_to_remove, all_affected_album_ids))
}
