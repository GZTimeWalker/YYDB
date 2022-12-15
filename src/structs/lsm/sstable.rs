use std::fmt::{Debug, LowerHex};

use async_trait::async_trait;

use crate::{
    structs::{kvstore::{SizedOnDisk, AsyncKVStoreRead}, mem::DataBlock},
    utils::error::Result,
    utils::io_handler::{IOHandler, IOHandlerFactory},
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
}

#[async_trait]
impl AsyncKVStoreRead for SSTable {
    async fn get(&self, _key: u64) -> Option<DataBlock> {
        todo!()
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

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SSTableKey(pub u64);

impl SSTableKey {
    pub fn new(level: impl Into<u64>) -> Self {
        let level: u64 = level.into();
        let current = chrono::Utc::now().timestamp_millis();
        Self((level << 60) + (!(0x0F << 60) & !(current as u64)))
    }

    /// get level of this SSTable
    pub fn level(&self) -> u64 {
        self.0 >> 60
    }

    /// get timestamp of this SSTable
    pub fn timestamp(&self) -> i64 {
        !(0x0F << 60) & !(self.0) as i64
    }
}

impl LowerHex for SSTableKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

impl Debug for SSTableKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SSTableKey[{:02}]({})",
            self.0 >> 60,
            !(0x0F << 60) & !(self.0)
        )
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
        for i in 0..keys.len() {
            println!("{:?}", keys[i]);
        }
    }
}
