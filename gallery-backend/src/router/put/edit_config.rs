// src/router/put/edit_config.rs

use log::error;
use rocket::http::Status;
use rocket::put;
use rocket::serde::json::Json;
use tokio::task::spawn_blocking;

// Import PublicConfig
use crate::public::error::{AppError, ErrorKind, ResultExt};
use crate::public::structure::config::{APP_CONFIG, AppConfig, PublicConfig};
use crate::router::fairing::guard_auth::GuardAuth;
use crate::router::fairing::guard_read_only_mode::GuardReadOnlyMode;
use crate::router::{AppResult, GuardResult};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConfigRequest {
    #[serde(flatten)]
    pub public: PublicConfig,
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
            error!("Failed to update config: {e}");
            AppError::from_err(ErrorKind::Internal, e)
        })?;

        Ok(Status::Ok)
    })
    .await
    .or_raise(|| (ErrorKind::Internal, "Task join error"))?
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePasswordRequest {
    pub password: Option<String>,
    pub old_password: Option<String>,
}

#[put("/put/config/password", data = "<req>")]
pub async fn update_password_handler(
    _auth: GuardAuth,
    read_only: GuardResult<GuardReadOnlyMode>,
    req: Json<UpdatePasswordRequest>,
) -> AppResult<Status> {
    let _ = read_only?;
    let req_data = req.into_inner();

    spawn_blocking(move || -> Result<Status, AppError> {
        // 1. Get current config
        let mut current_config = APP_CONFIG.get().unwrap().read().unwrap().clone();
        
        // 2. Verify old password
        if req_data.old_password != current_config.private.password {
            // Using ErrorKind::InvalidInput (HTTP 400) to prevent frontend redirect (which happens on 401)
            return Err(AppError::new(ErrorKind::InvalidInput, "Incorrect current password"));
        }

        // 3. Update password
        if let Some(pwd) = req_data.password {
             let trimmed_pwd = pwd.trim().to_string();
             if trimmed_pwd.is_empty() {
                 current_config.private.password = None;
             } else {
                 current_config.private.password = Some(trimmed_pwd);
             }
        } else {
            // Explicitly requested to remove password?
            // If the frontend sends null/None, it usually means "don't change" or "remove"?
            // In our previous logic, we used empty string to remove.
            // Let's stick to: "If you call this endpoint, you are updating the password."
            // If `password` is None, we'll treat it as "Remove password" for safety/clarity, 
            // OR we can decide based on frontend.
            // Let's assume frontend sends Some("") to remove. 
            // If frontend sends None, we do nothing? No, this endpoint is specific for updating password.
            // Let's assume:
            // Some(pwd) -> update (or remove if empty)
            // None -> remove? Or Error?
            // Let's default to "None = Remove" to be consistent with "optional string".
             current_config.private.password = None;
        }

        // 4. Update
        AppConfig::update(current_config).map_err(|e| {
            error!("Failed to update config: {e}");
            AppError::from_err(ErrorKind::Internal, e)
        })?;

        Ok(Status::Ok)
    })
    .await
    .or_raise(|| (ErrorKind::Internal, "Task join error"))?
}

