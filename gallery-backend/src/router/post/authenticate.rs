use rocket::post;
use rocket::serde::json::Json;

use crate::public::structure::config::APP_CONFIG;
use crate::router::claims::claims::Claims;
use crate::router::{AppError, AppResult, ErrorKind};

#[post("/post/authenticate", data = "<password>")]
pub fn authenticate(password: Json<String>) -> AppResult<Json<String>> {
    // Trim input password to match storage behavior
    let input_password = password.into_inner().trim().to_string();

    let current_password = APP_CONFIG
        .get()
        .unwrap()
        .read()
        .unwrap()
        .private
        .password
        .clone();

    let is_valid = match current_password {
        Some(pwd) => input_password == pwd,
        None => true,
    };

    if is_valid {
        let token = Claims::new_admin().encode();
        Ok(Json(token))
    } else {
        Err(AppError::new(ErrorKind::Auth, "Invalid password").context("Authentication failed"))
    }
}
