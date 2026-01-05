// src/router/put/edit_config.rs

use log::error;
use rocket::http::Status;
use rocket::put;
use rocket::serde::json::Json;
use tokio::task::spawn_blocking;

// Import PublicConfig
use crate::public::structure::config::{AppConfig, PublicConfig, APP_CONFIG};
use crate::router::fairing::guard_auth::GuardAuth;
use crate::router::fairing::guard_read_only_mode::GuardReadOnlyMode;
use crate::router::{AppResult, GuardResult};
use crate::public::error::{AppError, ErrorKind, ResultExt};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConfigRequest {
    #[serde(flatten)]
    pub public: PublicConfig,
    // Make password optional so it's only updated if provided
    pub password: Option<String>,
    pub old_password: Option<String>,
    pub auth_key: Option<String>,
}

#[put("/put/config", data = "<req>")]
pub async fn update_config_handler(
    _auth: GuardAuth,
    read_only: GuardResult<GuardReadOnlyMode>,
    req: Json<UpdateConfigRequest>,
) -> AppResult<Status> {
    let _ = read_only?;
    let req_data = req.into_inner();

    spawn_blocking(move || -> Result<Status, AppError> {
        // 1. Get the current private config to preserve fields not being updated
        let mut current_private = {
            let read_lock = APP_CONFIG.get().unwrap().read().unwrap();
            read_lock.private.clone()
        };

        // 2. Update private fields if they are present in the request
        if let Some(pwd) = req_data.password {
            // Verify old password if changing password
            if req_data.old_password.as_ref() != Some(&current_private.password) {
                return Err(AppError::new(ErrorKind::Auth, "Incorrect current password"));
            }
            current_private.password = pwd;
        }
        if let Some(key) = req_data.auth_key {
            current_private.auth_key = Some(key);
        }

        // 3. Construct the new full config
        let new_full_config = AppConfig {
            public: req_data.public,
            private: current_private,
        };

        // 4. Update using the full config
        AppConfig::update(new_full_config).map_err(|e| {
            error!("Failed to update config: {}", e);
            AppError::from_err(ErrorKind::Internal, e)
        })?;

        Ok(Status::Ok)
    })
    .await
    .or_raise(|| (ErrorKind::Internal, "Task join error"))?
}

