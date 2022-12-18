use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use tokio::sync::RwLock;

use super::{
    lsm::{SSTable, SSTableLevel},
    TABLE_COMPACT_THRESHOLD, manifest::Manifest,
};

#[macro_export]
macro_rules! impl_deque_push {
    ($name:ident) => {
        pub fn $name(&mut self, table: Arc<SSTable>) {
            let level = table.meta().key.level();
            let tables = self.inner.entry(level).or_insert_with(VecDeque::new);
            tables.$name(table);
        }
    };
}

#[macro_export]
macro_rules! impl_deque_pop {
    ($name:ident) => {
        pub fn $name(&mut self, level: SSTableLevel) -> Option<Arc<SSTable>> {
            let tables = self.inner.entry(level).or_insert_with(VecDeque::new);
            tables.$name()
        }
    };
}

#[derive(Debug)]
pub struct SSTableTracker {
    inner: HashMap<SSTableLevel, VecDeque<Arc<SSTable>>>,
    manifest: Arc<RwLock<Manifest>>,
}

impl SSTableTracker {
    pub fn new(manifest: Arc<RwLock<Manifest>>) -> Self {
        Self {
            inner: HashMap::new(),
            manifest
        }
    }

    impl_deque_push!(push_front);
    impl_deque_push!(push_back);
    impl_deque_pop!(pop_front);
    impl_deque_pop!(pop_back);

    pub async fn get_compactable_tables(&self) -> Vec<(SSTableLevel, Vec<Arc<SSTable>>)> {
        let mut ret = Vec::new();

        for (level, tables) in self.inner.iter() {
            let mut compactable = Vec::with_capacity(tables.len());

            for table in tables.iter() {
                if table.is_available().await {
                    compactable.push(table.clone());
                }

                if compactable.len() >= TABLE_COMPACT_THRESHOLD {
                    break;
                }
            }

            if compactable.len() >= TABLE_COMPACT_THRESHOLD {
                ret.push((*level, compactable));
            }
        }

        ret
    }

    pub async fn compact(&mut self) {
        for (level, tables) in self.get_compactable_tables().await {
            debug!("Compacting tables at level {} with {:#?}", level, tables.iter().map(|t| t.meta().key).collect::<Vec<_>>());
        }
    }
}
