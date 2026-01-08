use std::sync::LazyLock;

use super::Expire;

use crate::public::constant::storage::get_data_path;

static EXPIRE_IN_DISK: LazyLock<redb::Database> =
    LazyLock::new(|| {
        let path = get_data_path().join("db/expire_db.redb");
        redb::Database::create(path).unwrap()
    });

impl Expire {
    pub fn new() -> Self {
        Expire {
            in_disk: &EXPIRE_IN_DISK,
        }
    }
}
