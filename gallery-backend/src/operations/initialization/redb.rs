use crate::public::constant::storage::EnvironmentManager;
use std::fs;

pub fn initialize_file() -> Result<(), String> {
    {
        let db_path = EnvironmentManager::temp_db_path();
        if fs::metadata(&db_path).is_ok() {
            match fs::remove_file(&db_path) {
                Ok(()) => {
                    info!("Clear tree cache");
                }
                Err(e) => {
                    return Err(format!("Fail to delete cache data {db_path:?}: {e}"));
                }
            }
        }
    }
    {
        let db_path = EnvironmentManager::cache_db_path();
        if fs::metadata(&db_path).is_ok() {
            match fs::remove_file(&db_path) {
                Ok(()) => {
                    info!("Clear query cache");
                }
                Err(e) => {
                    return Err(format!("Fail to delete cache data {db_path:?}: {e}"));
                }
            }
        }
    }
    {
        let db_path = EnvironmentManager::expire_db_path();
        if fs::metadata(&db_path).is_ok() {
            match fs::remove_file(&db_path) {
                Ok(()) => {
                    info!("Clear expire table");
                }
                Err(e) => {
                    return Err(format!("Fail to delete expire table {db_path:?}: {e}"));
                }
            }
        }
    }

    Ok(())
}
