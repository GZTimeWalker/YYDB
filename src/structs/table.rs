use async_trait::async_trait;
use tokio::sync::RwLock;

use super::lsm::LsmTreeIterator;
use super::manifest::Manifest;
use super::mem::MemTable;
use super::{kvstore::*, MemTableIterator};
use crate::utils::*;
use std::collections::HashSet;
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
    manifest: Arc<RwLock<Manifest>>,

    // iterators
    memtable_iter: RwLock<Option<MemTableIterator>>,
    lsm_iter: RwLock<Option<LsmTreeIterator>>,
    deleted: RwLock<Option<HashSet<Key>>>
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
            manifest,
            memtable_iter: RwLock::new(None),
            lsm_iter: RwLock::new(None),
            deleted: RwLock::new(None),
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
        self.deleted.write().await.replace(HashSet::new());
    }

    pub async fn end_iter(&self) {
        self.memtable_iter.write().await.take();
        self.lsm_iter.write().await.take();
        self.deleted.write().await.take();
    }

    pub async fn next(&self) -> Result<Option<KvStore>> {
        if let Some(memtable_iter) = self.memtable_iter.write().await.as_mut() {
            while let Some(kvstore) = memtable_iter.next() {
                if let DataStore::Deleted = kvstore.1 {
                    self.deleted.write().await.as_mut().unwrap().insert(kvstore.0);
                    continue;
                } else if self.deleted.read().await.as_ref().unwrap().contains(&kvstore.0) {
                    continue;
                }
                
                return Ok(Some(kvstore));
            }
        }

        if let Some(lsm_iter) = self.lsm_iter.write().await.as_mut() {
            while let Some(kvstore) = lsm_iter.next().await? {
                if let DataStore::Deleted = kvstore.1 {
                    self.deleted.write().await.as_mut().unwrap().insert(kvstore.0);
                    continue;
                } else if self.deleted.read().await.as_ref().unwrap().contains(&kvstore.0) {
                    continue;
                }

                return Ok(Some(kvstore));
            }
        }

        Ok(None)
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

        let start = std::time::Instant::now();
        let table = Table::open(test_dir.to_string()).await?;

        assert_eq!(table.name(), test_dir);
        assert_eq!(table.id(), TableId::new(test_dir));

        const TEST_SIZE: u64 = 666;
        const DATA_SIZE: usize = 666;
        const NUMBER_TESTS: usize = 600;
        const RANDOM_TESTS: usize = TEST_SIZE as usize / 2;

        debug!("{:=^80}", " Init Test Set ");

        for i in 0..TEST_SIZE {
            // random with seed i
            let mut data = vec![(i % 57 + 65) as u8; NUMBER_TESTS];

            let mut rng = rand::rngs::StdRng::seed_from_u64(i);
            let mut rnd_data = vec![0; DATA_SIZE - NUMBER_TESTS];
            rng.fill_bytes(&mut rnd_data);

            data.extend_from_slice(&rnd_data);
            table.set(i, data).await;
        }

        for i in (0..TEST_SIZE).step_by(5) {
            table.delete(i).await;
        }

        debug!("{:=^80}", format!(" Init Test Set Done ({:?}) ", start.elapsed()));

        debug!(">>> Waiting for flush...");
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        // check files
        debug!("{:=^80}", " Check Files ");
        let test_start = std::time::Instant::now();

        for table in table.manifest.read().await.table_files() {
            check_file(&table).await?;
        }

        debug!("{:=^80}", format!(" Check Files Done ({:?}) ", test_start.elapsed()));

        debug!("{:=^80}", " Sequential Read Test ");
        let start = std::time::Instant::now();

        for i in (5..TEST_SIZE).step_by(13) {
            // random with seed i
            match table.get(i).await? {
                DataStore::Value(v) => {
                    let mut data = vec![(i % 57 + 65) as u8; NUMBER_TESTS];

                    let mut rng = rand::rngs::StdRng::seed_from_u64(i);
                    let mut rnd_data = vec![0; DATA_SIZE - NUMBER_TESTS];
                    rng.fill_bytes(&mut rnd_data);

                    data.extend_from_slice(&rnd_data);

                    assert_eq!(v.as_ref(), &data);
                }
                _ => {
                    if i % 5 == 0 {
                        continue;
                    } else {
                        panic!("Unexpected value for key {}", i);
                    }
                }
            }
        }

        debug!("{:=^80}", format!(" Sequential Read Test Done ({:?}) ", start.elapsed()));

        // test for random key reading
        debug!("{:=^80}", " Random Read Test ");
        let start = std::time::Instant::now();

        for _ in 0..RANDOM_TESTS {
            let mut rng = rand::rngs::StdRng::seed_from_u64(rand::random());
            let key = rng.next_u64() % TEST_SIZE;

            if key % 5 == 0 {
                assert_eq!(table.get(key).await?, DataStore::Deleted);
            } else {
                if let DataStore::Value(v) = table.get(key).await? {
                    let mut data = vec![(key % 57 + 65) as u8; NUMBER_TESTS];

                    let mut rng = rand::rngs::StdRng::seed_from_u64(key);
                    let mut rnd_data = vec![0; DATA_SIZE - NUMBER_TESTS];
                    rng.fill_bytes(&mut rnd_data);

                    data.extend_from_slice(&rnd_data);

                    assert_eq!(v.as_ref(), &data);
                } else {
                    warn!("Value Not found: {}", key);
                }
            }
        }

        debug!("{:=^80}", format!(" Random Read Test Done ({:?}) ", start.elapsed()));

        debug!("{:=^80}", " NotFound Test ");

        assert_eq!(table.get(TEST_SIZE + 20).await?, DataStore::NotFound);

        debug!("{:=^80}", " Iter Test ");
        let start = std::time::Instant::now();

        table.init_iter().await;

        let mut count = 0;
        while let Some(next) = table.next().await? {
            trace!("Got next item: [{}] -> [{}]", next.0, next.1);
            count += 1;
        }

        table.end_iter().await;

        debug!("{:=^80}", format!(" Got {} Items ({:?}) ", count, start.elapsed()));

        let size_on_disk = table.size_on_disk().await?;

        debug!("Size on disk: {}", human_read_size(size_on_disk));

        debug!("{:=^80}", format!(" All Test Passed ({:?}) ", test_start.elapsed()));
        Ok(())
    }
}
