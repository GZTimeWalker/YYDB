use std::{
    collections::BTreeMap,
    fmt::{Debug, LowerHex},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use async_trait::async_trait;
use chrono::TimeZone;
use tokio::{
    fs,
    io::AsyncWriteExt,
    sync::{Mutex, MutexGuard},
};

use crate::{structs::*, utils::*};

use super::*;

#[derive(Debug)]
pub struct SSTable {
    meta: SSTableMeta,
    iter: Mutex<SSTableIter>,
    locked: AtomicBool,
    file_name: Arc<PathBuf>,
    data_size: u32,
}

impl SSTable {
    pub async fn new(
        meta: SSTableMeta,
        factory: &IOHandlerFactory,
        data_size: u32,
    ) -> Result<Self> {
        let key = meta.key;
        let io = factory.create(key).await?;
        Ok(Self {
            meta,
            data_size,
            locked: AtomicBool::new(false),
            file_name: io.file_path.clone(),
            iter: Mutex::new(SSTableIter::new(io, data_size).await?),
        })
    }

    #[inline]
    pub fn meta(&self) -> &SSTableMeta {
        &self.meta
    }

    #[inline]
    pub fn file_name(&self) -> &str {
        self.file_name.to_str().unwrap()
    }

    #[inline]
    pub async fn iter(&self) -> MutexGuard<SSTableIter> {
        self.iter.lock().await
    }

    #[inline]
    pub async fn init_iter(&self) -> Result<()> {
        self.iter.lock().await.init_iter_for_key(0).await
    }

    #[inline]
    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn lock(&self) -> bool {
        self.locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    #[inline]
    pub fn unlock(&self) {
        self.locked.store(false, Ordering::Relaxed);
    }

    #[inline]
    pub async fn new_iter(&self) -> Result<SSTableIter> {
        trace!("New iter for sstable: {:?}", self.file_name);
        let io = self.iter.lock().await.clone_io().await?;
        SSTableIter::new(io, self.data_size).await
    }

    /// archive data to disk
    ///
    /// # Arguments
    /// * `data` - data to be archived, must be sorted by key
    pub async fn archive(&self, data: &BTreeMap<Key, DataStore>) -> Result<()> {
        let min_key = *data.iter().next().unwrap().0;
        let max_key = *data.iter().next_back().unwrap().0;

        let entries_count = data.len() as u32;
        let mut deleted_count = 0;
        let mut raw_hasher = crc32fast::Hasher::new();

        let mut bytes_read = 0;
        let bytes = {
            let mut writer = CompressionEncoder::with_quality(Vec::new(), COMPRESSION_LEVEL);
            for kvstore in data.iter() {
                if kvstore.1.is_deleted() {
                    deleted_count += 1;
                }
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

        if self.meta.key.level() > 0 {
            debug!(
                "Encoded ({}/{}) bytes with checksum ({:08x}/{:08x}), key range: [{}, {}]",
                bytes_read,
                bytes.len(),
                raw_checksum,
                compressed_checksum,
                min_key,
                max_key
            );
        }

        let io = self.iter.lock().await.clone_io().await?;
        let mut file_io = io.inner().await?;

        file_io.write_u32(SSTABLE_MAGIC_NUMBER).await?;
        file_io.write_u32(raw_checksum).await?;
        file_io.write_u32(compressed_checksum).await?;
        file_io.write_u32(entries_count).await?;
        file_io.write_u32(deleted_count).await?;
        file_io.write_u64(min_key).await?;
        file_io.write_u64(max_key).await?;
        file_io.write_all(&bytes).await?;

        drop(file_io); // release lock

        if self.meta.key.level() > 0 {
            debug!(
                "Archived {} entries ({} deleted) to {}",
                entries_count,
                deleted_count,
                self.file_name.to_str().unwrap()
            );
        }

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
        self.meta.entries_count
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
    pub fn level(&self) -> SSTableLevel {
        (self.0 >> 60) as SSTableLevel
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
