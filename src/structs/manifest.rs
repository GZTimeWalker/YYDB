use std::path::PathBuf;

use async_trait::async_trait;
use avl::AvlTreeMap;

use super::{
    kvstore::*,
    lsm::sstable::{SSTable, SSTableKey},
};
use crate::utils::*;

#[derive(Debug)]
pub struct Manifest {
    tables: AvlTreeMap<SSTableKey, SSTable>,
    factory: IOHandlerFactory,
    io: IOHandler,
}

impl Manifest {
    pub async fn new(table_name: impl Into<PathBuf>) -> Self {
        let table_name: PathBuf = table_name.into();
        let path = PathBuf::from(&table_name).join(".meta");

        debug!("Manifest path: {:?}", path);

        let io = IOHandler::new(&path).await.unwrap();

        // TODO: load manifest from disk

        Self {
            io,
            tables: AvlTreeMap::new(),
            factory: IOHandlerFactory::new(&table_name),
        }
    }
}

#[async_trait]
impl AsyncKVStoreRead for Manifest {
    async fn get(&self, key: u64) -> DataStore {
        for table in self.tables.values() {
            return match table.get(key).await {
                DataStore::Value(block) => DataStore::Value(block),
                DataStore::Deleted => DataStore::Deleted,
                DataStore::NotFound => continue,
            };
        }
        DataStore::NotFound
    }

    async fn len(&self) -> usize {
        let mut len = 0;
        for table in self.tables.values() {
            len += table.len().await;
        }
        len
    }
}

#[async_trait]
impl SizedOnDisk for Manifest {
    async fn size_on_disk(&self) -> Result<u64> {
        let mut size = 0;
        for table in self.tables.values() {
            size += table.size_on_disk().await?;
        }
        Ok(size)
    }
}

impl Drop for Manifest {
    fn drop(&mut self) {
        // TODO: save manifest to disk
    }
}

#[cfg(test)]
mod tests {
    use crate::structs::lsm::metadata::SSTableMeta;

    use super::*;

    #[tokio::test]
    async fn it_works() {
        let test_dir = "helper/sstable_test";

        std::fs::remove_dir_all(test_dir).ok();
        std::fs::create_dir_all(test_dir).unwrap();

        let mut manifest = Manifest::new(test_dir).await;

        for _ in 0..5 {
            let rnd = rand::random::<u32>() % 3;
            let mut meta = SSTableMeta::default();
            meta.key = SSTableKey::new(rnd);
            manifest
                .tables
                .insert(meta.key, SSTable::new(meta, &manifest.factory).await);
        }
        for (k, v) in manifest.tables.iter() {
            println!("{:?}: \n{:#?}", k, v);
        }
    }
}
