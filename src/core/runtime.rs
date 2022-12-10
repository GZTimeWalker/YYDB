use futures::Future;
use once_cell::sync::OnceCell;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
use tokio::task::JoinHandle;

use crate::structs::table::{Table, TableId};

static RUNTIME: OnceCell<Runtime> = OnceCell::new();

/// The runtime of YYDB.
///
/// There will be only one runtime in the whole process.
/// It is initialized in `rust_init` function, and hold
/// by `RUNTIME` static variable.
pub struct Runtime {
    tokio_rt: tokio::runtime::Runtime,
    tables: Mutex<BTreeMap<TableId, Arc<Table>>>,
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
///
/// # Examples
///
/// ```
/// use yydb::core::open_table;
///
/// let id = open_table("test_table").unwrap();
/// ```
#[inline]
pub fn open_table(table_name: &str) -> Option<TableId> {
    if let Ok(table) = Table::open(table_name) {
        let id = table.id();
        Runtime::global().tables.lock().unwrap().insert(id, Arc::new(table));
        Some(id)
    } else {
        None
    }
}

/// Get a table by id.
///
/// # Examples
///
/// ```
/// use yydb::core::{open_table, get_table};
///
/// let id = open_table("test_table").unwrap();
/// let table = get_table(&id).unwrap();
/// ```
#[inline]
pub fn get_table(id: &TableId) -> Option<Arc<Table>> {
    Runtime::global().tables.lock().unwrap().get(id).cloned()
}

/// Close a table by id.
///
/// # Examples
///
/// ```
/// use yydb::core::{open_table, close_table};
///
/// let id = open_table("test_table").unwrap();
/// close_table(&id);
/// ```
#[inline]
pub fn close_table(id: &TableId) {
    if let Some(table) = get_table(id) {
        table.close();
        Runtime::global().tables.lock().unwrap().remove(&id);
    }
}

impl Runtime {
    pub fn global() -> &'static Runtime {
        RUNTIME.get_or_init(|| {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();

            Runtime {
                tokio_rt: rt,
                tables: Mutex::new(BTreeMap::new()),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use futures::future;
    use rand::Rng;
    use std::time::Duration;

    #[test]
    fn async_runtime_works() {
        super::block_on(async {
            let futures = (0..10)
                .map(|i| {
                    super::spawn(async move {
                        let delay = rand::thread_rng().gen_range(0..=500);
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                        println!("YYDB async task: {} @ {:>3}ms", i, delay);
                    })
                })
                .collect::<Vec<_>>();

            future::join_all(futures).await;
        });
    }
}
