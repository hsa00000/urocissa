// src/router/get/get_config.rs

use rocket::get;
use rocket::http::ContentType;
use rocket::serde::json::Json;

// Import PublicConfig
use crate::public::structure::config::{PublicConfig, APP_CONFIG};
use crate::router::fairing::guard_auth::GuardAuth;

#[get("/get/config")]
pub fn get_config_handler(_auth: GuardAuth) -> Json<PublicConfig> {
    // Only return the public part
    Json(APP_CONFIG.get().unwrap().read().unwrap().public.clone())
}

#[get("/get/config/export")]
pub fn export_config_handler(_auth: GuardAuth) -> (ContentType, String) {
    // Export remains full config for backup purposes
    let config = APP_CONFIG.get().unwrap().read().unwrap();
    let json = serde_json::to_string_pretty(&*config).unwrap_or_default();
    (ContentType::JSON, json)
}