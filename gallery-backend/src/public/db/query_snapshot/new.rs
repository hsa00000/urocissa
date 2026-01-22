use dashmap::DashMap;
use std::sync::LazyLock;

use super::{Prefetch, QuerySnapshot};

use crate::public::constant::storage::EnvironmentManager;

static QUERY_SNAPSHOT_IN_DISK: LazyLock<redb::Database> = LazyLock::new(|| {
    let path = EnvironmentManager::root_path().join("db/cache_db.redb");
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).unwrap();
        }
    }
    redb::Database::create(path).unwrap()
});

static QUERY_SNAPSHOT_IN_MEMORY: LazyLock<DashMap<u64, Prefetch>> = LazyLock::new(DashMap::new);

impl QuerySnapshot {
    pub fn new() -> Self {
        Self {
            in_disk: &QUERY_SNAPSHOT_IN_DISK,
            in_memory: &QUERY_SNAPSHOT_IN_MEMORY,
        }
    }
}
