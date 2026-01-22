use crate::operations::initialization::{ffmpeg::check_ffmpeg_and_ffprobe, redb::initialize_file};

use crate::public::constant::storage::EnvironmentManager;
use crate::public::structure::config::AppConfig;

/// Initializes all core application subsystems.
pub fn initialize() {
    EnvironmentManager::init();

    // Config must be initialized first to ensure 'config.json' exists for subsequent subsystems.
    if let Err(e) = AppConfig::init() {
        eprintln!("Failed to initialize configuration: {e:#}");
        std::process::exit(1);
    }

    // Ensure storage folders exist before trying to download FFmpeg into them
    if let Err(e) = EnvironmentManager::ensure_layout() {
        eprintln!("Failed to initialize storage layout: {e}");
        std::process::exit(1);
    }

    check_ffmpeg_and_ffprobe();
    initialize_file();
}

#[cfg(test)]
mod tests {
    use super::initialize;
    use crate::public::constant::storage::EnvironmentManager;
    use std::path::PathBuf;

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let pid = std::process::id();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{pid}-{nanos}"))
    }

    struct DataPathOverrideGuard;

    impl Drop for DataPathOverrideGuard {
        fn drop(&mut self) {
            EnvironmentManager::clear_root_path_override_for_test();
        }
    }

    #[test]
    fn initialize_creates_fresh_layout_in_isolated_root() {
        let root = unique_temp_dir("urocissa-init");
        std::fs::create_dir_all(&root).expect("create temp root");

        EnvironmentManager::set_root_path_override_for_test(root.clone());
        let _guard = DataPathOverrideGuard;

        // Should behave like a clean first run: create config.json and required folders.
        initialize();

        assert!(root.join("config.json").exists());
        assert!(EnvironmentManager::db_dir().is_dir());
        assert!(EnvironmentManager::object_imported_dir().is_dir());
        assert!(EnvironmentManager::object_compressed_dir().is_dir());
        assert!(EnvironmentManager::upload_dir().is_dir());

        let _ = std::fs::remove_dir_all(&root);
    }
}
