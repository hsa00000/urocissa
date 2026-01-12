// src/router/claims/claims_timestamp.rs
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};

use crate::public::structure::album::ResolvedShare;
use crate::public::structure::config::APP_CONFIG;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimsTimestamp {
    pub resolved_share_opt: Option<ResolvedShare>,
    pub timestamp: i64,
    pub exp: u64,
}

impl ClaimsTimestamp {
    pub fn new(resolved_share_opt: Option<ResolvedShare>, timestamp: i64) -> Self {
        #[allow(clippy::cast_sign_loss)]
        let exp = (Utc::now().timestamp_millis() / 1000) as u64 + 300;

        Self {
            resolved_share_opt,
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
