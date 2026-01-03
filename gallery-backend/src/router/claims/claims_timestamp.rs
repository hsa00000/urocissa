// src/router/claims/claims_timestamp.rs
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

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
        let exp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 300;

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
