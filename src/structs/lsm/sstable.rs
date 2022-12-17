use std::{
    fmt::{Debug, LowerHex},
    path::PathBuf,
    sync::Arc,
};

use async_compression::Level;
use async_trait::async_trait;
use chrono::TimeZone;
use tokio::{fs, io::AsyncWriteExt, sync::Mutex};

use crate::{structs::*, utils::*};

use super::*;

#[derive(Debug)]
pub struct SSTable {
    meta: SSTableMeta,
    iter: Mutex<SSTableIter>,
    file_name: Arc<PathBuf>,
}

impl SSTable {
    pub async fn new(meta: SSTableMeta, factory: &IOHandlerFactory, row_size: u32) -> Result<Self> {
        let key = meta.key;
        let io = factory.create(key).await?;
        Ok(Self {
            meta,
            file_name: io.file_path.clone(),
            iter: Mutex::new(SSTableIter::new(io, row_size).await?),
        })
    }

    pub fn meta(&self) -> &SSTableMeta {
        &self.meta
    }

    /// archive data to disk
    ///
    /// # Note
    /// should only be called in L0 level
    pub async fn archive(&self, data: Vec<KvStore>) -> Result<()> {
        let bytes = bincode::encode_to_vec(data, BIN_CODE_CONF)?;
        let (_, len_offset): (u64, usize) = bincode::decode_from_slice(&bytes, BIN_CODE_CONF)?;

        let bytes = &bytes[len_offset..];

        let mut writer = CompressionEncoder::with_quality(Vec::new(), Level::Default);
        writer.write_all(bytes).await?;
        writer.shutdown().await?;

        let bytes = writer.into_inner();

        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&bytes);
        let checksum = hasher.finalize();

        let io = self.iter.lock().await.clone_io().await?;
        let mut file_io = io.inner().await?;

        file_io.write_u32(SSTABLE_MAGIC_NUMBER).await?;
        file_io.write_u32(checksum).await?;
        file_io.write_all(&bytes).await?;

        drop(file_io);

        self.iter.lock().await.recreate().await?;

        Ok(())
    }
}

#[async_trait]
impl AsyncKvStoreRead for SSTable {
    async fn get(&self, key: Key) -> Result<DataStore> {
        let mut iter = self.iter.lock().await;
        iter.init_iter().await.unwrap();

        while let Some(kvstore) = iter.next().await? {
            if kvstore.0 == key {
                return Ok(kvstore.1);
            }
        }

        Ok(DataStore::NotFound)
    }

    async fn len(&self) -> usize {
        todo!();
    }
}

#[async_trait]
impl SizedOnDisk for SSTable {
    async fn size_on_disk(&self) -> Result<u64> {
        Ok(fs::metadata(self.file_name.as_ref()).await?.len())
    }
}

#[derive(Clone, Copy, Encode, Decode, PartialEq, Eq, PartialOrd, Ord)]
pub struct SSTableKey(pub u64);

impl SSTableKey {
    pub fn new(level: impl Into<u64>) -> Self {
        let level: u64 = level.into();
        let current = chrono::Utc::now().timestamp_micros();
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
            .timestamp_micros();
        let right = chrono::Utc
            .with_ymd_and_hms(3000, 1, 1, 0, 1, 1)
            .unwrap()
            .timestamp_micros();
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
        for i in 0..6u64 {
            keys.push(SSTableKey::new(i));
            std::thread::sleep(std::time::Duration::from_millis(5));
            keys.push(SSTableKey::new(i));
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        keys.sort();

        for i in 0..3usize {
            assert_eq!(keys[i * 2].level(), i as u32);
            assert_eq!(keys[i * 2 + 1].level(), i as u32);
        }
    }
}
