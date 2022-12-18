use async_compression::Level;
use async_trait::async_trait;
use avl::AvlTreeMap;
use std::{collections::VecDeque, io::SeekFrom, path::PathBuf, sync::Arc};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use super::{kvstore::*, lsm::*, META_MAGIC_NUMBER};
use crate::{structs::table::TableId, utils::*};

#[derive(Debug)]
pub struct Manifest {
    io: IOHandler,
    tables: AvlTreeMap<SSTableKey, Arc<SSTable>>,

    pub factory: IOHandlerFactory,
    pub table_id: TableId,
    pub row_size: u32,
    pub bloom_filter: BloomFilter,
}

impl Manifest {
    pub async fn new(table_name: impl Into<PathBuf>) -> Result<Self> {
        let table_name: PathBuf = table_name.into();
        let path = PathBuf::from(&table_name).join(".meta");
        let io = IOHandler::new(&path).await?;

        debug!("Load Manifest       : {:?}", path);

        Manifest::from_io(&io).await.or_else(|_| {
            debug!("Create Manifest     : {:?}", path);
            Ok(Self {
                io,
                factory: IOHandlerFactory::new(&table_name),
                table_id: TableId::new(table_name.to_str().unwrap()),
                row_size: 0,
                tables: AvlTreeMap::new(),
                bloom_filter: BloomFilter::new_global(),
            })
        })
    }

    pub fn with_row_size(&mut self, row_size: u32) {
        if self.row_size == 0 {
            self.row_size = row_size;
        }
    }

    pub fn iter(&self) -> LsmTreeIterator {
        let mut cache = VecDeque::with_capacity(self.tables.len());

        for table in self.tables.values() {
            cache.push_back(table.clone());
        }

        LsmTreeIterator::new(cache)
    }

    pub async fn add_table(&mut self, table: SSTable) {
        let table = Arc::new(table);
        self.tables.insert(table.meta().key, table.clone());

        // TODO: compact existing tables
        //
        // 0. spawn a new task with `crate::core::runtime::spawn`
        // 1. read all table into a map (level, (key, Arc<SSTable>))
        // 2. check if there are any tables that can be compacted
        // 3. compact them with `fn compact_tables(tables: Vec<Arc<SSTable>>) -> SSTable`
        // 4. add the new table to the map with `self.add_table`

        // self.table_tracker.push_back(table);
        // crate::core::runtime::spawn(async move {
        //     self.table_tracker.compact().await;
        // });
    }

    pub fn table_files(&self) -> Vec<String> {
        self.tables
            .values()
            .map(|table| table.file_name().to_string())
            .collect()
    }
}

#[async_trait]
impl AsyncKvStoreRead for Manifest {
    async fn get(&self, key: Key) -> Result<DataStore> {
        trace!("Try to get key [{:?}] from manifest", key);

        for table in self.tables.values() {
            trace!(
                "Try to get key [{:?}] from sstable @{:?}",
                key,
                table.meta().key
            );

            if !table.meta().bloom_filter.contains(key) {
                trace!("Key not found in table {:?}: [{:?}]", table.meta().key, key);
                continue;
            }

            return match table.get(key).await? {
                DataStore::Value(block) => Ok(DataStore::Value(block)),
                DataStore::Deleted => Ok(DataStore::Deleted),
                DataStore::NotFound => continue,
            };
        }

        Ok(DataStore::NotFound)
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

#[async_trait]
impl AsyncToIO for Manifest {
    /// write the manifest's data to disk
    ///
    /// the order is `magic_number`, `table_id`,
    /// `row_size`, `global_add_bloom_filter`, `global_del_bloom_filter`, `tables`
    async fn to_io(&self, io: &IOHandler) -> Result<()> {
        let mut io = io.inner().await?;
        io.seek(SeekFrom::Start(0)).await?;

        io.write_u32(META_MAGIC_NUMBER).await?;
        io.write_u64(self.table_id.0).await?;
        io.write_u32(self.row_size).await?;

        let bloom_filter_bytes = {
            let mut writer = CompressionEncoder::with_quality(Vec::new(), Level::Default);
            writer
                .write_all(&bincode::encode_to_vec(&self.bloom_filter, BIN_CODE_CONF)?)
                .await?;
            writer.shutdown().await?;
            writer.into_inner()
        };

        io.write_u32(bloom_filter_bytes.len() as u32).await?;
        io.write_all(&bloom_filter_bytes).await?;

        for (key, table) in self.tables.iter() {
            io.write_u64(key.0).await?;
            // write the meta only
            let meta = table.meta();
            let bytes = bincode::encode_to_vec(meta, BIN_CODE_CONF)?;
            io.write_u32(bytes.len() as u32).await?;
            io.write_all(&bytes).await?;
        }

        io.flush().await?;
        Ok(())
    }
}

#[async_trait]
impl AsyncFromIO for Manifest {
    async fn from_io(io: &IOHandler) -> Result<Self> {
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

        let bloom_filter: BloomFilter = {
            let filter_size = file_io.read_u32().await?;
            let mut bytes = vec![0; filter_size as usize];
            file_io.read_exact(&mut bytes).await?;

            let mut reader = CompressionDecoder::new(bytes.as_slice());
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            bincode::decode_from_slice(&bytes, BIN_CODE_CONF)?.0
        };

        let mut tables = AvlTreeMap::new();

        while let Ok(rawkey) = file_io.read_u64().await {
            let key = SSTableKey(rawkey);

            if !key.valid() {
                return Err(DbError::InvalidSSTableKey);
            }

            let size = file_io.read_u32().await?;
            let mut bytes = vec![0; size as usize];
            file_io.read_exact(&mut bytes).await?;
            let meta: SSTableMeta = bincode::decode_from_slice(&bytes, BIN_CODE_CONF)?.0;
            let table = SSTable::new(meta, &factory, row_size).await?;
            tables.insert(key, Arc::new(table));
        }

        // let mut table_tracker = SSTableTracker::new();

        // for table in tables.values() {
        //     table_tracker.push_back(table.clone());
        // }

        Ok(Self {
            io: io.clone().await?,
            table_id,
            row_size,
            tables,
            factory,
            bloom_filter,
            // table_tracker,
        })
    }
}

impl Drop for Manifest {
    fn drop(&mut self) {
        debug!("Save Manifest       : {:?}", self.io.file_path);

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
        crate::utils::logger::init();

        let test_dir = "helper/sstable_test";

        std::fs::remove_dir_all(test_dir).ok();
        std::fs::create_dir_all(test_dir).unwrap();

        {
            let mut manifest = Manifest::new(test_dir).await?;

            let row_size = 10;
            manifest.with_row_size(row_size);

            for _ in 0..7 {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;

                let rnd = rand::random::<u32>() % 3;
                let key = SSTableKey::new(rnd);
                let meta = SSTableMeta::new(key);

                manifest.tables.insert(
                    meta.key,
                    Arc::new(SSTable::new(meta, &manifest.factory, row_size).await?),
                );
            }

            assert_eq!(manifest.tables.len(), 7);
        }

        Ok(())
    }
}
