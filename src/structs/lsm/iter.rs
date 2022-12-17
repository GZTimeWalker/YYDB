use std::io::SeekFrom;

use futures::Future;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use crate::{
    structs::{AsyncKvIterator, SSTABLE_MAGIC_NUMBER},
    utils::*,
};

pub const SSTABLE_ITER_BUF_SIZE: usize = 1024;

#[derive(Debug)]
pub struct SSTableIter {
    io: IOHandler,
    buf: Vec<u8>,
    row_size: usize,
    checksum: Option<u32>,
}

impl SSTableIter {
    pub async fn new(io: IOHandler, row_size: u32) -> Result<Self> {
        let mut iter = Self {
            io,
            buf: Vec::new(),
            row_size: row_size as usize,
            checksum: None,
        };

        if iter.io.inner().await?.metadata().await?.len() == 0 {
            return Ok(iter);
        }

        let mut file_io = iter.io.inner().await?;
        file_io.seek(SeekFrom::Start(0)).await?;

        let magic_number = file_io.read_u32().await?;

        if magic_number != SSTABLE_MAGIC_NUMBER {
            return Err(DbError::InvalidMagicNumber);
        }

        iter.checksum = Some(file_io.read_u32().await?);
        drop(file_io);

        Ok(iter)
    }

    pub fn init_iter(&mut self) -> Result<()> {
        self.buf.clear();
        self.buf
            .resize(std::cmp::max(SSTABLE_ITER_BUF_SIZE, self.row_size * 2), 0);
        Ok(())
    }
}

impl AsyncKvIterator for SSTableIter {
    type NextFuture<'a> = impl Future<Output = Result<(Key, DataInner)>> + 'a;
    fn next(&mut self) -> Self::NextFuture<'_> {
        async move {
            todo!();
        }
    }
}
