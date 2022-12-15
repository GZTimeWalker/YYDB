use std::collections::BTreeMap;
use std::sync::Arc;

use tokio::sync::RwLock;

pub(crate) type DataBlock = Arc<Option<Vec<u8>>>;

pub(crate) type MemStore = BTreeMap<u64, DataBlock>;

#[derive(Debug)]
pub struct MemTable {
    mut_map: RwLock<MemStore>,
    lock_map: RwLock<MemStore>,
}

impl MemTable {
    pub fn new() -> Self {
        Self {
            mut_map: RwLock::new(BTreeMap::new()),
            lock_map: RwLock::new(BTreeMap::new()),
        }
    }

    pub async fn get(&self, key: u64) -> Option<DataBlock> {
        self.mut_map.read().await.get(&key).cloned()
    }

    pub async fn set(&self, key: u64, value: Vec<u8>) {
        self.mut_map.write().await.insert(key, Arc::new(Some(value)));
    }

    pub async fn delete(&self, key: u64) {
        self.mut_map.write().await.remove(&key);
    }

    pub async fn swap(&self) {
        let mut mut_map = self.mut_map.write().await;
        let mut lock_map = self.lock_map.write().await;

        lock_map.clear();
        std::mem::swap(&mut *mut_map, &mut *lock_map);
    }
}
