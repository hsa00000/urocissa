use dashmap::DashMap;
use std::sync::LazyLock;

use crate::public::structure::response::reduced_data::ReducedData;

use super::TreeSnapshot;

use crate::public::constant::storage::get_data_path;

static TREE_SNAPSHOT_IN_DISK: LazyLock<redb::Database> =
    LazyLock::new(|| {
        let path = get_data_path().join("db/temp_db.redb");
        redb::Database::create(path).unwrap()
    });

static TREE_SNAPSHOT_IN_MEMORY: LazyLock<DashMap<i64, Vec<ReducedData>>> =
    LazyLock::new(DashMap::new);

impl TreeSnapshot {
    pub fn new() -> Self {
        Self {
            in_disk: &TREE_SNAPSHOT_IN_DISK,
            in_memory: &TREE_SNAPSHOT_IN_MEMORY,
        }
    }
}
