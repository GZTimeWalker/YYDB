use futures::Future;
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::structs::table::{Table, TableId};

lazy_static! {
    static ref RUNTIME: Runtime = {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        Runtime {
            tokio_rt: rt,
            tables: RwLock::new(BTreeMap::new()),
        }
    };
}

/// The runtime of YYDB.
///
/// There will be only one runtime in the whole process.
/// It is initialized in `rust_init` function, and hold
/// by `RUNTIME` static variable.
pub struct Runtime {
    tokio_rt: tokio::runtime::Runtime,
    tables: RwLock<BTreeMap<TableId, Arc<Table>>>,
}

/// Init the runtime of YYDB.
pub fn init() {
    Runtime::global();
    info!("Runtime Initialized.");
}

/// Runs a future to completion on the global runtime.
/// Should be called from sync code.
///
/// # Examples
///
/// ```
/// use yydb::core::block_on;
///
/// // Execute the future, blocking the current thread until completion
/// block_on(async {
///     println!("hello");
/// });
/// ```
#[inline]
pub fn block_on<F, T>(future: F) -> T
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    Runtime::global().tokio_rt.block_on(future)
}

/// Spawn a future to the global runtime.
///
/// This spawns the given future onto the runtime's executor,
/// returning a [`JoinHandle`] for the spawned future.
///
/// # Examples
///
/// ```
/// use yydb::core::spawn;
///
/// spawn(async {
///     println!("now running on a worker thread");
/// });
/// ```
#[inline]
pub fn spawn<F, T>(future: F) -> JoinHandle<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    Runtime::global().tokio_rt.spawn(future)
}

/// Open a table by name.
#[inline]
pub async fn open_table(table_name: String) -> Option<TableId> {
    if let Ok(table) = Table::open(table_name).await {
        let id = table.id();
        Runtime::global()
            .tables
            .write()
            .await
            .insert(id, Arc::new(table));
        Some(id)
    } else {
        None
    }
}

/// Get a table by id.
#[inline]
pub async fn get_table(id: &TableId) -> Option<Arc<Table>> {
    Runtime::global().tables.read().await.get(id).cloned()
}

/// Close a table by id.
#[inline]
pub async fn close_table(id: &TableId) -> Option<Arc<Table>> {
    Runtime::global().tables.write().await.remove(id)
}

impl Runtime {
    pub fn global() -> &'static Runtime {
        &RUNTIME
    }
}

#[cfg(test)]
mod tests {
    use futures::future;
    use rand::Rng;
    use std::time::Duration;

    #[test]
    fn it_works() {
        super::block_on(async {
            let futures = (0..10)
                .map(|_| rand::thread_rng().gen_range(0..=500))
                .map(|delay| {
                    super::spawn(async move {
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                    })
                })
                .collect::<Vec<_>>();

            future::join_all(futures).await;
        });
    }
}
