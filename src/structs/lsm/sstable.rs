use std::{
    fmt::{Debug, LowerHex},
    path::PathBuf,
    sync::Arc,
};

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

    pub fn file_name(&self) -> &str {
        self.file_name.to_str().unwrap()
    }

    /// archive data to disk
    ///
    /// # Note
    /// should only be called in L0 level
    pub async fn archive(&self, data: &mut Vec<KvStore>) -> Result<()> {
        data.sort_by(|a, b| a.0.cmp(&b.0));

        let min_key = data.first().unwrap().0;
        let max_key = data.last().unwrap().0;

        let entries_count = data.len() as u32;
        let mut raw_hasher = crc32fast::Hasher::new();

        let mut bytes_read = 0;
        let bytes = {
            let mut writer = CompressionEncoder::new(Vec::new());
            for kvstore in data.iter() {
                let row = bincode::encode_to_vec(kvstore, BIN_CODE_CONF).unwrap();
                bytes_read += row.len();
                raw_hasher.update(&row);
                writer.write_all(&row).await?;
            }
            writer.shutdown().await?;
            writer.into_inner()
        };

        let raw_checksum = raw_hasher.finalize();
        let mut compressed_hasher = crc32fast::Hasher::new();
        compressed_hasher.update(&bytes);
        let compressed_checksum = compressed_hasher.finalize();

        debug!(
            "Encoded ({}/{}) bytes, {} entries, with checksum ({:08x}/{:08x}), key range: [{}, {}]",
            bytes_read,
            bytes.len(),
            entries_count,
            raw_checksum,
            compressed_checksum,
            min_key,
            max_key
        );

        let io = self.iter.lock().await.clone_io().await?;
        let mut file_io = io.inner().await?;

        file_io.write_u32(SSTABLE_MAGIC_NUMBER).await?;
        file_io.write_u32(raw_checksum).await?;
        file_io.write_u32(compressed_checksum).await?;
        file_io.write_u32(entries_count).await?;
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
        iter.init_iter_for_key(key).await?;

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
