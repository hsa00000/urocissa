// src/router/post/import_config.rs

use log::error;
use rocket::http::Status;
use rocket::post;
use rocket::serde::json::Json;

use crate::public::structure::config::AppConfig;
use crate::router::fairing::guard_auth::GuardAuth;
use crate::router::AppResult;
use crate::public::error::{AppError, ErrorKind};

#[post("/post/config/import", data = "<file>")]
pub fn import_config_handler(_auth: GuardAuth, file: Json<AppConfig>) -> AppResult<Status> {
    match AppConfig::update(file.into_inner()) {
        Ok(_) => Ok(Status::Ok),
        Err(e) => {
            error!("Import failed: {}", e);
            Err(AppError::from_err(ErrorKind::Internal, e))
        }
    }
}

