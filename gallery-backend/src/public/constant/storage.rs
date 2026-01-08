use std::path::PathBuf;
use std::sync::OnceLock;
use directories::ProjectDirs;
use std::path::Path;
use log::{info, error};

pub static DATA_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn get_data_path() -> &'static PathBuf {
    DATA_PATH.get_or_init(|| {
        // 1. Check for portable marker or existing directories
        // The user said: "first check portable db and object"
        
        let portable_db = Path::new("db");
        let portable_object = Path::new("object");
        
        // If "db" or "object" folder exists in current directory, assume portable mode
        if portable_db.exists() || portable_object.exists() {
            info!("Portable mode detected (found ./db or ./object)");
            return PathBuf::from(".");
        }

        // 2. Fallback to installed mode (AppData/XDG_DATA_HOME)
        if let Some(proj_dirs) = ProjectDirs::from("com", "urocissa", "urocissa") {
            let data_dir = proj_dirs.data_dir().to_path_buf();
            
            // Create the directory if it doesn't exist
            if !data_dir.exists() {
                 if let Err(e) = std::fs::create_dir_all(&data_dir) {
                     error!("Failed to create data directory {:?}: {}", data_dir, e);
                     // Fallback to local if we can't write to AppData
                     return PathBuf::from(".");
                 }
            }
            
            info!("Installed mode detected. Using data directory: {:?}", data_dir);
            return data_dir;
        }

        // 3. Fallback to current directory if ProjectDirs fails
        info!("Could not determine system data directory. Defaulting to portable mode.");
        PathBuf::from(".")
    })
}

