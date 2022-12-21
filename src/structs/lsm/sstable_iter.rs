use crc32fast::Hasher;
use futures::Future;
use std::{collections::VecDeque, io::SeekFrom};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt, BufReader},
};

use crate::{
    structs::{AsyncIterator, SSTABLE_MAGIC_NUMBER},
    utils::*,
};

pub const SSTABLE_ITER_BUF_SIZE: usize = 0x800;
const HEADER_SIZE: u64 = 36;

#[derive(Debug)]
pub struct SSTableIter {
    io: IOHandler,
    entries_count: u32,
    deleted_count: u32,
    entry_cur: u32,
    last_entry_key: Option<Key>,
    bytes_read: usize,
    hasher: Option<Hasher>,
    buf: VecDeque<u8>,
    raw_checksum: u32,
    compressed_checksum: u32,
    min_key: Key,
    max_key: Key,
    reader: Option<CompressionDecoder<BufReader<File>>>,
}

impl SSTableIter {
    pub async fn new(io: IOHandler, data_size: u32) -> Result<Self> {
        let mut iter = Self {
            io,
            entries_count: 0,
            deleted_count: 0,
            entry_cur: 0,
            raw_checksum: 0,
            last_entry_key: None,
            compressed_checksum: 0,
            bytes_read: 0,
            min_key: 0,
            max_key: 0,
            hasher: None,
            buf: VecDeque::with_capacity(data_size as usize * 2),
            reader: None,
        };

        iter.recreate().await?;

        Ok(iter)
    }

    pub async fn recreate(&mut self) -> Result<()> {
        let mut file_io = self.io.inner().await?;

        if file_io.metadata().await?.len() < HEADER_SIZE {
            trace!("Empty Iter          : {:?}", self.io.file_path);
            return Ok(());
        }

        file_io.seek(SeekFrom::Start(0)).await?;

        let magic_number = file_io.read_u32().await?;

        if magic_number != SSTABLE_MAGIC_NUMBER {
            return Err(DbError::InvalidMagicNumber);
        }

        self.raw_checksum = file_io.read_u32().await?;
        self.compressed_checksum = file_io.read_u32().await?;
        self.entries_count = file_io.read_u32().await?;
        self.deleted_count = file_io.read_u32().await?;

        self.min_key = file_io.read_u64().await?;
        self.max_key = file_io.read_u64().await?;

        debug!("Recreated Iter      : {:?}", self.io.file_path);
        Ok(())
    }

    #[inline]
    pub async fn init_iter(&mut self) -> Result<()> {
        self.init_iter_for_key(0).await
    }

    pub async fn init_iter_for_key(&mut self, key: Key) -> Result<()> {
        if let Some(last_key) = self.last_entry_key {
            if last_key < key {
                trace!("Iter for further key: [{}]", key);
                return Ok(());
            }
        }

        trace!("Init Iter for key   : [{}]: {:?}", key, self.io.file_path);

        self.entry_cur = 0;
        self.hasher.replace(Hasher::new());
        self.last_entry_key = None;
        self.bytes_read = 0;
        self.buf.clear();

        let mut file = File::open(self.io.file_path.as_ref()).await?;
        file.seek(SeekFrom::Start(HEADER_SIZE)).await?;
        self.reader
            .replace(CompressionDecoder::new(BufReader::new(file)));

        Ok(())
    }

    pub async fn clone_io(&self) -> Result<IOHandler> {
        self.io.clone().await
    }
}

impl AsyncIterator<KvStore> for SSTableIter {
    type NextFuture<'a> = impl Future<Output = Result<Option<KvStore>>> + 'a;

    fn next(&mut self) -> Self::NextFuture<'_> {
        async {
            if self.entry_cur >= self.entries_count {
                debug!(
                    "Decoded {} bytes ({}/{}) with checksum {:08x} from file {}",
                    self.bytes_read,
                    self.entry_cur,
                    self.entries_count,
                    self.raw_checksum,
                    self.io.file_path.display()
                );
                if let Some(hasher) = self.hasher.take() {
                    let hash = hasher.finalize();
                    if self.raw_checksum != hash {
                        error!(
                            "Checksum mismatch in file {}, expected {:08x}, got {:08x}",
                            self.io.file_path.display(),
                            self.raw_checksum,
                            hash
                        );
                    }
                } else {
                    warn!(
                        "No hasher found for file \"{}\"",
                        self.io.file_path.display()
                    );
                }
            }

            if self.buf.len() < self.buf.capacity() {
                let mut buf = vec![0u8; self.buf.capacity() - self.buf.len()];
                if let Ok(len) = self.reader.as_mut().unwrap().read(&mut buf).await {
                    if len == 0 {
                        return Ok(None);
                    }
                    self.bytes_read += len;
                    self.hasher.as_mut().unwrap().update(&buf[..len]);
                    self.buf.extend(&buf[..len]);
                }
            }

            let slice = self.buf.make_contiguous();

            let (data_store, offset) =
                bincode::decode_from_slice::<KvStore, BincodeConfig>(slice, BIN_CODE_CONF)
                    .map_err(|err| {
                        error!(
                            "Error decoding data : {:#?} in file {}, entry {}, offset {}, {}",
                            err,
                            self.io.file_path.display(),
                            self.entry_cur,
                            self.bytes_read,
                            hex_view(slice)
                                .or_else(|_| Result::Ok("< cannot format >".to_string()))
                                .unwrap()
                        );
                        err
                    })?;

            trace!(
                "Decoded data        : [{}] -> [{}], {}",
                data_store.0,
                data_store.1,
                hex_view(&slice[..offset])?
            );

            self.entry_cur += 1;
            self.buf.drain(..offset);
            self.last_entry_key.replace(data_store.0);

            Ok(Some(data_store))
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::structs::SSTABLE_MAGIC_NUMBER;
    use tokio::fs::File;

    pub async fn check_file(file_name: &str) -> Result<()> {
        debug!("Checking sstable file {}", file_name);

        let mut file = File::open(file_name).await?;

        let magic_number = file.read_u32().await?;

        if magic_number != SSTABLE_MAGIC_NUMBER {
            return Err(DbError::InvalidMagicNumber);
        }

        let raw_checksum = file.read_u32().await?;
        let compressed_checksum = file.read_u32().await?;

        let entries_count = file.read_u32().await?;
        let deleted = file.read_u32().await?;

        let min_key = file.read_u64().await?;
        let max_key = file.read_u64().await?;

        let mut bytes = Vec::new();
        let bytes_total = file.read_to_end(&mut bytes).await?;

        let mut hasher = Hasher::new();
        hasher.update(&bytes);
        let computed_compressed_checksum = hasher.finalize();

        debug!(
            "Validating checksums : {:08x} == {:08x}",
            compressed_checksum, computed_compressed_checksum
        );
        assert_eq!(compressed_checksum, computed_compressed_checksum);

        let mut raw = Vec::new();
        CompressionDecoder::new(bytes.as_slice())
            .read_to_end(&mut raw)
            .await?;

        let mut hasher = Hasher::new();
        hasher.update(&raw);
        let computed_raw_checksum = hasher.finalize();

        debug!(
            "Validating checksums : {:08x} == {:08x}",
            raw_checksum, computed_raw_checksum
        );
        assert_eq!(raw_checksum, computed_raw_checksum);

        let mut bytes_read = 0;
        for _ in 0..entries_count {
            let (_, offset) = bincode::decode_from_slice::<KvStore, BincodeConfig>(
                &raw[bytes_read..],
                BIN_CODE_CONF,
            )?;
            bytes_read += offset;
        }

        debug!("Validating decoded   : {} == {}", bytes_read, raw.len());
        assert_eq!(bytes_read, raw.len());

        debug!(
            "File \"{}\" has {} bytes, {} entries ({} deleted), key range [{}, {}]",
            file_name, bytes_total, entries_count, deleted, min_key, max_key
        );

        Ok(())
    }
}
