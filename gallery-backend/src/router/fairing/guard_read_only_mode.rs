// src/router/fairing/guard_read_only_mode.rs
use crate::public::error::{AppError, ErrorKind};
use crate::public::structure::config::APP_CONFIG;
use crate::router::GuardError;
use rocket::Request;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};

pub struct GuardReadOnlyMode;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for GuardReadOnlyMode {
    type Error = GuardError;
    async fn from_request(_req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if APP_CONFIG
            .get()
            .unwrap()
            .read()
            .unwrap()
            .public
            .read_only_mode
        {
            return Outcome::Error((
                Status::MethodNotAllowed,
                AppError::new(ErrorKind::ReadOnlyMode, "Read-only mode is enabled"),
            ));
        }

        Outcome::Success(GuardReadOnlyMode)
    }
}
