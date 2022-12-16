use async_trait::async_trait;

use crate::utils::*;

#[async_trait]
pub trait AsyncKVStoreRead: Send + 'static + Sized {
    /// Get the value specified by the key
    async fn get(&self, key: Key) -> DataStore;

    /// Get the number of keys in the store
    async fn len(&self) -> usize;
}

#[async_trait]
pub trait AsyncKVStoreWrite: AsyncKVStoreRead {
    /// Set the value specified by the key
    async fn set(&self, key: Key, value: DataInner);

    /// Delete the value specified by the key
    async fn delete(&self, key: Key);
}

#[async_trait]
pub trait SizedOnDisk {
    /// Get the size of the store on disk
    async fn size_on_disk(&self) -> Result<u64>;
}

#[async_trait]
pub trait AsyncFromIO: Sized {
    /// Create a new instance from an IOHandler
    async fn from_io(io: &IOHandler) -> Result<Self>;
}

#[async_trait]
pub trait AsyncToIO {
    /// Write the instance to an IOHandler
    async fn to_io(&self, io: &IOHandler) -> Result<()>;
}
