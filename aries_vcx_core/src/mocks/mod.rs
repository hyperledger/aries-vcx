use std::collections::HashSet;
use std::sync::RwLock;

lazy_static! {
    pub static ref ENABLED_MOCKS: RwLock<HashSet<String>> = RwLock::new(HashSet::new());
}

pub mod did_mocks;
pub mod pool_mocks;
