use async_trait::async_trait;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::utils::*;

use super::kvstore::*;

pub type MemStore = BTreeMap<u64, DataStore>;

#[derive(Debug)]
pub struct MemTable {
    mut_map: RwLock<MemStore>,
    lock_map: RwLock<MemStore>,
    io: IOHandler,
}

impl MemTable {
    pub async fn new(table_name: impl Into<PathBuf>) -> Self {
        let path: PathBuf = table_name.into().join(".cache");
        let io = IOHandler::new(&path).await.unwrap();

        debug!("MemTable path: {:?}", path);

        // TODO: load memtable from disk cache

        Self {
            mut_map: RwLock::new(BTreeMap::new()),
            lock_map: RwLock::new(BTreeMap::new()),
            io,
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
    async fn get(&self, key: u64) -> DataStore {
        if let Some(value) = self.mut_map.read().await.get(&key) {
            return value.clone();
        }

        if let Some(value) = self.lock_map.read().await.get(&key) {
            return value.clone();
        }

        DataStore::NotFound
    }

    async fn len(&self) -> usize {
        let lock_map_len = self.lock_map.read().await.len();
        let mut_map_len = self.mut_map.read().await.len();

        lock_map_len + mut_map_len
    }
}

#[async_trait]
impl AsyncKVStoreWrite for MemTable {
    async fn set(&self, key: u64, value: Vec<u8>) {
        self.mut_map
            .write()
            .await
            .insert(key, DataStore::Value(Arc::new(value)));
    }

    async fn delete(&self, key: u64) {
        self.mut_map.write().await.insert(key, DataStore::Deleted);
    }
}

impl Drop for MemTable {
    fn drop(&mut self) {
        debug!("Saving memtable...");

        futures::executor::block_on(async move {
            let mut mut_map = self.mut_map.write().await;
            let mut lock_map = self.lock_map.write().await;

            for (key, value) in lock_map.iter() {
                mut_map.insert(*key, value.clone());
            }

            lock_map.clear();
        });

        // TODO: flush memtable into disk cache
    }
}
