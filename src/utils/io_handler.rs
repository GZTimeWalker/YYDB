use async_trait::async_trait;
use crc32fast::Hasher;
use std::{path::PathBuf, sync::Arc};
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{Mutex, MutexGuard};

use crate::structs::kvstore::SizedOnDisk;
use crate::structs::lsm::sstable::SSTableKey;
use crate::utils::error::Result;

#[derive(Debug)]
pub struct IOHandler {
    pub file_path: Arc<PathBuf>,
    file: Mutex<File>,
}

impl IOHandler {
    pub async fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let path: PathBuf = path.into();

        let file_opt = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(&path)
            .await?;

        trace!("Opening file: {:?}", path);

        Ok(Self {
            file_path: Arc::new(path.clone()),
            file: Mutex::new(file_opt),
        })
    }

    /// Get the file size
    pub async fn file_size(&self) -> Result<u64> {
        Ok(fs::metadata(self.file_path.as_ref()).await?.len())
    }

    /// Get the checksum
    pub async fn checksum(&self) -> Result<u32> {
        let mut hasher = Hasher::new();
        let mut reader = self.file.lock().await;
        let mut buf = [0u8; 32 * 1024];
        loop {
            let len = reader.read(&mut buf).await?;
            if len == 0 {
                break;
            }
            hasher.update(&buf[..len]);
        }
        Ok(hasher.finalize())
    }

    pub async fn inner(&self) -> Result<MutexGuard<File>> {
        Ok(self.file.lock().await)
    }

    pub async fn is_empty(&self) -> Result<bool> {
        Ok(self.file_size().await? == 0)
    }

    pub async fn base_dir(&self) -> PathBuf {
        self.file_path.as_ref().parent().unwrap().to_path_buf()
    }

    /// delete the file
    ///
    /// Warning: The writer and reader couldn't be used after this function is called
    pub async fn delete(&self) -> Result<()> {
        let mut file = self.file.lock().await;
        file.shutdown().await?;

        fs::remove_file(self.file_path.as_ref()).await?;
        Ok(())
    }

    pub async fn clone(&self) -> Result<Self> {
        Self::new(self.file_path.as_ref()).await
    }
}

impl Drop for IOHandler {
    fn drop(&mut self) {
        trace!("Closing file: {:?}", self.file_path);

        futures::executor::block_on(async move {
            let mut file = self.file.lock().await;
            file.shutdown().await.unwrap();
        });
    }
}

#[async_trait]
impl SizedOnDisk for IOHandler {
    async fn size_on_disk(&self) -> Result<u64> {
        Ok(fs::metadata(self.file_path.as_ref()).await?.len())
    }
}

#[derive(Debug)]
pub struct IOHandlerFactory {
    base_dir: Arc<PathBuf>,
}

impl IOHandlerFactory {
    pub fn new(table_name: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: Arc::new(table_name.into()),
        }
    }

    pub async fn create(&self, key: SSTableKey) -> Result<IOHandler> {
        let mut path = self.base_dir.to_path_buf();
        path.push(format!("{:x}.l{}", key, key.level()));

        IOHandler::new(&path).await
    }
}

#[cfg(test)]
mod test {
    use std::io::SeekFrom;

    use tokio::io::AsyncSeekExt;

    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let test_dir = "helper/io_test";

        std::fs::remove_dir_all(test_dir).ok();
        std::fs::create_dir_all(test_dir).unwrap();

        let factory = IOHandlerFactory::new(test_dir);
        let key = SSTableKey::new(0u64);

        let io_handler = factory.create(key).await?;

        {
            let mut io = io_handler.inner().await?;
            io.write(b"hello world").await?;

            io.flush().await?;

            io.seek(SeekFrom::Start(0)).await?;

            let mut buf = [0u8; 11];
            io.read(&mut buf).await?;

            assert_eq!(b"hello world", &buf);

            io.seek(SeekFrom::Start(0)).await?;
        }

        let checksum = io_handler.checksum().await?;
        assert_eq!(checksum, 0xd4a1185);

        io_handler.delete().await?;
        Ok(())
    }
}
