// src/router/get/get_config.rs

use rocket::get;
use rocket::http::ContentType;
use rocket::serde::json::Json;

use crate::public::structure::config::{AppConfig, APP_CONFIG};
use crate::router::fairing::guard_auth::GuardAuth;

#[get("/get/config")]
pub fn get_config_handler(_auth: GuardAuth) -> Json<AppConfig> {
    Json(APP_CONFIG.get().unwrap().read().unwrap().clone())
}

#[get("/get/config/export")]
pub fn export_config_handler(_auth: GuardAuth) -> (ContentType, String) {
    let config = APP_CONFIG.get().unwrap().read().unwrap();
    let json = serde_json::to_string_pretty(&*config).unwrap_or_default();
    (ContentType::JSON, json)
}