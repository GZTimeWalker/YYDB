use async_trait::async_trait;
use tokio::sync::RwLock;

use super::kvstore::*;
use super::manifest::Manifest;
use super::mem::{DataBlock, MemTable};
use crate::structs::TABLE_FILE_SUFFIX;
use crate::utils::error::Result;
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct TableId(pub u64);

impl TableId {
    pub fn new(table_name: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        table_name.hash(&mut hasher);
        Self(hasher.finish())
    }
}

#[derive(Debug)]
pub struct Table {
    id: TableId,
    name: String,
    memtable: MemTable,
    manifest: Arc<RwLock<Manifest>>,
}

impl Table {
    pub fn open(table_name: String) -> Result<Table> {
        let table_name = format!("{}{}", table_name, TABLE_FILE_SUFFIX);

        info!("Opening table: {}", table_name);

        // open the table file, create if not exists
        File::create(&table_name)?;

        let table_name_ref = table_name.as_str();

        Ok(Table {
            id: TableId::new(table_name_ref),
            name: table_name.clone(),
            memtable: MemTable::new(),
            manifest: Arc::new(RwLock::new(Manifest::new(table_name_ref))),
        })
    }

    pub fn id(&self) -> TableId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        info!("Closing table: {}", self.name);
    }
}

#[async_trait]
impl AsyncKVStoreRead for Table {
    async fn get(&self, key: u64) -> Option<DataBlock> {
        self.memtable.get(key).await
    }

    async fn len(&self) -> usize {
        self.memtable.len().await + self.manifest.read().await.len().await
    }
}

#[async_trait]
impl AsyncKVStoreWrite for Table {
    async fn set(&self, key: u64, value: Vec<u8>) {
        self.memtable.set(key, value).await
    }

    async fn delete(&self, key: u64) {
        self.memtable.delete(key).await
    }
}

#[async_trait]
impl SizedOnDisk for Table {
    async fn size_on_disk(&self) -> Result<u64> {
        self.manifest.read().await.size_on_disk().await
    }
}
