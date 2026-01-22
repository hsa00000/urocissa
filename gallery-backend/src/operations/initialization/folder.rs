use crate::public::constant::storage::EnvironmentStatus;
use log::info;

pub fn initialize_folder() {
    let root = EnvironmentStatus::get_data_path();
    info!("Storage root initialized at: {}", root.display());
    std::fs::create_dir_all(root.join("db")).unwrap();

    std::fs::create_dir_all(root.join("object/imported")).unwrap();
    std::fs::create_dir_all(root.join("object/compressed")).unwrap();
    std::fs::create_dir_all(root.join("upload")).unwrap();
}
