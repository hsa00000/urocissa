// src/router/fairing/guard_hash.rs
use jsonwebtoken::{DecodingKey, decode};
use log::warn;
use rocket::Request;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::serde::json::Json;

use crate::router::claims::claims_hash::ClaimsHash;
use crate::router::claims::claims_timestamp::ClaimsTimestamp;
use crate::router::fairing::VALIDATION;
use crate::public::structure::config::APP_CONFIG;
use crate::router::GuardError;
use crate::public::error::{AppError, ErrorKind, ResultExt}; // Import AppError stuff
use serde::{Deserialize, Serialize};

use super::VALIDATION_ALLOW_EXPIRED;
use super::auth_utils::{extract_bearer_token, extract_hash_from_path, my_decode_token};

pub struct GuardHash;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for GuardHash {
    type Error = GuardError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token = match extract_bearer_token(req) {
            Ok(token) => token,
            Err(err) => {
                return Outcome::Error((
                    Status::Unauthorized,
                    AppError::from_err(ErrorKind::Auth, err)
                        .context("Bearer token extraction failed"),
                ));
            }
        };

        let claims: ClaimsHash = match my_decode_token(token, &VALIDATION) {
            Ok(claims) => claims,
            Err(err) => {
                return Outcome::Error((
                    Status::Unauthorized,
                    AppError::from_err(ErrorKind::Auth, err)
                        .context("JWT decoding failed"),
                ));
            }
        };

        let data_hash = match extract_hash_from_path(req) {
            Ok(hash) => hash,
            Err(err) => {
                return Outcome::Error((
                    Status::Unauthorized,
                    AppError::from_err(ErrorKind::Auth, err)
                        .context("Hash extraction failed"),
                ));
            }
        };

        // Compare hash in the token with the hash in the request path
        if data_hash != *claims.hash {
            warn!(
                "Hash does not match. Received: {}, Expected: {}.",
                data_hash, claims.hash
            );
            return Outcome::Error((
                Status::Unauthorized, 
                AppError::new(ErrorKind::Auth, "Hash does not match")
            ));
        }
        Outcome::Success(GuardHash)
    }
}

pub struct GuardHashOriginal;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for GuardHashOriginal {
    type Error = GuardError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token = match extract_bearer_token(req) {
            Ok(token) => token,
            Err(err) => {
                return Outcome::Error((
                    Status::Unauthorized,
                    AppError::from_err(ErrorKind::Auth, err)
                        .context("Bearer token extraction failed"),
                ));
            }
        };

        let claims: ClaimsHash = match my_decode_token(token, &VALIDATION) {
            Ok(claims) => claims,
            Err(err) => {
                return Outcome::Error((
                    Status::Unauthorized,
                    AppError::from_err(ErrorKind::Auth, err)
                        .context("JWT decoding failed"),
                ));
            }
        };

        if !claims.allow_original {
            warn!("Original hash access is not allowed.");
            return Outcome::Forward(Status::Unauthorized);
        }

        let data_hash = match extract_hash_from_path(req) {
            Ok(hash) => hash,
            Err(err) => {
                return Outcome::Error((
                    Status::Unauthorized,
                    AppError::from_err(ErrorKind::Auth, err)
                        .context("Hash extraction failed"),
                ));
            }
        };

        // Compare hash in the token with the hash in the request path
        if data_hash != *claims.hash {
            warn!(
                "Hash does not match. Received: {}, Expected: {}.",
                data_hash, claims.hash
            );
             return Outcome::Error((
                Status::Unauthorized, 
                AppError::new(ErrorKind::Auth, "Hash does not match")
            ));
        }
        Outcome::Success(GuardHashOriginal)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenewHashToken {
    pub expired_hash_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenewHashTokenReturn {
    pub token: String,
}

use crate::router::AppResult;

#[post("/post/renew-hash-token", format = "json", data = "<token_request>")]
pub async fn renew_hash_token(
    auth: TimestampGuardModified,
    token_request: Json<RenewHashToken>,
) -> AppResult<Json<RenewHashTokenReturn>> {
    tokio::task::spawn_blocking(move || {
        let expired_hash_token = token_request.into_inner().expired_hash_token;
        let token_data = match decode::<ClaimsHash>(
            &expired_hash_token,
            &DecodingKey::from_secret(&APP_CONFIG.get().unwrap().read().unwrap().get_jwt_secret_key()),
            &VALIDATION_ALLOW_EXPIRED,
        ) {
            Ok(data) => data,
            Err(err) => {
                warn!(
                    "Token renewal failed: unable to decode token. Error: {err:#?}"
                );
                return Err(AppError::new(ErrorKind::Auth, "Unauthorized: Invalid token"));
            }
        };

        if token_data.claims.timestamp != auth.timestamp_decoded {
            warn!(
                "Timestamp does not match. Received: {}, Expected: {}",
                token_data.claims.timestamp, auth.timestamp_decoded
            );
            return Err(AppError::new(ErrorKind::Auth, "Unauthorized: Timestamp mismatch"));
        }

        let claims = token_data.claims;
        let new_hash_claims = ClaimsHash::new(claims.hash, claims.timestamp, claims.allow_original);
        let new_hash_token = new_hash_claims.encode();

        Ok(Json(RenewHashTokenReturn {
            token: new_hash_token,
        }))
    })
    .await
    .or_raise(|| (ErrorKind::Internal, "Failed to join blocking task"))?
}


pub struct TimestampGuardModified {
    pub timestamp_decoded: i64,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for TimestampGuardModified {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let Ok(token) = extract_bearer_token(req) else {
            // Note: We can't access `err` easily with let-else if we just unwrap Ok.
            // But extract_bearer_token returns Result.
            // To preserve error message we need the match, or be clever.
            // Clippy suggestion was `let Ok(token) = ... else { ... }`
            // But we need to use `err` in the `Outcome::Error`.
            // Wait, if we use let-else, we lose the error variable unless we match it out.
            // `let Ok(token) = ...` destructures Ok.
            // If it is Err(err), the else block executes, but we don't have access to `err`.
            // So for these cases, `manual_let_else` might be WRONG if we need the error value.
            // But let's look at the clippy warning again.
            // It says "this could be rewritten as `let...else`".
            // If I rewrite it, I might lose the error context unless I re-extract it or use a different pattern.
            // Actually, if I use `let Ok(token) = ...` I can't get the error.
            // So I will ignore this specific instance if I need the error.
            // BUT, wait, for `GuardHash`, line 200: `let token = match extract_bearer_token(req) { Ok(token) => token, Err(_) => return Outcome::Forward(Status::Unauthorized) };`
            // That one (line 200) ignores the error! So that one IS valid for let-else.
            // Lines 27-36 USE the error. So clippy shouldn't be complaining about those?
            // Let's re-read the clippy output.
            // Warning at `src/router/fairing/guard_hash.rs:200:9` -> This is inside `TimestampGuardModified`.
            // Warning for `GuardHash` itself? I don't see it in the snippet I posted in thought block.
            // Ah, I see "warning: this could be rewritten as `let...else` --> src/router/fairing/guard_hash.rs:200:9".
            // Only line 200 is complained about!
            // Line 27 was NOT complained about in the log I read.
            // Okay, so I only fix line 200.
            return match extract_bearer_token(req) {
                Ok(token) => match my_decode_token::<ClaimsTimestamp>(token, &VALIDATION) {
                    Ok(claims) => Outcome::Success(TimestampGuardModified {
                        timestamp_decoded: claims.timestamp,
                    }),
                    Err(_) => Outcome::Forward(Status::Unauthorized),
                },
                Err(_) => Outcome::Forward(Status::Unauthorized),
            }
        };

        // Wait, the code at 200 is:
        /*
        let token = match extract_bearer_token(req) {
            Ok(token) => token,
            Err(_) => return Outcome::Forward(Status::Unauthorized),
        };
        */
        // I will change it to let-else.
        let Ok(_token) = extract_bearer_token(req) else {
            return Outcome::Forward(Status::Unauthorized);
        };

        let Ok(claims) = my_decode_token::<ClaimsTimestamp>(token, &VALIDATION) else {
             return Outcome::Forward(Status::Unauthorized);
        };

        Outcome::Success(TimestampGuardModified {
            timestamp_decoded: claims.timestamp,
        })
    }
}
