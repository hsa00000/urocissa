// src/router/claims/claims_hash.rs
use arrayvec::ArrayString;
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};

use crate::public::structure::config::APP_CONFIG;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimsHash {
    pub allow_original: bool,
    pub hash: ArrayString<64>,
    pub timestamp: i64,
    pub exp: u64,
}

impl ClaimsHash {
    pub fn new(hash: ArrayString<64>, timestamp: i64, allow_original: bool) -> Self {
        #[allow(clippy::cast_sign_loss)]
        let exp = (Utc::now().timestamp_millis() / 1000) as u64 + 300;

        Self {
            allow_original,
            hash,
            timestamp,
            exp,
        }
    }

    pub fn encode(&self) -> String {
        let secret_key = APP_CONFIG
            .get()
            .unwrap()
            .read()
            .unwrap()
            .get_jwt_secret_key();
        encode(
            &Header::default(),
            &self,
            &EncodingKey::from_secret(&secret_key),
        )
        .expect("Failed to generate token")
    }
}
