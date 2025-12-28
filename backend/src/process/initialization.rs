use crate::operations::initialization::{
    ffmpeg::check_ffmpeg_and_ffprobe, folder::initialize_folder, logger::initialize_logger,
    redb::initialize_file,
};
use crate::public::structure::config::AppConfig;
use tokio::sync::mpsc::UnboundedReceiver;

/// Initializes all core application subsystems.
///
/// # Returns
/// An `UnboundedReceiver` capturing early log events, intended for TUI dashboard visualization.
pub fn initialize() -> UnboundedReceiver<String> {
    let log_receiver = initialize_logger();

    // Config must be initialized first to ensure 'config.json' exists for subsequent subsystems.
    AppConfig::init();

    check_ffmpeg_and_ffprobe();
    initialize_folder();
    initialize_file();

    log_receiver
}
