use futures::Future;
use indicatif::HumanBytes;
use std::{collections::BTreeMap, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::{
    structs::{
        table::{Table, TableId},
        SizedOnDisk,
    }
};

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

pub fn deinit() {
    run_async! {
        Runtime::global().close_all_tables().await;
    }

    info!("Runtime Deinitialized.");
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
#[inline(always)]
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
#[inline(always)]
pub fn spawn<F, T>(future: F) -> JoinHandle<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    Runtime::global().tokio_rt.spawn(future)
}

impl Runtime {
    pub fn global() -> &'static Runtime {
        &RUNTIME
    }

    #[inline]
    pub async fn contains_table(&self, id: &TableId) -> bool {
        self.tables.read().await.contains_key(id)
    }

    /// Open a table by name
    pub async fn open_table(&self, table_name: String) -> Option<TableId> {
        let id = TableId::new(&table_name);

        if self.tables.read().await.contains_key(&id) {
            Some(id)
        } else if let Ok(table) = Table::open(table_name).await {
            info!(
                "Table opened        : {}",
                HumanBytes(table.size_on_disk().await.unwrap()).to_string()
            );
            let id = table.id();
            self.tables.write().await.insert(id, Arc::new(table));
            Some(id)
        } else {
            None
        }
    }

    /// Get a table by id.
    #[inline(always)]
    pub async fn get_table(&self, id: &TableId) -> Option<Arc<Table>> {
        self.tables.read().await.get(id).cloned()
    }

    /// Close a table by id.
    #[inline(always)]
    pub async fn close_table(&self, id: &TableId) -> Option<Arc<Table>> {
        self.tables.write().await.remove(id)
    }

    /// insert a table by id.
    #[inline(always)]
    pub async fn insert_table(&self, id: TableId, table: Arc<Table>) {
        self.tables.write().await.insert(id, table);
    }

    /// Close all tables.
    #[inline(always)]
    pub async fn close_all_tables(&self) {
        self.tables.write().await.clear();
    }

    /// Shutdown the runtime.
    #[inline(always)]
    pub async fn shutdown(self) {
        self.tokio_rt.shutdown_timeout(Duration::from_secs(1));
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
