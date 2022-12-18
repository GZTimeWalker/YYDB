use async_trait::async_trait;
use tokio::sync::RwLock;

use super::lsm::LsmTreeIterator;
use super::manifest::Manifest;
use super::mem::MemTable;
use super::{kvstore::*, MemTableIterator};
use crate::utils::*;
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Formatter, LowerHex};
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

impl LowerHex for TableId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

#[derive(Debug)]
pub struct Table {
    id: TableId,
    name: String,
    memtable: MemTable,
    memtable_iter: RwLock<Option<MemTableIterator>>,
    manifest: Arc<RwLock<Manifest>>,
    lsm_iter: RwLock<Option<LsmTreeIterator>>,
}

impl Table {
    pub async fn open(table_name: String) -> Result<Table> {
        let table_id = TableId::new(&table_name);
        info!("Open table          : \"{}\"@{:x}", table_name, table_id);

        let table_name = &table_name;
        std::fs::create_dir_all(table_name)?;

        let manifest = Arc::new(RwLock::new(Manifest::new(table_name).await?));

        Ok(Table {
            id: table_id,
            name: table_name.to_string(),
            memtable: MemTable::new(table_name, Some(manifest.clone())).await?,
            memtable_iter: RwLock::new(None),
            manifest,
            lsm_iter: RwLock::new(None),
        })
    }

    pub fn id(&self) -> TableId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn init_iter(&self) {
        self.memtable_iter
            .write()
            .await
            .replace(self.memtable.iter().await);
        self.lsm_iter
            .write()
            .await
            .replace(self.manifest.read().await.iter());
    }

    pub async fn end_iter(&self) {
        self.memtable_iter.write().await.take();
        self.lsm_iter.write().await.take();
    }

    pub async fn next(&self) -> Option<KvStore> {
        if let Some(iter) = self.memtable_iter.write().await.as_mut() {
            if let Some(kv) = iter.next() {
                return Some(kv);
            };
        };

        todo!();
    }

    pub async fn table_files(&self) -> Vec<String> {
        self.manifest.read().await.table_files()
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        info!("Close table         : {:?}@{:x}", self.name, self.id);
    }
}

#[async_trait]
impl AsyncKvStoreRead for Table {
    async fn get(&self, key: Key) -> Result<DataStore> {
        let manifest = self.manifest.read().await;

        if !manifest.bloom_filter.contains(key) {
            return Ok(DataStore::NotFound);
        }

        trace!("Try Get key in table {:x}: [{:?}]", self.id, key);

        drop(manifest); // release lock

        match self.memtable.get(key).await? {
            DataStore::Value(value) => return Ok(DataStore::Value(value)),
            DataStore::Deleted => return Ok(DataStore::Deleted),
            DataStore::NotFound => (),
        };

        let manifest = self.manifest.read().await;

        let ret = manifest.get(key).await?;
        trace!("Get value: [{}] -> [{}]", key, ret);

        Ok(ret)
    }

    async fn len(&self) -> usize {
        self.memtable.len().await + self.manifest.read().await.len().await
    }
}

#[async_trait]
impl AsyncKvStoreWrite for Table {
    async fn set(&self, key: Key, value: DataInner) {
        self.manifest
            .write()
            .await
            .with_row_size(value.len() as u32);
        self.memtable.set(key, value).await
    }

    async fn delete(&self, key: Key) {
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
    use rand::{RngCore, SeedableRng};

    use super::*;
    use crate::{structs::lsm::tests::check_file, utils::error::Result};

    #[test]
    fn it_works() -> Result<()> {
        crate::core::runtime::block_on(it_works_async()) // ensure use one async runtime
    }

    async fn it_works_async() -> Result<()> {
        crate::utils::logger::init();
        let test_dir = "helper/table_test";

        std::fs::remove_dir_all(test_dir).ok();
        std::fs::create_dir_all(test_dir).unwrap();

        let table = Table::open(test_dir.to_string()).await?;

        assert_eq!(table.name(), test_dir);
        assert_eq!(table.id(), TableId::new(test_dir));

        const TEST_SIZE: u64 = 600;
        const DATA_SIZE: usize = 223;
        const RANDOM_TESTS: usize = 100;

        debug!("{:=^80}", " Init Test Set ");

        for i in 0..TEST_SIZE {
            // random with seed i
            let mut data = vec![(i % 57 + 65) as u8; 64];

            let mut rng = rand::rngs::StdRng::seed_from_u64(i);
            let mut rnd_data = vec![0; DATA_SIZE];
            rng.fill_bytes(&mut rnd_data);

            data.extend_from_slice(&rnd_data);
            table.set(i, data).await;
        }

        for i in (0..TEST_SIZE).step_by(5) {
            table.delete(i).await;
        }

        tokio::time::sleep(std::time::Duration::from_millis(400)).await;

        // check files
        debug!("{:=^80}", " Check Files ");

        for table in table.manifest.read().await.table_files() {
            check_file(&table).await?;
        }

        debug!("{:=^80}", " Sequential Read Test ");

        for i in (5..TEST_SIZE).step_by(13) {
            // random with seed i
            if let DataStore::Value(v) = table.get(i).await? {
                let mut data = vec![(i % 57 + 65) as u8; 64];

                let mut rng = rand::rngs::StdRng::seed_from_u64(i);
                let mut rnd_data = vec![0; DATA_SIZE];
                rng.fill_bytes(&mut rnd_data);

                data.extend_from_slice(&rnd_data);

                assert_eq!(v.as_ref(), &data);
            } else {
                assert_eq!(i % 5, 0);
            }
        }

        // test for random key reading
        debug!("{:=^80}", " Random Read Test ");

        for _ in 0..RANDOM_TESTS {
            let mut rng = rand::rngs::StdRng::seed_from_u64(rand::random());
            let key = rng.next_u64() % TEST_SIZE;

            if key % 5 == 0 {
                assert_eq!(table.get(key).await?, DataStore::Deleted);
            } else {
                if let DataStore::Value(v) = table.get(key).await? {
                    let mut data = vec![(key % 57 + 65) as u8; 64];

                    let mut rng = rand::rngs::StdRng::seed_from_u64(key);
                    let mut rnd_data = vec![0; DATA_SIZE];
                    rng.fill_bytes(&mut rnd_data);

                    data.extend_from_slice(&rnd_data);

                    assert_eq!(v.as_ref(), &data);
                } else {
                    warn!("Value Not found: {}", key);
                }
            }
        }

        debug!("{:=^80}", " Delete Test ");

        table.delete(43).await;
        assert_eq!(table.get(43).await?, DataStore::Deleted);
        assert_eq!(table.get(TEST_SIZE + 20).await?, DataStore::NotFound);

        debug!("{:=^80}", " All Test Passed ");
        Ok(())
    }
}
