use crate::operations::initialization::{
    ffmpeg::check_ffmpeg_and_ffprobe, folder::initialize_folder, redb::initialize_file,
};

use crate::public::constant::storage::EnvironmentStatus;
use crate::public::structure::config::AppConfig;

/// Initializes all core application subsystems.
pub fn initialize() {
    EnvironmentStatus::init();

    // Config must be initialized first to ensure 'config.json' exists for subsequent subsystems.
    if let Err(e) = AppConfig::init() {
        eprintln!("Failed to initialize configuration: {e:#}");
        std::process::exit(1);
    }

    // Ensure storage folders exist before trying to download FFmpeg into them
    initialize_folder();

    check_ffmpeg_and_ffprobe();
    initialize_file();
}
