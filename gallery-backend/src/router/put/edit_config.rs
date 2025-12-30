// src/router/put/edit_config.rs

use log::error;
use rocket::http::Status;
use rocket::put;
use rocket::serde::json::Json;
use serde::Deserialize;

// Import PublicConfig
use crate::public::structure::config::{APP_CONFIG, AppConfig, PublicConfig};
use crate::router::fairing::guard_auth::GuardAuth;
use crate::router::fairing::guard_read_only_mode::GuardReadOnlyMode;
use crate::router::AppResult;
use anyhow::Result;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConfigRequest {
    #[serde(flatten)]
    pub public: PublicConfig,
    // Make password optional so it's only updated if provided
    pub password: Option<String>,
    pub auth_key: Option<String>,
}

#[put("/put/config", data = "<req>")]
pub fn update_config_handler(
    _auth: GuardAuth,
    read_only: Result<GuardReadOnlyMode>,
    req: Json<UpdateConfigRequest>,
) -> AppResult<Status> {
    let _ = read_only?;
    let req_data = req.into_inner();

    // 1. Get the current private config to preserve fields not being updated
    let mut current_private = {
        let read_lock = APP_CONFIG.get().unwrap().read().unwrap();
        read_lock.private.clone()
    };

    // 2. Update private fields if they are present in the request
    if let Some(pwd) = req_data.password {
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
        e
    })?;

    Ok(Status::Ok)
}
