use anyhow::Error;

use reqwest::blocking::Client;
use serde_json::json;

use crate::public::error::{AppError, ErrorKind};
use crate::public::structure::config::APP_CONFIG;

pub fn handle_error(error: Error) -> Error {
    error!("{:?}", error);
    if let Some(url) = &APP_CONFIG
        .get()
        .unwrap()
        .read()
        .unwrap()
        .public
        .discord_hook_url
    {
        send_discord_webhook(url, &format!("{error:?}"));
    }
    error
}

pub fn handle_app_error(error: &AppError) {
    // Filter out expected user errors to reduce noise
    match error.kind {
        ErrorKind::Auth
        | ErrorKind::PermissionDenied
        | ErrorKind::NotFound
        | ErrorKind::InvalidInput
        | ErrorKind::ReadOnlyMode => return,
        _ => {}
    }

    if let Some(url) = &APP_CONFIG
        .get()
        .unwrap()
        .read()
        .unwrap()
        .public
        .discord_hook_url
    {
        let debug_string = format!("{error}");
        send_discord_webhook(url, &debug_string);
    }
}

fn send_discord_webhook(webhook_url: &str, error_msg: &str) {
    let client = Client::new();
    let debug_string = format!("```rust\n{error_msg}\n```");
    let params = json!({ "content": debug_string });
    if let Err(e) = client.post(webhook_url).json(&params).send() {
        error!("Failed to send discord webhook: {}", e);
    }
}
