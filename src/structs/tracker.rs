use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use tokio::sync::RwLock;

use crate::utils::{KvStore, DataStore};

use super::{
    lsm::{SSTable, SSTableLevel, SSTableKey, SSTableMeta},
    manifest::Manifest,
    TABLE_COMPACT_THRESHOLD,
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
            manifest,
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

    pub async fn compact(&self) {
        for (level, tables) in self.get_compactable_tables().await {
            debug!(
                "Compacting tables at level {} with {:#?}",
                level,
                tables.iter().map(|t| t.meta().key).collect::<Vec<_>>()
            );
            for table in tables.iter(){
                println!("{:?}",table);
            }
        }
    }

    // pub async fn my_compact(tables: Vec<Vec<KvStore>>){
    //     let mut iter_vec = Vec::new();
    //     let mut now_vec = Vec::new();
    //     let mut i = 0;
    //     for sstable in tables.iter(){
    //         iter_vec.push(sstable.iter());
    //         if let Some(v) = iter_vec[i].next(){
    //             now_vec.push(Some((*v).0));
    //         }else{
    //             now_vec.push(None);
    //         }
    //         i = i + 1;
    //     }
    //     for sstable in tables.iter(){
    //         for record in sstable.iter(){
    //             let (key,value) = record;
    //             match value{
    //                 crate::utils::DataStore::Value(data) =>{

    //                 }
    //                 crate::utils::DataStore::Deleted =>{

    //                 }
    //                 crate::utils::DataStore::NotFound =>{
    //                     panic!("[Err] Sstable contains NotFound");
    //                 }
    //             }
    //         }
    //     }
    // }

    pub async fn my_compact2(level: u64,tables: Vec<Vec<KvStore>>) -> Vec<KvStore> {
        let mut result = Vec::new();
        let mut map = HashMap::new();
    
        for table in tables.iter() {
            for (key, value) in table.iter() {
                match value {
                    DataStore::Value(data) => {
                        map.insert(*key, (data.clone(), true));
                    }
                    DataStore::Deleted => {
                        if let Some((_, ref mut exists)) = map.get_mut(key) {
                            *exists = false;
                        }
                    }
                    DataStore::NotFound => {
                        panic!("[Err] Sstable couldn't include NotFound");
                    }
                }
            }
        }
    
        for (key, (data, exists)) in map {
            if exists {
                result.push((key, DataStore::Value(data)));
            }
        }
        result.sort_by_key(|(key, _)| *key);

        // stores in lower level's sstable
        
        let key = SSTableKey::new(level+1);
        let mut meta = SSTableMeta::new(key);

        meta.set_entries_count(result.len());
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    pub async fn it_works_async() {
        let data1 = Arc::new(vec![1, 2, 3]);
        let data2 = Arc::new(vec![4, 5, 6]);
        let data3 = Arc::new(vec![7, 8, 9]);
        let data4 = Arc::new(vec![10, 11, 12]);

        let kvstore1 = (1, DataStore::Value(data1.clone()));
        let kvstore2 = (2, DataStore::Value(data2.clone()));
        let kvstore3 = (3, DataStore::Value(data3.clone()));
        let kvstore4 = (1, DataStore::Deleted);
        let kvstore5 = (2, DataStore::Value(data4.clone()));
        let kvstore6 = (3, DataStore::Deleted);
        let kvstore7 = (4, DataStore::Value(data1.clone()));

        let table1 = vec![kvstore1.clone(), kvstore2.clone()];
        let table2 = vec![kvstore3.clone(), kvstore4.clone()];
        let table3 = vec![kvstore5.clone(), kvstore6.clone()];
        let table4 = vec![kvstore7.clone()];

        let tables = vec![table1, table2, table3, table4];

        let compacted = SSTableTracker::my_compact2(0,tables).await;

        print!("{:?}",compacted);
        assert_eq!(compacted, vec![kvstore5.clone(), kvstore7.clone()]);
    }
}





