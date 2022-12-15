use async_trait::async_trait;
use std::collections::BTreeMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use super::kvstore::*;

pub type DataBlock = Arc<Option<Vec<u8>>>;

pub type MemStore = BTreeMap<u64, DataBlock>;

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

    pub async fn swap(&self) {
        let mut mut_map = self.mut_map.write().await;
        let mut lock_map = self.lock_map.write().await;

        lock_map.clear();
        std::mem::swap(&mut *mut_map, &mut *lock_map);
    }
}

#[async_trait]
impl AsyncKVStoreRead for MemTable {
    async fn get(&self, key: u64) -> Option<DataBlock> {
        let mut_map = self.mut_map.read().await;

        if let Some(value) = mut_map.get(&key) {
            return Some(value.clone());
        }

        let lock_map = self.lock_map.read().await;

        if let Some(value) = lock_map.get(&key) {
            return Some(value.clone());
        }

        None
    }

    async fn len(&self) -> usize {
        let lock_map = self.lock_map.read().await;
        let mut_map = self.mut_map.read().await;

        lock_map.len() + mut_map.len()
    }
}

#[async_trait]
impl AsyncKVStoreWrite for MemTable {
    async fn set(&self, key: u64, value: Vec<u8>) {
        let mut mut_map = self.mut_map.write().await;

        mut_map.insert(key, Arc::new(Some(value)));
    }

    async fn delete(&self, key: u64) {
        let mut mut_map = self.mut_map.write().await;

        mut_map.insert(key, Arc::new(None));
    }
}
