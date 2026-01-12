// src/router/get/get_config.rs

use rocket::get;
use rocket::http::ContentType;
use rocket::serde::json::Json;

// Import PublicConfig
use crate::public::structure::config::{APP_CONFIG, PublicConfig};
use crate::router::fairing::guard_auth::GuardAuth;
use crate::router::fairing::guard_share::GuardShare;
use serde::Serialize;

use crate::router::{AppResult, GuardResult};
// use crate::public::error::{AppError, ErrorKind, ResultExt};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicConfigResponse {
    #[serde(flatten)]
    pub public: PublicConfig,
    pub has_password: bool,
}

#[get("/get/config")]
pub fn get_config_handler(auth: GuardResult<GuardShare>) -> AppResult<Json<PublicConfigResponse>> {
    let _ = auth?;
    // Only return the public part
    let config = APP_CONFIG.get().unwrap().read().unwrap();
    let response = PublicConfigResponse {
        public: config.public.clone(),
        has_password: config.private.password.is_some(),
    };
    Ok(Json(response))
}

#[get("/get/config/export")]
pub fn export_config_handler(auth: GuardResult<GuardAuth>) -> AppResult<(ContentType, String)> {
    let _ = auth?;
    // Export remains full config for backup purposes
    let config = APP_CONFIG.get().unwrap().read().unwrap();
    let json = serde_json::to_string_pretty(&*config).unwrap_or_default();
    Ok((ContentType::JSON, json))
}
