use std::sync::LazyLock;

use super::Expire;

use crate::public::constant::storage::EnvironmentManager;

static EXPIRE_IN_DISK: LazyLock<redb::Database> = LazyLock::new(|| {
    let path = EnvironmentManager::root_path().join("db/expire_db.redb");
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).unwrap();
        }
    }
    redb::Database::create(path).unwrap()
});

impl Expire {
    pub fn new() -> Self {
        Expire {
            in_disk: &EXPIRE_IN_DISK,
        }
    }
}
