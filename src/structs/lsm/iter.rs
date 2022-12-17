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

#[derive(Debug)]
pub struct SSTableIter {
    io: IOHandler,
    buf: VecDeque<u8>,
    checksum: Option<u32>,
    reader: Option<CompressionDecoder<BufReader<File>>>,
}

impl SSTableIter {
    pub async fn new(io: IOHandler, data_size: u32) -> Result<Self> {
        let mut iter = Self {
            io,
            buf: VecDeque::with_capacity(data_size as usize + 32),
            checksum: None,
            reader: None,
        };

        iter.recreate().await?;

        Ok(iter)
    }

    pub async fn recreate(&mut self) -> Result<()> {
        let mut file_io = self.io.inner().await?;

        if file_io.metadata().await?.len() == 0 {
            return Ok(());
        }

        file_io.seek(SeekFrom::Start(0)).await?;

        let magic_number = file_io.read_u32().await?;

        if magic_number != SSTABLE_MAGIC_NUMBER {
            return Err(DbError::InvalidMagicNumber);
        }

        self.checksum = Some(file_io.read_u32().await?);

        let mut len_buffer = [0u8; 8];
        file_io.read_exact(&mut len_buffer).await?;

        debug!("Recreated Iter      : {:?}", self.io.file_path);
        Ok(())
    }

    pub async fn init_iter(&mut self) -> Result<()> {
        if let Some(reader) = self.reader.as_mut() {
            reader.get_mut().get_mut().seek(SeekFrom::Start(8)).await?;
            return Ok(());
        }

        let mut file = File::open(self.io.file_path.as_ref()).await?;
        file.seek(SeekFrom::Start(8)).await?;
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
        async move {
            let reader = self.reader.as_mut().unwrap();

            if self.buf.len() < self.buf.capacity() {
                let mut buf = vec![0u8; self.buf.capacity() - self.buf.len()];
                if let Ok(len) = reader.read_exact(&mut buf).await {
                    if len == 0 {
                        return Ok(None);
                    }

                    self.buf.extend(buf);
                }
            }

            let (data_store, offset) = bincode::decode_from_slice::<KvStore, BincodeConfig>(
                self.buf.make_contiguous(),
                BIN_CODE_CONF,
            )?;
            self.buf.drain(..offset);

            #[cfg(test)]
            if let DataStore::Value(value) = &data_store.1 {
                trace!("SSTableIter: {}", hex_view(&value.as_ref())?);
            }

            Ok(Some(data_store))
        }
    }
}

#[cfg(test)]
mod test {
    use tokio::io::AsyncWriteExt;

    use super::*;

    #[tokio::test]
    async fn test_compress() -> Result<()> {
        {
            let mut file = File::create("helper/test_compress").await?;
            file.write_u32(233).await?;

            let mut writer = CompressionEncoder::new(file);

            for i in 0..10000u32 {
                writer
                    .write_all(vec![(i % 256) as u8; 128].as_slice())
                    .await?;
            }
            writer.shutdown().await?;

            let mut file = writer.into_inner();
            file.flush().await?;
            file.shutdown().await?;
        }

        {
            let mut file = File::open("helper/test_compress").await?;
            file.seek(SeekFrom::Start(4)).await?;

            let mut reader = CompressionDecoder::new(BufReader::new(file));

            let mut buf = vec![0u8; 128];
            for i in 0..10000u32 {
                reader.read_exact(&mut buf).await?;
                assert_eq!(buf, vec![(i % 256) as u8; 128]);
            }
        }

        Ok(())
    }
}
