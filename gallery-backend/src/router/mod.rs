pub mod claims;
pub mod delete;
pub mod fairing;
pub mod get;
pub mod post;
pub mod put;

// Re-export the designed error type
pub use crate::public::error::{AppError, ErrorKind, ResultExt};

// Define AppResult using the new error type
pub type AppResult<T> = Result<T, AppError>;

// GuardResult using the same error type
pub type GuardResult<T> = Result<T, AppError>;

// GuardError alias
pub type GuardError = AppError;
