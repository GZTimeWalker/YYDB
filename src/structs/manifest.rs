use std::{io::SeekFrom, path::PathBuf};

use async_trait::async_trait;
use avl::AvlTreeMap;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use super::{
    kvstore::*,
    lsm::{
        metadata::SSTableMeta,
        sstable::{SSTable, SSTableKey},
    },
    META_MAGIC_NUMBER,
};
use crate::{structs::table::TableId, utils::*};

#[derive(Debug)]
pub struct Manifest {
    io: IOHandler,
    factory: IOHandlerFactory,
    table_id: TableId,
    row_size: Option<u32>,
    tables: AvlTreeMap<SSTableKey, SSTable>,
}

impl Manifest {
    pub async fn new(table_name: impl Into<PathBuf>) -> Self {
        let table_name: PathBuf = table_name.into();
        let path = PathBuf::from(&table_name).join(".meta");

        debug!("Manifest path: {:?}", path);

        let io = IOHandler::new(&path).await.unwrap();

        if let Ok(manifest) = Manifest::from_io(&io).await {
            debug!("manifest from io ok");
            manifest
        } else {
            debug!("manifest from io error");
            Self {
                io,
                factory: IOHandlerFactory::new(&table_name),
                table_id: TableId::new(table_name.to_str().unwrap()),
                row_size: None,
                tables: AvlTreeMap::new(),
            }
        }
    }

    pub fn with_row_size(&mut self, row_size: u32) {
        if self.row_size.is_none() {
            self.row_size = Some(row_size);
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
        let mut size = self.io.size_on_disk().await?;
        for table in self.tables.values() {
            size += table.size_on_disk().await?;
        }
        Ok(size)
    }
}

impl WithIOConfig for Manifest {}

#[async_trait]
impl AsyncToIO for Manifest {
    async fn to_io(&self, io: &IOHandler) -> Result<()> {
        if self.row_size.is_none() {
            return Err(DbError::UnknownRowSize);
        }

        let mut io = io.inner().await?;
        io.seek(SeekFrom::Start(0)).await?;

        io.write_u32(META_MAGIC_NUMBER).await?;
        io.write_u64(self.table_id.0).await?;
        io.write_u32(self.row_size.unwrap()).await?;

        for (key, table) in self.tables.iter() {
            io.write_u64(key.0).await?;
            // write the meta only
            let meta = table.meta();
            let bytes = bincode::encode_to_vec(meta, Self::CONF)?;
            io.write_u32(bytes.len() as u32).await?;
            io.write(&bytes).await?;
        }

        io.flush().await?;
        Ok(())
    }
}

#[async_trait]
impl AsyncFromIO for Manifest {
    async fn from_io(io: &IOHandler) -> Result<Self> {
        io.seek(SeekFrom::Start(0)).await?;

        let table_name = io.base_dir().await;
        let factory = IOHandlerFactory::new(&table_name);

        let mut file_io = io.inner().await?;
        file_io.seek(SeekFrom::Start(0)).await?;

        let magic_number = file_io.read_u32().await?;
        if magic_number != META_MAGIC_NUMBER {
            return Err(DbError::InvalidMagicNumber);
        }

        let table_id = TableId(file_io.read_u64().await?);
        let row_size = file_io.read_u32().await?;

        let mut tables = AvlTreeMap::new();

        loop {
            if let Ok(rawkey) = file_io.read_u64().await {
                let key = SSTableKey(rawkey);
                let size = file_io.read_u32().await?;
                let mut bytes = vec![0; size as usize];
                file_io.read_exact(&mut bytes).await?;
                let meta: SSTableMeta = bincode::decode_from_slice(&bytes, Self::CONF)?.0;
                let table = SSTable::new(meta, &factory).await;
                tables.insert(key, table);
            } else {
                break;
            }
        }

        Ok(Self {
            io: io.clone().await?,
            table_id,
            row_size: Some(row_size),
            tables,
            factory,
        })
    }
}


impl Drop for Manifest {
    fn drop(&mut self) {
        debug!("Saving manifest...");

        futures::executor::block_on(async move {
            self.to_io(&self.io).await.unwrap();
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::structs::lsm::metadata::SSTableMeta;

    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let test_dir = "helper/sstable_test";

        std::fs::remove_dir_all(test_dir).ok();
        std::fs::create_dir_all(test_dir).unwrap();

        {
            let mut manifest = Manifest::new(test_dir).await;
            manifest.with_row_size(10);

            for _ in 0..7 {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;

                let rnd = rand::random::<u32>() % 3;
                let key = SSTableKey::new(rnd);
                let meta = SSTableMeta::new(key, rand::random::<u32>());

                manifest
                    .tables
                    .insert(meta.key, SSTable::new(meta, &manifest.factory).await);
            }

            assert_eq!(manifest.tables.len(), 7);
            manifest.to_io(&manifest.io).await.unwrap();
        }

        {
            let manifest = Manifest::new(test_dir).await;

            assert_eq!(manifest.tables.len(), 7);
            assert_eq!(manifest.table_id, TableId::new(test_dir));
            assert_eq!(manifest.row_size, Some(10));
        }

        Ok(())
    }
}
