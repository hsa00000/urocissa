// src/router/claims/claims.rs
use crate::public::structure::album::ResolvedShare;
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    Admin,
    Share(ResolvedShare),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Claims {
    pub role: Role,
    pub exp: u64,
}

impl Claims {
    pub fn new_admin() -> Self {
        let exp = (Utc::now().timestamp_millis() / 1000) as u64 + 14 * 86_400; // 14 days

        Self {
            role: Role::Admin,
            exp,
        }
    }

    pub fn new_share(resolved_share: ResolvedShare) -> Self {
        let exp = (Utc::now().timestamp_millis() / 1000) as u64 + 14 * 86_400; // 14 days

        Self {
            role: Role::Share(resolved_share),
            exp,
        }
    }
    pub fn is_admin(&self) -> bool {
        match &self.role {
            Role::Admin => true,
            _ => false,
        }
    }
    pub fn get_share(self) -> Option<ResolvedShare> {
        match self.role {
            Role::Share(share) => Some(share),
            _ => None,
        }
    }

    pub fn encode(&self) -> String {
        use crate::public::structure::config::APP_CONFIG;

        let config = APP_CONFIG.get().unwrap().read().unwrap();
        self.encode_with_key(&config.get_jwt_secret_key())
    }

    pub fn encode_with_key(&self, key: &[u8]) -> String {
        encode(&Header::default(), &self, &EncodingKey::from_secret(key))
            .expect("Failed to generate token")
    }
}
