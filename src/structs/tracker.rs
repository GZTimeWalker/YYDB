use std::{
    collections::{btree_map::Entry, BTreeMap, HashMap, VecDeque},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use tokio::sync::RwLock;

use crate::utils::*;

use super::{
    lsm::{SSTable, SSTableKey, SSTableLevel, SSTableList, SSTableMeta},
    manifest::Manifest,
    AsyncIterator, TABLE_COMPACT_THRESHOLD,
};

#[macro_export]
macro_rules! impl_deque_push {
    ($name:ident) => {
        pub fn $name(&mut self, table: Arc<SSTable>) {
            let level = table.meta().key.level();
            let tables = self.inner.entry(level).or_insert_with(VecDeque::new);
            trace!("Add table to tracker: {:?}", table.meta().key);
            tables.$name(table);
        }
    };
}

#[macro_export]
macro_rules! impl_deque_pop {
    ($name:ident) => {
        pub fn $name(&mut self, level: SSTableLevel) -> Option<Arc<SSTable>> {
            let tables = self.inner.entry(level).or_insert_with(VecDeque::new);
            trace!("Remove table from tracker: {:?}", level);
            tables.$name()
        }
    };
}

#[derive(Debug, Default)]
pub struct SSTableTracker {
    inner: HashMap<SSTableLevel, VecDeque<Arc<SSTable>>>,
}

impl SSTableTracker {
    impl_deque_push!(push_front);
    impl_deque_push!(push_back);
    impl_deque_pop!(pop_front);
    impl_deque_pop!(pop_back);

    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn get_compactable_tables(&self) -> Vec<(SSTableLevel, SSTableList)> {
        let mut ret = Vec::new();

        for (level, tables) in self.inner.iter() {
            trace!(
                "All tables for L{:?}   : {:#?}",
                level,
                tables.iter().map(|t| t.meta().key).collect::<Vec<_>>()
            );

            let mut compactable = Vec::with_capacity(tables.len());

            for table in tables.iter() {
                trace!(
                    "Table {:?} is locked? [{}]",
                    table.meta().key,
                    table.is_locked()
                );

                if table.lock() {
                    trace!("Lock table for L{:?} : {:?}", level, table.meta().key);
                    compactable.push(table.clone());
                } else {
                    trace!("Skip table for L{:?} : {:?}", level, table.meta().key);
                    for table in compactable.iter() {
                        table.unlock();
                    }
                    compactable.clear();
                    break;
                }

                if compactable.len() >= TABLE_COMPACT_THRESHOLD {
                    break;
                }
            }

            if compactable.len() < TABLE_COMPACT_THRESHOLD {
                trace!(
                    "Not enough tables for L{:?} {}/{}",
                    level,
                    compactable.len(),
                    TABLE_COMPACT_THRESHOLD
                );
                for table in compactable.iter() {
                    table.unlock();
                }
            } else {
                compactable.reverse();
                ret.push((*level, compactable));
            }
        }

        ret
    }
}

pub async fn compact_worker(
    level: SSTableLevel,
    tables: SSTableList,
    manifest: Arc<RwLock<Manifest>>,
    iter_in_progress: Arc<AtomicBool>,
) -> Result<()> {
    // TODO: use async write + merge sort instead of all in memory

    let mut data = BTreeMap::new();
    let key = SSTableKey::new(level + 1);
    let mut meta = SSTableMeta::new(key);

    for table in tables.iter() {
        trace!("Compact table: {:?} -> {:?}", table.meta().key, meta.key);
        let mut iter = table.new_iter().await?;
        iter.init_iter().await?;

        while let Some((key, value)) = iter.next().await? {
            if let Entry::Vacant(e) = data.entry(key) {
                meta.bloom_filter.insert(key);
                e.insert(value);
            }
        }
    }

    meta.set_entries_count(data.len());

    let gurad_manifest = manifest.read().await;
    let sstable = SSTable::new(meta, &gurad_manifest.factory, gurad_manifest.row_size).await?;
    sstable.archive(&data).await?;
    drop(gurad_manifest);

    let mut gurad_manifest = manifest.write().await;
    gurad_manifest.add_table(sstable).await;
    gurad_manifest.pop_tables(&tables);

    if !iter_in_progress.load(Ordering::Relaxed) {
        gurad_manifest.do_cleanup();
    }
    drop(gurad_manifest);

    Ok(())
}
