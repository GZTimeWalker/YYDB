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
const HEADER_SIZE: u64 = 16;

#[derive(Debug)]
pub struct SSTableIter {
    io: IOHandler,
    entries_count: u32,
    entrie_cur: u32,
    bytes_read: usize,
    hasher: Option<Hasher>,
    buf: VecDeque<u8>,
    raw_checksum: u32,
    compressed_checksum: u32,
    reader: Option<CompressionDecoder<BufReader<File>>>,
}

impl SSTableIter {
    pub async fn new(io: IOHandler, data_size: u32) -> Result<Self> {
        let mut iter = Self {
            io,
            entries_count: 0,
            entrie_cur: 0,
            raw_checksum: 0,
            compressed_checksum: 0,
            bytes_read: 0,
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

        debug!("Recreated Iter      : {:?}", self.io.file_path);
        Ok(())
    }

    pub async fn init_iter(&mut self) -> Result<()> {
        if let Some(reader) = self.reader.as_mut() {
            reader.get_mut().get_mut().seek(SeekFrom::Start(HEADER_SIZE)).await?;
            return Ok(());
        }

        let mut file = File::open(self.io.file_path.as_ref()).await?;
        file.seek(SeekFrom::Start(HEADER_SIZE)).await?;
        self.reader
            .replace(CompressionDecoder::new(BufReader::new(file)));
        self.entrie_cur = 0;

        self.hasher.replace(Hasher::new());

        Ok(())
    }

    pub async fn clone_io(&self) -> Result<IOHandler> {
        self.io.clone().await
    }
}

impl AsyncIterator<KvStore> for SSTableIter {
    type NextFuture<'a> = impl Future<Output = Result<Option<KvStore>>> + 'a;

    fn next(&mut self) -> Self::NextFuture<'_> {
        async move {
            if self.entrie_cur >= self.entries_count {
                debug!(
                    "Decoded {} bytes with checksum {:08x}",
                    self.bytes_read,
                    self.raw_checksum
                );
                let hash = self.hasher.take().unwrap().finalize();
                if self.raw_checksum != hash {
                    error!(
                        "Checksum mismatch in file {}, expected {:08x}, got {:08x}",
                        self.io.file_path.display(),
                        self.raw_checksum,
                        hash
                    );
                    panic!();
                }
            }

            let reader = self.reader.as_mut().unwrap();

            if self.buf.len() < self.buf.capacity() {
                let mut buf = vec![0u8; self.buf.capacity() - self.buf.len()];
                if let Ok(len) = reader.read(&mut buf).await {
                    if len == 0 {
                        return Ok(None);
                    }
                    self.bytes_read += len;
                    self.hasher.as_mut().unwrap().update(&buf[..len]);
                    self.buf.extend(&buf[..len]);
                }
            }

            let slice = self.buf.make_contiguous();

            let (data_store, offset) = bincode::decode_from_slice::<KvStore, BincodeConfig>(slice, BIN_CODE_CONF).map_err(|err| {
                error!(
                    "Error decoding data : {:#?} in file {}, {}",
                    err,
                    self.io.file_path.display(),
                    hex_view(&slice).unwrap()
                );
                err
            })?;

            debug!(
                "Decoded data        : [{}] -> [{}], {}",
                data_store.0,
                data_store.1,
                hex_view(&slice[..offset])?
            );

            trace!("Remaining data      : {}", hex_view(&slice[offset..])?);


            #[cfg(test)]
            if let DataStore::Value(value) = &data_store.1 {
                trace!("SSTableIter: {}", hex_view(&value.as_ref())?);
            }

            self.entrie_cur += 1;
            self.buf.drain(..offset);

            Ok(Some(data_store))
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use tokio::fs::File;
    use crate::structs::SSTABLE_MAGIC_NUMBER;

    pub async fn check_file(file_name: &str) -> Result<()> {
        debug!("Checking sstable file {}", file_name);

        let mut file = File::open(file_name).await?;

        let magic_number = file.read_u32().await?;

        if magic_number != SSTABLE_MAGIC_NUMBER {
            return Err(DbError::InvalidMagicNumber);
        }

        let raw_checksum = file.read_u32().await?;
        let compressed_checksum = file.read_u32().await?;

        let len = file.read_u32().await?;

        let mut bytes = Vec::new();
        let bytes_total = file.read_to_end(&mut bytes).await?;

        let mut hasher = Hasher::new();
        hasher.update(&bytes);
        let computed_compressed_checksum = hasher.finalize();

        debug!("Validating checksums : {:08x} == {:08x}", compressed_checksum, computed_compressed_checksum);
        assert_eq!(compressed_checksum, computed_compressed_checksum);

        let mut raw = Vec::new();
        CompressionDecoder::new(bytes.as_slice()).read_to_end(&mut raw).await?;

        let mut hasher = Hasher::new();
        hasher.update(&raw);
        let computed_raw_checksum = hasher.finalize();

        debug!("Validating checksums : {:08x} == {:08x}", raw_checksum, computed_raw_checksum);
        assert_eq!(raw_checksum, computed_raw_checksum);

        debug!(
            "File \"{}\" has {} bytes, {} entries, no problems found",
            file_name,
            bytes_total,
            len
        );

        Ok(())
    }
}
