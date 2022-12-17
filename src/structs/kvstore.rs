use async_trait::async_trait;
use futures::Future;

use crate::utils::*;

#[async_trait]
pub trait AsyncKvStoreRead: Send + 'static + Sized {
    /// Get the value specified by the key
    async fn get(&self, key: Key) -> Result<DataStore>;

    /// Get the number of keys in the store
    async fn len(&self) -> usize;
}

#[async_trait]
pub trait AsyncKvStoreWrite: AsyncKvStoreRead {
    /// Set the value specified by the key
    async fn set(&self, key: Key, value: DataInner);

    /// Delete the value specified by the key
    async fn delete(&self, key: Key);
}

pub trait AsyncIterator<T> {
    type NextFuture<'a>: Future<Output = Result<Option<T>>>
    where
        Self: 'a;

    /// Get the next key-value pair
    fn next(&mut self) -> Self::NextFuture<'_>;
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
