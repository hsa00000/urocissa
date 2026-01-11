use crate::router::fairing::guard_auth::GuardAuth;
use crate::router::fairing::guard_read_only_mode::GuardReadOnlyMode;
use crate::router::{AppResult, GuardResult};
use crate::public::error::{AppError, ErrorKind};
use crate::tasks::BATCH_COORDINATOR;
use crate::tasks::batcher::update_tree::UpdateTreeTask;
use crate::{
    public::structure::abstract_data::AbstractData, tasks::batcher::flush_tree::FlushTreeTask,
};

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use log::info;

#[get("/put/generate_random_data?<number>")]
pub async fn generate_random_data(
    auth: GuardResult<GuardAuth>,
    read_only_mode: GuardResult<GuardReadOnlyMode>,
    number: usize,
) -> AppResult<()> {
    let _ = auth?;
    let _ = read_only_mode?;
    let data_list: Vec<AbstractData> = (0..number)
        .into_par_iter()
        .map(|_| AbstractData::generate_random_data())
        .collect();
    BATCH_COORDINATOR.execute_batch_detached(FlushTreeTask::insert(data_list));
    BATCH_COORDINATOR
        .execute_batch_waiting(UpdateTreeTask)
        .await
        .map_err(|e| AppError::new(ErrorKind::Internal, format!("Failed to update tree: {e}")))?;
    info!("Insert random data complete");
    Ok(())
}

