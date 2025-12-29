// src/router/put/edit_config.rs

use log::error;
use rocket::put;
use rocket::http::Status;
use rocket::serde::json::Json;

// Import PublicConfig
use crate::public::structure::config::{AppConfig, PublicConfig, APP_CONFIG};
use crate::router::fairing::guard_auth::GuardAuth;

#[put("/put/config", data = "<config>")]
pub fn update_config_handler(_auth: GuardAuth, config: Json<PublicConfig>) -> Result<Status, Status> {
    // 1. Get the current private config from memory
    let current_private = {
        let read_lock = APP_CONFIG.get().unwrap().read().unwrap();
        read_lock.private.clone()
    };

    // 2. Construct the new full config merging new public + old private
    let new_full_config = AppConfig {
        public: config.into_inner(),
        private: current_private,
    };

    // 3. Update using the full config
    match AppConfig::update(new_full_config) {
        Ok(_) => Ok(Status::Ok),
        Err(e) => {
            error!("Failed to update config: {}", e);
            Err(Status::InternalServerError)
        }
    }
}