use std::future::*;
use tokio::task::{JoinHandle, JoinError};
pub mod runtime;

#[inline]
pub fn run_sync<F, T>(future: F) -> T
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    runtime::Runtime::global().block_on(future)
}

#[inline]
pub fn spawn<F, T>(future: F) -> JoinHandle<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    runtime::Runtime::global().spawn(future)
}

#[inline]
pub async fn wait_all(futures: Vec<JoinHandle<()>>) -> Result<(), JoinError> {
    for f in futures {
        f.await?;
    }

    Ok(())
}
