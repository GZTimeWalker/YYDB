use std::fmt::{Debug, LowerHex};

use async_trait::async_trait;
use chrono::TimeZone;

use crate::{
    structs::{kvstore::*, mem::MemStore},
    utils::*,
};

use super::metadata::SSTableMeta;

#[derive(Debug)]
pub struct SSTable {
    meta: SSTableMeta,
    io: IOHandler,
}

impl SSTable {
    pub async fn new(meta: SSTableMeta, factory: &IOHandlerFactory) -> Self {
        let key = meta.key;
        Self {
            meta,
            io: factory.create(key).await.unwrap(),
        }
    }

    pub fn meta(&self) -> &SSTableMeta {
        &self.meta
    }

    pub async fn archive(&self, _mem: &MemStore) {
        assert_eq!(self.meta.level, 0);
    }
}

#[async_trait]
impl AsyncKVStoreRead for SSTable {
    async fn get(&self, _key: u64) -> DataStore {
        if !self.meta.bloom_filter.contains(_key) {
            return DataStore::NotFound;
        }

        DataStore::NotFound
    }

    async fn len(&self) -> usize {
        // todo!()
        0
    }
}

#[async_trait]
impl SizedOnDisk for SSTable {
    async fn size_on_disk(&self) -> Result<u64> {
        self.io.size_on_disk().await
    }
}

#[derive(Clone, Copy, Encode, Decode, PartialEq, Eq, PartialOrd, Ord)]
pub struct SSTableKey(pub u64);

impl SSTableKey {
    pub fn new(level: impl Into<u64>) -> Self {
        let level: u64 = level.into();
        let current = chrono::Utc::now().timestamp_millis();
        Self((level << 60) + (!(0x0F << 60) & !(current as u64)))
    }

    /// get level of this SSTable
    pub fn level(&self) -> u32 {
        (self.0 >> 60) as u32
    }

    /// get timestamp of this SSTable
    pub fn timestamp(&self) -> i64 {
        !(0x0F << 60) & !(self.0) as i64
    }

    pub fn valid(&self) -> bool {
        let left = chrono::Utc
            .with_ymd_and_hms(2000, 1, 1, 0, 1, 1)
            .unwrap()
            .timestamp_millis();
        let right = chrono::Utc
            .with_ymd_and_hms(3000, 1, 1, 0, 1, 1)
            .unwrap()
            .timestamp_millis();
        let key_time = self.timestamp();

        self.level() < 15 && key_time > left && key_time < right
    }
}

impl LowerHex for SSTableKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

impl Debug for SSTableKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SSTableKey[{:02}]({})", self.level(), self.timestamp())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut keys = vec![];
        for i in 0..3u64 {
            keys.push(SSTableKey::new(i));
            std::thread::sleep(std::time::Duration::from_millis(1));
            keys.push(SSTableKey::new(i));
        }
        keys.sort();

        for i in 0..3usize {
            assert_eq!(keys[i * 2].level(), i as u32);
            assert_eq!(keys[i * 2 + 1].level(), i as u32);
        }
    }
}
