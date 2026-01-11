pub mod new;
pub mod read_tags;

use crate::public::structure::response::database_timestamp::DatabaseTimestamp;
use std::sync::{atomic::AtomicI64, Arc, LazyLock, RwLock};

pub struct Tree {
    pub in_disk: &'static redb::Database,
    pub in_memory: &'static Arc<RwLock<Vec<DatabaseTimestamp>>>,
}

pub static TREE: LazyLock<Tree> = LazyLock::new(Tree::new);

pub static VERSION_COUNT_TIMESTAMP: AtomicI64 = AtomicI64::new(0);
