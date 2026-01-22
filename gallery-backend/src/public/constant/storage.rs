use directories::ProjectDirs;
use log::{error, info};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(Debug)]
pub struct EnvironmentStatus {
    pub is_portable: bool,
    pub data_path: PathBuf,
}

pub static ENVIROMENT_STATUS: OnceLock<EnvironmentStatus> = OnceLock::new();
// Backwards-compat alias for older name.
#[allow(unused_imports)]
pub use ENVIROMENT_STATUS as ENVIRONMENT_STATUS;

impl EnvironmentStatus {
    pub fn init() -> &'static Self {
        ENVIROMENT_STATUS.get_or_init(|| {
            let (is_portable, data_path) = Self::detect_environment();
            let status = EnvironmentStatus {
                is_portable,
                data_path,
            };

            info!(
                "ENVIROMENT_STATUS initialized: mode={}, root={}",
                if status.is_portable {
                    "portable"
                } else {
                    "installed"
                },
                status.data_path.display()
            );

            status
        })
    }

    fn detect_environment() -> (bool, PathBuf) {
        // 1. Check for portable marker or existing directories
        let portable_db = Path::new("db");
        let portable_object = Path::new("object");

        // If "db" or "object" folder exists in current directory, assume portable mode
        if portable_db.exists() || portable_object.exists() {
            info!("Portable mode detected (found ./db or ./object)");
            return (true, PathBuf::from("."));
        }

        // 2. Fallback to installed mode (AppData/XDG_DATA_HOME)
        if let Some(proj_dirs) = ProjectDirs::from("com", "urocissa", "urocissa") {
            let data_dir = proj_dirs.data_dir().to_path_buf();

            // Create the directory if it doesn't exist
            if !data_dir.exists() {
                if let Err(e) = std::fs::create_dir_all(&data_dir) {
                    error!(
                        "Failed to create data directory {}: {e}",
                        data_dir.display()
                    );
                    // Fallback to local if we can't write to AppData
                    return (true, PathBuf::from("."));
                }
            }

            info!(
                "Installed mode detected. Using data directory: {}",
                data_dir.display()
            );
            return (false, data_dir);
        }

        // 3. Fallback to current directory if ProjectDirs fails
        info!("Could not determine system data directory. Defaulting to portable mode.");
        (true, PathBuf::from("."))
    }
}

pub fn get_data_path() -> PathBuf {
    crate::public::structure::config::AppConfig::get_data_path()
}

pub fn get_config_path() -> PathBuf {
    crate::public::structure::config::AppConfig::get_config_path()
}
