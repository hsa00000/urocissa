use directories::ProjectDirs;
use log::{error, info};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[cfg(test)]
use std::cell::RefCell;

#[derive(Debug)]
pub struct EnvironmentManager {
    pub is_portable: bool,
    pub root_path: PathBuf,
}

pub static ENVIRONMENT_MANAGER: OnceLock<EnvironmentManager> = OnceLock::new();

#[cfg(test)]
thread_local! {
    static TEST_ROOT_PATH_OVERRIDE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

impl EnvironmentManager {
    pub fn init() -> &'static Self {
        ENVIRONMENT_MANAGER.get_or_init(|| {
            let (is_portable, root_path) = Self::detect_environment();
            let manager = EnvironmentManager {
                is_portable,
                root_path,
            };

            info!(
                "ENVIRONMENT_MANAGER initialized: mode={}, root={}",
                if manager.is_portable {
                    "portable"
                } else {
                    "installed"
                },
                manager.root_path.display()
            );

            manager
        })
    }

    pub fn root_path() -> PathBuf {
        #[cfg(test)]
        if let Some(path) = TEST_ROOT_PATH_OVERRIDE.with(|p| p.borrow().clone()) {
            return path;
        }

        Self::init().root_path.clone()
    }

    pub fn config_path() -> PathBuf {
        Self::root_path().join("config.json")
    }

    pub fn db_dir() -> PathBuf {
        Self::root_path().join("db")
    }

    pub fn object_dir() -> PathBuf {
        Self::root_path().join("object")
    }

    pub fn object_imported_dir() -> PathBuf {
        Self::object_dir().join("imported")
    }

    pub fn object_compressed_dir() -> PathBuf {
        Self::object_dir().join("compressed")
    }

    pub fn upload_dir() -> PathBuf {
        Self::root_path().join("upload")
    }

    pub fn db_file_path(file_name: &str) -> PathBuf {
        Self::db_dir().join(file_name)
    }

    pub fn index_v4_db_path() -> PathBuf {
        Self::db_file_path("index_v4.redb")
    }

    pub fn index_v5_db_path() -> PathBuf {
        Self::db_file_path("index_v5.redb")
    }

    pub fn temp_db_path() -> PathBuf {
        Self::db_file_path("temp_db.redb")
    }

    pub fn cache_db_path() -> PathBuf {
        Self::db_file_path("cache_db.redb")
    }

    pub fn expire_db_path() -> PathBuf {
        Self::db_file_path("expire_db.redb")
    }

    fn hash_prefix_2(hash: &str) -> &str {
        &hash[0..2]
    }

    pub fn object_imported_prefix_dir(hash: &str) -> PathBuf {
        Self::object_imported_dir().join(Self::hash_prefix_2(hash))
    }

    pub fn object_compressed_prefix_dir(hash: &str) -> PathBuf {
        Self::object_compressed_dir().join(Self::hash_prefix_2(hash))
    }

    pub fn imported_file_path(hash: &str, ext: &str) -> PathBuf {
        Self::object_imported_prefix_dir(hash).join(format!("{hash}.{ext}"))
    }

    pub fn compressed_file_path(hash: &str, ext: &str) -> PathBuf {
        Self::object_compressed_prefix_dir(hash).join(format!("{hash}.{ext}"))
    }

    pub fn compressed_image_path(hash: &str) -> PathBuf {
        Self::compressed_file_path(hash, "jpg")
    }

    pub fn compressed_video_path(hash: &str) -> PathBuf {
        Self::compressed_file_path(hash, "mp4")
    }

    pub fn ensure_layout() -> std::io::Result<()> {
        std::fs::create_dir_all(Self::db_dir())?;
        std::fs::create_dir_all(Self::object_imported_dir())?;
        std::fs::create_dir_all(Self::object_compressed_dir())?;
        std::fs::create_dir_all(Self::upload_dir())?;

        Ok(())
    }

    #[cfg(test)]
    pub fn set_root_path_override_for_test(path: PathBuf) {
        TEST_ROOT_PATH_OVERRIDE.with(|p| {
            *p.borrow_mut() = Some(path);
        });
    }

    #[cfg(test)]
    pub fn clear_root_path_override_for_test() {
        TEST_ROOT_PATH_OVERRIDE.with(|p| {
            *p.borrow_mut() = None;
        });
    }

    fn detect_environment() -> (bool, PathBuf) {
        // 1. Check for portable marker or existing directories
        let portable_db = Path::new("db");
        let portable_object = Path::new("object");
        let portable_config = Path::new("config.json");

        // If "db" or "object" folder exists in current directory, assume portable mode.
        // Also treat an existing ./config.json as a portable marker.
        if portable_db.exists() || portable_object.exists() || portable_config.exists() {
            info!("Portable mode detected (found ./db, ./object, or ./config.json)");
            return (true, PathBuf::from("."));
        }

        // 2. Fallback to installed mode (AppData/XDG_DATA_HOME)
        if let Some(proj_dirs) = ProjectDirs::from("com", "urocissa", "urocissa") {
            let root_dir = proj_dirs.data_dir().to_path_buf();

            // Create the directory if it doesn't exist
            if !root_dir.exists() {
                if let Err(e) = std::fs::create_dir_all(&root_dir) {
                    error!(
                        "Failed to create data directory {}: {e}",
                        root_dir.display()
                    );
                    // Fallback to local if we can't write to AppData
                    return (true, PathBuf::from("."));
                }
            }

            info!(
                "Installed mode detected. Using data directory: {}",
                root_dir.display()
            );
            return (false, root_dir);
        }

        // 3. Fallback to current directory if ProjectDirs fails
        info!("Could not determine system data directory. Defaulting to portable mode.");
        (true, PathBuf::from("."))
    }
}
