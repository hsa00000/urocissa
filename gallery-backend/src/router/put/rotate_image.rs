use crate::operations::indexation::generate_dynamic_image::generate_dynamic_image;
use crate::operations::indexation::generate_image_hash::{generate_phash, generate_thumbhash};
use crate::operations::indexation::generate_thumbnail::generate_thumbnail_for_image;
use crate::operations::open_db::open_data_table;
use crate::public::structure::abstract_data::AbstractData;
use crate::router::{AppResult, GuardResult};
use crate::tasks::batcher::flush_tree::FlushTreeTask;

use crate::router::fairing::guard_auth::GuardAuth;
use crate::router::fairing::guard_read_only_mode::GuardReadOnlyMode;
use crate::tasks::INDEX_COORDINATOR;
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use arrayvec::ArrayString;
use rocket::serde::{Deserialize, json::Json};

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct RotateImageRequest {
    /// Hash of the image to rotate
    pub hash: String,
}

#[put("/put/rotate-image", data = "<request>")]
pub async fn rotate_image(
    auth: GuardResult<GuardAuth>,
    read_only_mode: GuardResult<GuardReadOnlyMode>,
    request: Json<RotateImageRequest>,
) -> AppResult<()> {
    let _ = auth?;
    let _ = read_only_mode?;

    // Convert hash string to ArrayString
    let hash = ArrayString::<64>::from(&request.hash)
        .map_err(|_| anyhow!("Invalid hash length or format"))?;

    let abstract_data = tokio::task::spawn_blocking(move || -> Result<AbstractData> {
        let data_table = open_data_table();
        let access_guard = data_table
            .get(&*hash)
            .context("Failed to fetch DB record")?
            .ok_or_else(|| anyhow!("Hash not found"))?;

        let mut abstract_data = access_guard.value();

        // Only rotate images, not videos or albums
        if !matches!(abstract_data, AbstractData::Image(_)) {
            return Err(anyhow!("Only images can be rotated"));
        }

        // Load the compressed image (not the original)
        let compressed_path = abstract_data.compressed_path();
        let mut dyn_img = image::open(&compressed_path).context(format!(
            "Failed to load compressed image: {:?}",
            compressed_path
        ))?;

        // Rotate counter-clockwise (270 degrees clockwise = 90 degrees counter-clockwise)
        dyn_img = dyn_img.rotate270();

        // Swap width and height after rotation
        abstract_data.swap_width_height();

        // Generate and save the rotated thumbnail
        generate_thumbnail_for_image(&mut abstract_data, dyn_img.clone())
            .context("Failed to generate thumbnail for rotated image")?;

        // Update thumbhash and phash with the rotated image
        abstract_data.set_thumbhash(generate_thumbhash(&dyn_img));
        abstract_data.set_phash(generate_phash(&dyn_img));
        abstract_data.update_update_at();

        Ok(abstract_data)
    })
    .await
    .context("Failed to spawn blocking task")??;

    INDEX_COORDINATOR
        .execute_batch_waiting(FlushTreeTask::insert(vec![abstract_data]))
        .await
        .context("Failed to execute FlushTreeTask")?;

    info!("Image rotated successfully");
    Ok(())
}
