use async_trait::async_trait;
use tokio::sync::RwLock;

use super::kvstore::*;
use super::manifest::Manifest;
use super::mem::MemTable;
use crate::utils::*;
use std::collections::hash_map::DefaultHasher;
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
    pub async fn open(table_name: String) -> Result<Table> {
        info!("Opening table: {}", table_name);

        let table_name = table_name.as_str();
        std::fs::create_dir_all(table_name)?;

        Ok(Table {
            id: TableId::new(table_name),
            name: table_name.to_string(),
            memtable: MemTable::new(table_name).await,
            manifest: Arc::new(RwLock::new(Manifest::new(table_name).await)),
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
    async fn get(&self, key: u64) -> DataStore {
        match self.memtable.get(key).await {
            DataStore::Value(value) => return DataStore::Value(value),
            DataStore::Deleted => return DataStore::Deleted,
            DataStore::NotFound => (),
        }

        let manifest = self.manifest.read().await;

        manifest.get(key).await
    }

    async fn len(&self) -> usize {
        self.memtable.len().await + self.manifest.read().await.len().await
    }
}

#[async_trait]
impl AsyncKVStoreWrite for Table {
    async fn set(&self, key: u64, value: Vec<u8>) {
        self.manifest
            .write()
            .await
            .with_row_size(value.len() as u32);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::error::Result;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let test_dir = "helper/table_test";

        std::fs::remove_dir_all(test_dir).ok();
        std::fs::create_dir_all(test_dir).unwrap();

        let table = Table::open(test_dir.to_string()).await?;

        assert_eq!(table.name(), test_dir);
        assert_eq!(table.id(), TableId::new(test_dir));

        let key = 1;
        let value = vec![1, 3, 5, 7];

        table.set(key, value.clone()).await;

        if let DataStore::Value(v) = table.get(key).await {
            assert_eq!(v.as_ref(), &value);
        } else {
            panic!("Value not found");
        }

        table.delete(key).await;

        assert_eq!(table.get(key).await, DataStore::Deleted);

        assert_eq!(table.get(2).await, DataStore::NotFound);

        Ok(())
    }
}
