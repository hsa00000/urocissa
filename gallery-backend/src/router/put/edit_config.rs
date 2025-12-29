// src/router/put/edit_config.rs

use log::error;
use rocket::put;
use rocket::http::Status;
use rocket::serde::json::Json;

use crate::public::structure::config::AppConfig;
use crate::router::fairing::guard_auth::GuardAuth;

#[put("/put/config", data = "<config>")]
pub fn update_config_handler(_auth: GuardAuth, config: Json<AppConfig>) -> Result<Status, Status> {
    match AppConfig::update(config.into_inner()) {
        Ok(_) => Ok(Status::Ok),
        Err(e) => {
            error!("Failed to update config: {}", e);
            Err(Status::InternalServerError)
        }
    }
}