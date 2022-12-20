use async_trait::async_trait;
use tokio::sync::RwLock;

use super::lsm::LsmTreeIterator;
use super::manifest::Manifest;
use super::mem::MemTable;
use super::{kvstore::*, MemTableIterator};
use crate::utils::*;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::fmt::{Formatter, LowerHex};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};
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

    // new table added
    new_table_added: Arc<AtomicBool>,

    // iterators
    memtable_iter: RwLock<Option<MemTableIterator>>,
    lsm_iter: RwLock<Option<LsmTreeIterator>>,
    deleted: RwLock<Option<HashSet<Key>>>,
}

impl Table {
    pub async fn open(table_name: String) -> Result<Table> {
        let table_id = TableId::new(&table_name);
        debug!("Open table          : \"{}\"@{:x}", table_name, table_id);

        let table_name = &table_name;
        std::fs::create_dir_all(table_name)?;
        let manifest = Arc::new(RwLock::new(Manifest::new(table_name).await?));

        Ok(Table {
            id: table_id,
            name: table_name.to_string(),
            manifest: manifest.clone(),
            memtable: MemTable::new(table_name, Some(manifest)).await?,
            new_table_added: Arc::new(AtomicBool::new(false)),
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

    #[inline]
    async fn deleted_insert(&self, key: Key) {
        self.deleted.write().await.as_mut().unwrap().insert(key);
    }

    #[inline]
    async fn deleted_contains(&self, key: &Key) -> bool {
        self.deleted.read().await.as_ref().unwrap().contains(key)
    }

    #[inline]
    pub async fn table_files(&self) -> Vec<String> {
        self.manifest.read().await.table_files()
    }

    async fn compact(&self) {
        let compactable_tables = self.manifest.read().await.get_compactable_tables();

        for (level, tables) in compactable_tables {
            trace!(
                "Compacting tables at level {} with {:#?}",
                level,
                tables.iter().map(|t| t.meta().key).collect::<Vec<_>>()
            );

            let manifest = self.manifest.clone();

            crate::core::runtime::spawn(async move {
                super::tracker::compact_worker(level, tables, manifest).await
            });
        }
    }

    pub async fn next(&self) -> Result<Option<KvStore>> {
        if let Some(memtable_iter) = self.memtable_iter.write().await.as_mut() {
            for kvstore in memtable_iter.by_ref() {
                match kvstore {
                    (key, DataStore::Value(value)) => match self.deleted_contains(&key).await {
                        true => continue,
                        false => return Ok(Some((key, DataStore::Value(value)))),
                    },
                    (key, DataStore::Deleted) => {
                        self.deleted_insert(key).await;
                        continue;
                    }
                    _ => unreachable!(),
                }
            }
        }

        if let Some(lsm_iter) = self.lsm_iter.write().await.as_mut() {
            while let Some(kvstore) = lsm_iter.next().await? {
                match kvstore {
                    (key, DataStore::Value(value)) => match self.deleted_contains(&key).await {
                        true => continue,
                        false => return Ok(Some((key, DataStore::Value(value)))),
                    },
                    (key, DataStore::Deleted) => {
                        self.deleted_insert(key).await;
                        continue;
                    }
                    _ => unreachable!(),
                }
            }
        }

        Ok(None)
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
        self.memtable.set(key, value).await;
        self.memtable.do_persist(self.new_table_added.clone()).await;

        if self.new_table_added.load(Ordering::Relaxed) {
            trace!("New table added, compacting...");
            self.new_table_added.store(false, Ordering::Relaxed);
            self.compact().await;
        }
    }

    async fn delete(&self, key: Key) {
        self.memtable.delete(key).await;
    }
}

#[async_trait]
impl SizedOnDisk for Table {
    async fn size_on_disk(&self) -> Result<u64> {
        self.manifest.read().await.size_on_disk().await
    }
}
