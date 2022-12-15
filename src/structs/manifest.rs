use async_trait::async_trait;
use avl::AvlTreeMap;

use crate::utils::io_handler::IOHandlerFactory;
use crate::utils::error::Result;
use super::{lsm::sstable::{SSTable, SSTableKey}, kvstore::{SizedOnDisk, AsyncKVStoreRead}, mem::DataBlock};

#[derive(Debug)]
pub struct Manifest {
    tables: AvlTreeMap<SSTableKey, SSTable>,
    factory: IOHandlerFactory
}

impl Manifest {
    pub fn new(table_name: &str) -> Self {
        Self {
            tables: AvlTreeMap::new(),
            factory: IOHandlerFactory::new(table_name)
        }
    }
}

#[async_trait]
impl AsyncKVStoreRead for Manifest {
    async fn get(&self, key: u64) -> Option<DataBlock> {
        let mut result = None;
        for table in self.tables.values() {
            if let Some(block) = table.get(key).await {
                result = Some(block);
                break;
            }
        }
        result
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


#[cfg(test)]
mod tests {
    use crate::structs::lsm::metadata::SSTableMeta;

    use super::*;

    #[tokio::test]
    async fn it_works() {
        std::fs::remove_dir_all("helper/sstable").ok();

        let mut manifest = Manifest {
            tables: AvlTreeMap::new(),
            factory: IOHandlerFactory::new("helper/sstable")
        };
        for _ in 0..10 {
            let rnd = rand::random::<u32>() % 6;
            let mut meta = SSTableMeta::default();
            meta.key = SSTableKey::new(rnd);
            manifest.tables.insert(
                meta.key,
                SSTable::new(meta, &manifest.factory).await
            );
        }
        for (k, v) in manifest.tables.iter() {
            println!("{:?}: \n{:#?}", k, v);
        }
    }
}
