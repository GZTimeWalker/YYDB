use async_trait::async_trait;
use crc32fast::Hasher;
use std::collections::btree_map::*;
use std::io::SeekFrom;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::time::Instant;

use tokio::sync::RwLock;

use crate::structs::CACHE_MAGIC_NUMBER;
use crate::utils::*;

use super::lsm::*;
use super::manifest::Manifest;
use super::{kvstore::*, MEM_BLOCK_NUM};

pub type MemStore = BTreeMap<Key, DataStore>;
pub type MemTableIterator = IntoIter<Key, DataStore>;

#[derive(Debug)]
pub struct MemTable {
    mut_map: Arc<RwLock<MemStore>>,
    lock_map: Arc<RwLock<MemStore>>,
    manifest: Option<Arc<RwLock<Manifest>>>,
    lock_map_released: Arc<AtomicBool>,
    io: IOHandler,
}

impl MemTable {
    pub async fn new(
        table_name: impl Into<PathBuf>,
        manifest: Option<Arc<RwLock<Manifest>>>,
    ) -> Result<Self> {
        let path: PathBuf = table_name.into().join(".cache");
        let io = IOHandler::new(&path).await?;

        debug!("Load MemTable       : {:?}", path);

        MemTable::from_io(&io)
            .await
            .or_else(|_| {
                debug!("Create MemTable     : {:?}", path);
                Ok(Self {
                    manifest: None,
                    lock_map_released: Arc::new(AtomicBool::new(true)),
                    mut_map: Arc::new(RwLock::new(BTreeMap::new())),
                    lock_map: Arc::new(RwLock::new(BTreeMap::new())),
                    io,
                })
            })
            .map(|mut mem_table| {
                mem_table.manifest = manifest;
                mem_table
            })
    }

    pub async fn swap(&self) {
        let mut mut_map = self.mut_map.write().await;
        let mut lock_map = self.lock_map.write().await;

        self.lock_map_released
            .store(false, std::sync::atomic::Ordering::Relaxed);
        std::mem::swap(&mut *mut_map, &mut *lock_map);
    }

    pub async fn do_persist(&self, new_table_added: Arc<AtomicBool>) {
        if self.mut_map.read().await.len() >= MEM_BLOCK_NUM
            && self
                .lock_map_released
                .load(std::sync::atomic::Ordering::Relaxed)
        {
            self.swap().await;

            let locked_map = self.lock_map.clone();
            let manifest = self.manifest.clone().expect("Manifest is not set");
            let lock_map_released = self.lock_map_released.clone();

            crate::core::runtime::spawn(async move {
                let start = Instant::now();
                Self::persist(locked_map, lock_map_released, manifest)
                    .await
                    .unwrap();
                new_table_added.store(true, std::sync::atomic::Ordering::Relaxed);
                trace!("Persist to disk in {:?}", start.elapsed());
            });
        }
    }

    async fn persist(
        locked_map: Arc<RwLock<MemStore>>,
        lock_map_released: Arc<AtomicBool>,
        manifest: Arc<RwLock<Manifest>>,
    ) -> Result<()> {
        // 1. calculate the checksum and bloom filter of the SSTable
        // 2. create the L0 SSTable
        // 3. write the magic number to the SSTable
        // 4. write the checksum to the SSTable
        // 5. write the compressed data to the SSTable
        // 6. write the meta data to the manifest

        let key = SSTableKey::new(0u64);
        let mut meta = SSTableMeta::new(key);

        let data: BTreeMap<Key, DataStore> = locked_map
            .read()
            .await
            .iter()
            .map(|(k, v)| {
                meta.bloom_filter.insert(k);
                (*k, v.clone())
            })
            .collect();

        meta.set_entries_count(data.len());

        let gurad_manifest = manifest.read().await;
        let sstable = SSTable::new(meta, &gurad_manifest.factory, gurad_manifest.row_size).await?;
        sstable.archive(&data).await?;
        drop(gurad_manifest);

        manifest.write().await.add_table(sstable).await;

        locked_map.write().await.clear();
        lock_map_released.store(true, std::sync::atomic::Ordering::Relaxed);

        Ok(())
    }

    pub async fn iter(&self) -> MemTableIterator {
        let mut new_map = self.lock_map.read().await.clone();

        for (k, v) in self.mut_map.read().await.iter() {
            new_map.insert(*k, v.clone());
        }

        new_map.into_iter()
    }

    pub async fn to_self_io(&self) -> Result<()> {
        self.to_io(&self.io).await
    }
}

#[async_trait]
impl AsyncKvStoreRead for MemTable {
    async fn get(&self, key: Key) -> Result<DataStore> {
        if let Some(value) = self.mut_map.read().await.get(&key) {
            trace!("Get value: [{}] -> [{}]", key, value);

            return Ok(value.clone());
        }

        if let Some(value) = self.lock_map.read().await.get(&key) {
            trace!("Get value: [{}] -> [{}]", key, value);

            return Ok(value.clone());
        }

        trace!("Key not found in MemTable: [{:?}]", key);

        Ok(DataStore::NotFound)
    }

    async fn len(&self) -> usize {
        let lock_map_len = self.lock_map.read().await.len();
        let mut_map_len = self.mut_map.read().await.len();

        lock_map_len + mut_map_len
    }
}

#[async_trait]
impl AsyncKvStoreWrite for MemTable {
    async fn set(&self, key: Key, value: DataInner) {
        self.mut_map
            .write()
            .await
            .insert(key, DataStore::Value(Arc::new(value)));

        if let Some(manifest) = &self.manifest {
            manifest.write().await.bloom_filter.insert(key);
        }
    }

    async fn delete(&self, key: Key) {
        self.mut_map.write().await.insert(key, DataStore::Deleted);
    }
}

impl Drop for MemTable {
    fn drop(&mut self) {
        debug!("Save Memtable       : {:?}", self.io.file_path);
    }
}

#[async_trait]
impl AsyncFromIO for MemTable {
    async fn from_io(io: &IOHandler) -> Result<Self> {
        if let Ok(true) = io.is_empty().await {
            return Err(DbError::EmptyFile);
        }

        let mut file_io = io.inner().await?;

        file_io.seek(SeekFrom::Start(0)).await?;

        let magic_number = file_io.read_u32().await?;

        if magic_number != CACHE_MAGIC_NUMBER {
            return Err(DbError::InvalidMagicNumber);
        }

        let crc32 = file_io.read_u32().await?;

        let mut bytes = Vec::new();
        file_io.read_to_end(&mut bytes).await?;

        if crc32 != {
            let mut hasher = Hasher::new();
            hasher.update(&bytes);
            hasher.finalize()
        } {
            return Err(DbError::MissChecksum);
        }

        let mut_map: MemStore = {
            let mut reader = CompressionDecoder::new(bytes.as_slice());
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            bincode::decode_from_slice(&bytes, BIN_CODE_CONF)?.0
        };

        Ok(Self {
            io: io.clone().await?,
            manifest: None,
            lock_map_released: Arc::new(AtomicBool::new(true)),
            mut_map: Arc::new(RwLock::new(mut_map)),
            lock_map: Arc::new(RwLock::new(BTreeMap::new())),
        })
    }
}

#[async_trait]
impl AsyncToIO for MemTable {
    async fn to_io(&self, io: &IOHandler) -> Result<()> {
        let mut_map = self.mut_map.read().await;
        let lock_map = self.lock_map.read().await;

        let mut cache_map = lock_map.clone();

        for (key, value) in mut_map.iter() {
            cache_map.insert(*key, value.clone());
        }

        let bytes = {
            let mut writer = CompressionEncoder::with_quality(Vec::new(), COMPRESSION_LEVEL);
            writer
                .write_all(&bincode::encode_to_vec(cache_map, BIN_CODE_CONF)?)
                .await?;
            writer.shutdown().await?;
            writer.into_inner()
        };

        let crc32 = {
            let mut hasher = Hasher::new();
            hasher.update(&bytes);
            hasher.finalize()
        };

        let mut io = io.inner().await?;

        io.seek(SeekFrom::Start(0)).await?;
        io.write_u32(CACHE_MAGIC_NUMBER).await?;
        io.write_u32(crc32).await?;
        io.write_all(&bytes).await?;
        io.flush().await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;
    #[tokio::test]
    async fn it_works() -> Result<()> {
        crate::utils::logger::init();

        let test_dir = "helper/memtable_test";

        std::fs::remove_dir_all(test_dir).ok();
        std::fs::create_dir_all(test_dir).unwrap();

        let path = PathBuf::from(test_dir);

        {
            let mem = MemTable::new(path.clone(), None).await?;

            mem.set(1, vec![1, 2, 3]).await;
            mem.set(2, vec![4, 5, 6]).await;
            mem.set(3, vec![7, 8, 9]).await;

            assert_eq!(mem.get(1).await?, DataStore::Value(Arc::new(vec![1, 2, 3])));
            assert_eq!(mem.get(2).await?, DataStore::Value(Arc::new(vec![4, 5, 6])));
            assert_eq!(mem.get(3).await?, DataStore::Value(Arc::new(vec![7, 8, 9])));

            mem.delete(2).await;

            assert_eq!(mem.get(2).await?, DataStore::Deleted);

            mem.swap().await;

            assert_eq!(mem.get(1).await?, DataStore::Value(Arc::new(vec![1, 2, 3])));
            assert_eq!(mem.get(2).await?, DataStore::Deleted);
            assert_eq!(mem.get(3).await?, DataStore::Value(Arc::new(vec![7, 8, 9])));
            assert_eq!(mem.get(4).await?, DataStore::NotFound);

            mem.set(4, vec![10, 11, 12]).await;

            assert_eq!(
                mem.get(4).await?,
                DataStore::Value(Arc::new(vec![10, 11, 12]))
            );

            mem.to_io(&mem.io).await?;
        }

        {
            let mem = MemTable::from_io(&IOHandler::new(&path.join(".cache")).await?).await?;

            assert_eq!(mem.get(1).await?, DataStore::Value(Arc::new(vec![1, 2, 3])));
            assert_eq!(mem.get(2).await?, DataStore::Deleted);
            assert_eq!(mem.get(3).await?, DataStore::Value(Arc::new(vec![7, 8, 9])));
            assert_eq!(
                mem.get(4).await?,
                DataStore::Value(Arc::new(vec![10, 11, 12]))
            );
        }

        Ok(())
    }

    #[test]
    fn decode_works() -> Result<()> {
        crate::utils::logger::init();

        let mut mut_map = MemStore::new();

        for i in 0..64 {
            mut_map.insert(i, DataStore::Value(Arc::new(vec![i as u8; 4])));
            if i % 7 == 3 {
                mut_map.insert(i, DataStore::Deleted);
            } else if i % 7 == 5 {
                mut_map.insert(i, DataStore::NotFound);
            }
        }

        let data = mut_map.iter().collect::<Vec<_>>();
        let bytes = bincode::encode_to_vec(&data, BIN_CODE_CONF)?;

        print_hex_view(&bytes)?;

        let (mut count, mut pos) =
            bincode::decode_from_slice::<u64, BincodeConfig>(&bytes, BIN_CODE_CONF)?;

        while count > 0 {
            let slice = &bytes[pos..];
            if let Ok((data_store, offset)) =
                bincode::decode_from_slice::<KvStore, BincodeConfig>(slice, BIN_CODE_CONF)
            {
                assert_eq!(&data_store.1, mut_map.get(&data_store.0).unwrap());
                pos += offset;
            } else {
                break;
            }
            count -= 1;
        }

        Ok(())
    }
}
