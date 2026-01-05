use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::response::{Redirect, content};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use crate::router::AppResult;
use crate::public::error::{AppError, ErrorKind, ResultExt};

pub static INDEX_HTML: LazyLock<String> = LazyLock::new(|| {
    fs::read_to_string("../gallery-frontend/dist/index.html").expect("Unable to read index.html")
});

#[get("/")]
pub fn redirect_to_photo() -> content::RawHtml<String> {
    content::RawHtml(INDEX_HTML.to_string())
}

#[get("/login")]
pub async fn login() -> AppResult<NamedFile> {
    NamedFile::open(Path::new("../gallery-frontend/dist/index.html"))
        .await
        .or_raise(|| (ErrorKind::IO, "Failed to open login page"))
}

#[get("/redirect-to-login")]
pub fn redirect_to_login() -> Redirect {
    Redirect::to(uri!("/login"))
}

#[get("/unauthorized")]
pub async fn unauthorized() -> Status {
    Status::Unauthorized
}

#[get("/home")]
pub async fn home() -> AppResult<NamedFile> {
    NamedFile::open(Path::new("../gallery-frontend/dist/index.html"))
        .await
        .or_raise(|| (ErrorKind::IO, "Failed to open home page"))
}

#[get("/home/view/<_path..>")]
pub async fn home_view(_path: PathBuf) -> AppResult<NamedFile> {
    NamedFile::open(Path::new("../gallery-frontend/dist/index.html"))
        .await
        .or_raise(|| (ErrorKind::IO, "Failed to open home view page"))
}

#[get("/favorite")]
pub async fn favorite() -> AppResult<NamedFile> {
    NamedFile::open(Path::new("../gallery-frontend/dist/index.html"))
        .await
        .or_raise(|| (ErrorKind::IO, "Failed to open favorite page"))
}

#[get("/favorite/view/<_path..>")]
pub async fn favorite_view(_path: PathBuf) -> AppResult<NamedFile> {
    NamedFile::open(Path::new("../gallery-frontend/dist/index.html"))
        .await
        .or_raise(|| (ErrorKind::IO, "Failed to open favorite view page"))
}

#[get("/albums")]
pub async fn albums() -> AppResult<NamedFile> {
    NamedFile::open(Path::new("../gallery-frontend/dist/index.html"))
        .await
        .or_raise(|| (ErrorKind::IO, "Failed to open albums page"))
}

#[get("/albums/view/<_path..>")]
pub async fn albums_view(_path: PathBuf) -> AppResult<NamedFile> {
    NamedFile::open(Path::new("../gallery-frontend/dist/index.html"))
        .await
        .or_raise(|| (ErrorKind::IO, "Failed to open albums view page"))
}

#[get("/<dynamic_album_id>")]
pub async fn album_page(dynamic_album_id: String) -> AppResult<NamedFile> {
    if dynamic_album_id.starts_with("album-") {
        NamedFile::open(Path::new("../gallery-frontend/dist/index.html"))
            .await
            .or_raise(|| (ErrorKind::IO, "Failed to open album page"))
    } else {
        Err(AppError::new(ErrorKind::NotFound, "Page not found"))
    }
}

#[get("/share/<_path..>")]
pub async fn share(_path: PathBuf) -> AppResult<NamedFile> {
    NamedFile::open(Path::new("../gallery-frontend/dist/index.html"))
        .await
        .or_raise(|| (ErrorKind::IO, "Failed to open share page"))
}

#[get("/archived")]
pub async fn archived() -> AppResult<NamedFile> {
    NamedFile::open(Path::new("../gallery-frontend/dist/index.html"))
        .await
        .or_raise(|| (ErrorKind::IO, "Failed to open archived page"))
}

#[get("/archived/view/<_path..>")]
pub async fn archived_view(_path: PathBuf) -> AppResult<NamedFile> {
    NamedFile::open(Path::new("../gallery-frontend/dist/index.html"))
        .await
        .or_raise(|| (ErrorKind::IO, "Failed to open archived view page"))
}

#[get("/trashed")]
pub async fn trashed() -> AppResult<NamedFile> {
    NamedFile::open(Path::new("../gallery-frontend/dist/index.html"))
        .await
        .or_raise(|| (ErrorKind::IO, "Failed to open trashed page"))
}

#[get("/trashed/view/<_path..>")]
pub async fn trashed_view(_path: PathBuf) -> AppResult<NamedFile> {
    NamedFile::open(Path::new("../gallery-frontend/dist/index.html"))
        .await
        .or_raise(|| (ErrorKind::IO, "Failed to open trashed view page"))
}

#[get("/all")]
pub async fn all() -> AppResult<NamedFile> {
    NamedFile::open(Path::new("../gallery-frontend/dist/index.html"))
        .await
        .or_raise(|| (ErrorKind::IO, "Failed to open all page"))
}

#[get("/all/view/<_path..>")]
pub async fn all_view(_path: PathBuf) -> AppResult<NamedFile> {
    NamedFile::open(Path::new("../gallery-frontend/dist/index.html"))
        .await
        .or_raise(|| (ErrorKind::IO, "Failed to open all view page"))
}

#[get("/videos")]
pub async fn videos() -> AppResult<NamedFile> {
    NamedFile::open(Path::new("../gallery-frontend/dist/index.html"))
        .await
        .or_raise(|| (ErrorKind::IO, "Failed to open videos page"))
}

#[get("/videos/view/<_path..>")]
pub async fn videos_view(_path: PathBuf) -> AppResult<NamedFile> {
    NamedFile::open(Path::new("../gallery-frontend/dist/index.html"))
        .await
        .or_raise(|| (ErrorKind::IO, "Failed to open videos view page"))
}

#[get("/tags")]
pub async fn tags() -> AppResult<NamedFile> {
    NamedFile::open(Path::new("../gallery-frontend/dist/index.html"))
        .await
        .or_raise(|| (ErrorKind::IO, "Failed to open tags page"))
}

#[get("/links")]
pub async fn links() -> AppResult<NamedFile> {
    NamedFile::open(Path::new("../gallery-frontend/dist/index.html"))
        .await
        .or_raise(|| (ErrorKind::IO, "Failed to open links page"))
}

#[get("/config")]
pub async fn config() -> AppResult<NamedFile> {
    NamedFile::open(Path::new("../gallery-frontend/dist/index.html"))
        .await
        .or_raise(|| (ErrorKind::IO, "Failed to open config page"))
}

#[get("/setting")]
pub async fn setting() -> AppResult<NamedFile> {
    NamedFile::open(Path::new("../gallery-frontend/dist/index.html"))
        .await
        .or_raise(|| (ErrorKind::IO, "Failed to open setting page"))
}

#[get("/favicon.ico")]
pub async fn favicon() -> AppResult<NamedFile> {
    NamedFile::open(Path::new("../gallery-frontend/dist/favicon.ico"))
        .await
        .or_raise(|| (ErrorKind::IO, "Failed to open favicon.ico"))
}

#[get("/registerSW.js")]
pub async fn sregister_sw() -> AppResult<NamedFile> {
    NamedFile::open(Path::new("../gallery-frontend/dist/registerSW.js"))
        .await
        .or_raise(|| (ErrorKind::IO, "Failed to open registerSW.js"))
}

#[get("/serviceWorker.js")]
pub async fn service_worker() -> AppResult<NamedFile> {
    NamedFile::open(Path::new("../gallery-frontend/dist/serviceWorker.js"))
        .await
        .or_raise(|| (ErrorKind::IO, "Failed to open serviceWorker.js"))
}

