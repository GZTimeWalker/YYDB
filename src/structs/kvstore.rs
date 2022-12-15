use async_trait::async_trait;

use crate::utils::*;


#[async_trait]
pub trait AsyncKVStoreRead: Send + 'static + Sized {
    /// Get the value specified by the key
    async fn get(&self, key: u64) -> DataStore;

    /// Get the number of keys in the store
    async fn len(&self) -> usize;
}

#[async_trait]
pub trait AsyncKVStoreWrite: AsyncKVStoreRead {
    /// Set the value specified by the key
    async fn set(&self, key: u64, value: Vec<u8>);

    /// Delete the value specified by the key
    async fn delete(&self, key: u64);
}

#[async_trait]
pub trait SizedOnDisk {
    /// Get the size of the store on disk
    async fn size_on_disk(&self) -> Result<u64>;
}
