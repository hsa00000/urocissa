use crate::router::{fairing::guard_auth::GuardAuth, AppResult};
use rocket::get;
use rocket::serde::json::Json;
use std::fs;
use std::path::{Path, PathBuf};

#[get("/get/path-completion?<path>")]
pub fn get_fs_completion(_auth: GuardAuth, path: Option<String>) -> AppResult<Json<Vec<String>>> {
    let query = path.unwrap_or_default();
    
    // If empty, return roots combined with current directory contents
    if query.trim().is_empty() {
        let mut results = get_roots();
        
        // Add contents of current directory (./)
        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(path_str) = path.to_str() {
                        results.push(path_str.to_string());
                    }
                }
            }
        }

        // Sort and limit
        results.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        results.truncate(50);
        
        return Ok(Json(results));
    }
    
    let path_obj = PathBuf::from(&query);
    
    // Determine directory to list and the prefix to filter by
    // If query ends with separator, we list the contents of that directory
    // If not, we list the parent directory and filter by the file name
    let (dir_to_read, prefix) = if query.ends_with('/') || (cfg!(windows) && query.ends_with('\\')) {
        (path_obj.as_path(), "")
    } else {
        match path_obj.parent() {
            Some(p) => {
                // If parent is empty (relative path "test"), it means current directory.
                // But for security/clarity in config, we might prefer absolute paths or roots.
                // However, let's just try to read it.
                if p.as_os_str().is_empty() {
                    // This happens if query is just "test". Parent is "".
                    // We probably want to search in roots? Or current dir?
                    // For a "server settings" context, listing root drives when p is empty/None is safer UX.
                    return Ok(Json(get_roots()));
                }
                (p, path_obj.file_name().and_then(|s| s.to_str()).unwrap_or(""))
            },
            None => {
                // No parent (e.g. "C:" or "/")
                // If it's effectively a root or we can't get parent, try listing roots or itself?
                 return Ok(Json(get_roots()));
            }
        }
    };

    let mut suggestions = Vec::new();
    
    if let Ok(entries) = fs::read_dir(dir_to_read) {
        for entry in entries.filter_map(|e| e.ok()) {
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
    }

    // Sort alphabetically
    suggestions.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    // Limit to avoid huge payloads
    suggestions.truncate(50);

    Ok(Json(suggestions))
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
