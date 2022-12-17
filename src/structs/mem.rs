use async_trait::async_trait;
use crc32fast::Hasher;
use std::collections::BTreeMap;
use std::io::SeekFrom;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use tokio::sync::RwLock;

use crate::structs::CACHE_MAGIC_NUMBER;
use crate::utils::*;

use super::manifest::Manifest;
use super::{kvstore::*, MEM_BLOCK_NUM};

pub type MemStore = BTreeMap<Key, DataStore>;

#[derive(Debug)]
pub struct MemTable {
    mut_map: Arc<RwLock<MemStore>>,
    lock_map: Arc<RwLock<MemStore>>,
    manifest: Option<Arc<RwLock<Manifest>>>,
    io: IOHandler,
}

impl MemTable {
    pub async fn new(
        table_name: impl Into<PathBuf>,
        manifest: Option<Arc<RwLock<Manifest>>>,
    ) -> Self {
        let path: PathBuf = table_name.into().join(".cache");
        let io = IOHandler::new(&path).await.unwrap();

        debug!("Load MemTable       : {:?}", path);

        if let Ok(mut mem) = MemTable::from_io(&io).await {
            mem.manifest = manifest;
            mem
        } else {
            Self {
                manifest,
                mut_map: Arc::new(RwLock::new(BTreeMap::new())),
                lock_map: Arc::new(RwLock::new(BTreeMap::new())),
                io: IOHandler::new(&path).await.unwrap(),
            }
        }
    }

    pub async fn swap(&self) {
        let mut mut_map = self.mut_map.write().await;
        let mut lock_map = self.lock_map.write().await;

        lock_map.clear();
        std::mem::swap(&mut *mut_map, &mut *lock_map);
    }

    async fn do_persist(&self) {
        if self.mut_map.read().await.len() > MEM_BLOCK_NUM {
            self.swap().await;

            let locked_map = self.lock_map.clone();
            crate::core::runtime::spawn(async move {
                Self::persist(&locked_map).await;
            });
        }
    }

    async fn persist(_locked_map: &Arc<RwLock<MemStore>>) {
        // 1. create a new L0 SSTable
        // 2. calculate the checksum of the SSTable
        // 3. write the magic number to the SSTable
        // 4. write the checksum to the SSTable
        // 5. write the compressed data to the SSTable

        // let data = locked_map.write().await.iter().collect::<Vec<_>>();

        // let bytes = bincode::encode_to_vec(&data, BIN_CODE_CONF).unwrap();
    }
}

#[async_trait]
impl AsyncKvStoreRead for MemTable {
    async fn get(&self, key: Key) -> DataStore {
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
impl AsyncKvStoreWrite for MemTable {
    async fn set(&self, key: Key, value: DataInner) {
        self.mut_map
            .write()
            .await
            .insert(key, DataStore::Value(Arc::new(value)));
        self.do_persist().await;
    }

    async fn delete(&self, key: Key) {
        self.mut_map.write().await.insert(key, DataStore::Deleted);
        self.do_persist().await;
    }
}

impl Drop for MemTable {
    fn drop(&mut self) {
        debug!("Save Memtable       : {:?}", self.io.file_path);

        futures::executor::block_on(async move {
            self.to_io(&self.io).await.unwrap();
        });
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

        let mut hasher = Hasher::new();
        hasher.update(&bytes);

        if hasher.finalize() != crc32 {
            return Err(DbError::MissChecksum);
        }

        let mut_map: MemStore = bincode::decode_from_slice(&bytes, BIN_CODE_CONF)?.0;

        Ok(Self {
            manifest: None,
            mut_map: Arc::new(RwLock::new(mut_map)),
            lock_map: Arc::new(RwLock::new(BTreeMap::new())),
            io: io.clone().await?,
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

        let bytes = bincode::encode_to_vec(cache_map, BIN_CODE_CONF)?;

        let mut hasher = Hasher::new();
        hasher.update(&bytes);
        let crc32 = hasher.finalize();

        let mut io = io.inner().await?;

        io.seek(SeekFrom::Start(0)).await?;
        io.write_u32(CACHE_MAGIC_NUMBER).await?;
        io.write_u32(crc32).await?;
        io.write(&bytes).await?;
        io.flush().await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::structs::CACHE_MAGIC_NUMBER;
    use std::path::PathBuf;
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let test_dir = "helper/memtable_test";

        std::fs::remove_dir_all(test_dir).ok();
        std::fs::create_dir_all(test_dir).unwrap();

        let path = PathBuf::from(test_dir);

        {
            let mem = MemTable::new(path.clone(), None).await;

            mem.set(1, vec![1, 2, 3]).await;
            mem.set(2, vec![4, 5, 6]).await;
            mem.set(3, vec![7, 8, 9]).await;

            assert_eq!(mem.get(1).await, DataStore::Value(Arc::new(vec![1, 2, 3])));
            assert_eq!(mem.get(2).await, DataStore::Value(Arc::new(vec![4, 5, 6])));
            assert_eq!(mem.get(3).await, DataStore::Value(Arc::new(vec![7, 8, 9])));

            mem.delete(2).await;

            assert_eq!(mem.get(2).await, DataStore::Deleted);

            mem.swap().await;

            assert_eq!(mem.get(1).await, DataStore::Value(Arc::new(vec![1, 2, 3])));
            assert_eq!(mem.get(2).await, DataStore::Deleted);
            assert_eq!(mem.get(3).await, DataStore::Value(Arc::new(vec![7, 8, 9])));
            assert_eq!(mem.get(4).await, DataStore::NotFound);

            mem.set(4, vec![10, 11, 12]).await;

            assert_eq!(
                mem.get(4).await,
                DataStore::Value(Arc::new(vec![10, 11, 12]))
            );

            mem.to_io(&mem.io).await?;
        }

        {
            let io = IOHandler::new(&path.join(".cache")).await?;
            let mut file_io = io.inner().await?;

            let magic_number = file_io.read_u32().await?;

            assert_eq!(magic_number, CACHE_MAGIC_NUMBER);

            let crc32 = file_io.read_u32().await?;

            let mut bytes = Vec::new();
            file_io.read_to_end(&mut bytes).await?;

            let mut hasher = Hasher::new();
            hasher.update(&bytes);

            assert_eq!(hasher.finalize(), crc32);

            let config = bincode::config::standard();
            let mut_map: MemStore = bincode::decode_from_slice(&bytes, config)?.0;

            assert_eq!(mut_map.len(), 4);

            assert_eq!(
                mut_map.get(&1),
                Some(&DataStore::Value(Arc::new(vec![1, 2, 3])))
            );
            assert_eq!(mut_map.get(&2), Some(&DataStore::Deleted));
        }

        {
            let mem = MemTable::from_io(&IOHandler::new(&path.join(".cache")).await?).await?;

            assert_eq!(mem.get(1).await, DataStore::Value(Arc::new(vec![1, 2, 3])));
            assert_eq!(mem.get(2).await, DataStore::Deleted);
            assert_eq!(mem.get(3).await, DataStore::Value(Arc::new(vec![7, 8, 9])));
            assert_eq!(
                mem.get(4).await,
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
                bincode::decode_from_slice::<KVStore, BincodeConfig>(slice, BIN_CODE_CONF)
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
