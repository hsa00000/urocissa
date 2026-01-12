use crate::public::constant::storage::get_data_path;
use std::fs;

pub fn initialize_file() {
    let root = get_data_path();

    {
        let db_path = root.join("db/temp_db.redb");
        if fs::metadata(&db_path).is_ok() {
            match fs::remove_file(&db_path) {
                Ok(()) => {
                    info!("Clear tree cache");
                }
                Err(_) => {
                    error!("Fail to delete cache data {db_path:?}");
                }
            }
        }
    }
    {
        let db_path = root.join("db/cache_db.redb");
        if fs::metadata(&db_path).is_ok() {
            match fs::remove_file(&db_path) {
                Ok(()) => {
                    info!("Clear query cache");
                }
                Err(_) => {
                    error!("Fail to delete cache data {db_path:?}");
                }
            }
        }
    }
    {
        let db_path = root.join("db/expire_db.redb");
        if fs::metadata(&db_path).is_ok() {
            match fs::remove_file(&db_path) {
                Ok(()) => {
                    info!("Clear expire table");
                }
                Err(_) => {
                    error!("Fail to delete expire table {db_path:?}");
                }
            }
        }
    }
}
