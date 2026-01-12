use crate::router::{AppError, AppResult, ErrorKind, fairing::guard_auth::GuardAuth};
use rocket::get;
// use rocket::http::Status;
use rocket::serde::json::Json;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
pub struct FsCompletion {
    roots: Vec<String>,
    children: Vec<String>,
    is_default: bool,
}

#[get("/get/path-completion?<path>")]
pub fn get_fs_completion(_auth: GuardAuth, path: Option<String>) -> AppResult<Json<FsCompletion>> {
    let query = path.unwrap_or_default();

    // If empty, return roots separated from current directory contents
    if query.trim().is_empty() {
        let roots = get_roots();
        let mut children = Vec::new();

        // Add contents of current directory (./)
        // We ignore errors here as we have a fallback (roots)
        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries.filter_map(std::result::Result::ok) {
                let path = entry.path();
                if path.is_dir()
                    && let Some(path_str) = path.to_str()
                {
                    children.push(path_str.to_string());
                }
            }
        }

        // Sort children
        children.sort_by_key(|a| a.to_lowercase());
        children.truncate(50);

        return Ok(Json(FsCompletion {
            roots,
            children,
            is_default: true,
        }));
    }

    let path_obj = PathBuf::from(&query);

    // Determine directory to list and the prefix to filter by
    // If query ends with separator, we list the contents of that directory
    // If not, we list the parent directory and filter by the file name
    let (dir_to_read, prefix) = if query.ends_with('/') || (cfg!(windows) && query.ends_with('\\'))
    {
        (path_obj.as_path(), "")
    } else {
        match path_obj.parent() {
            Some(p) if !p.as_os_str().is_empty() => (
                p,
                path_obj.file_name().and_then(|s| s.to_str()).unwrap_or(""),
            ),
            _ => {
                // Parent is empty (e.g. "foo" or "C").
                // Search BOTH roots and current directory.

                let mut matches = Vec::new();

                // 1. Search Roots
                let roots = get_roots();
                for root in roots {
                    if root.to_lowercase().starts_with(&query.to_lowercase()) {
                        matches.push(root);
                    }
                }

                // 2. Search Current Directory
                if let Ok(entries) = fs::read_dir(".") {
                    for entry in entries.filter_map(std::result::Result::ok) {
                        let path = entry.path();
                        if path.is_dir() {
                            // Check prefix match
                            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                            if name.to_lowercase().starts_with(&query.to_lowercase())
                                && let Some(path_str) = path.to_str()
                            {
                                matches.push(path_str.to_string());
                            }
                        }
                    }
                }

                // Sort and limit
                matches.sort_by_key(|a| a.to_lowercase());
                matches.truncate(50);

                if matches.is_empty() {
                    return Err(AppError::new(ErrorKind::NotFound, "Directory not found"));
                }

                // We return everything in 'children' because 'roots' is specifically for
                // the "default view" (unfiltered list of drives).
                // When filtering, a flat list of matches is usually better UX.
                return Ok(Json(FsCompletion {
                    roots: vec![],
                    children: matches,
                    is_default: false,
                }));
            }
        }
    };

    let entries = fs::read_dir(dir_to_read).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AppError::new(ErrorKind::NotFound, "Directory not found")
        } else {
            AppError::from_err(ErrorKind::IO, e.into())
        }
    })?;

    let mut suggestions = Vec::new();

    for entry in entries.filter_map(std::result::Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            // Check prefix match (case-insensitive for better UX)
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if name.to_lowercase().starts_with(&prefix.to_lowercase()) {
                // Convert to string.
                if let Some(path_str) = path.to_str() {
                    // Normalize slashes? Rust Path usually keeps system separator.
                    suggestions.push(path_str.to_string());
                }
            }
        }
    }

    // Sort alphabetically
    suggestions.sort_by_key(|a| a.to_lowercase());
    // Limit to avoid huge payloads
    suggestions.truncate(50);

    // If we were searching (prefix is not empty) and found nothing, return 404.
    // This handles the case where the user typed a path that doesn't exist.
    if suggestions.is_empty() && !prefix.is_empty() {
        return Err(AppError::new(ErrorKind::NotFound, "Directory not found"));
    }

    Ok(Json(FsCompletion {
        roots: vec![],
        children: suggestions,
        is_default: false,
    }))
}

fn get_roots() -> Vec<String> {
    if cfg!(windows) {
        let mut roots = Vec::new();
        for b in b'A'..=b'Z' {
            let drive = format!("{}:\\", b as char);
            if Path::new(&drive).exists() {
                roots.push(drive);
            }
        }
        roots
    } else {
        // Unix roots
        vec!["/".to_string()]
    }
}
