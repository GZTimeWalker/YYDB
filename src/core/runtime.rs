use futures::Future;
use once_cell::sync::OnceCell;
use tokio::task::JoinHandle;

static RUNTIME: OnceCell<Runtime> = OnceCell::new();

pub struct Runtime {
    tokio_rt: tokio::runtime::Runtime,
}

pub fn init() {
    Runtime::global();
    info!("Runtime Initialized.");
}

#[inline]
pub fn block_on<F, T>(future: F) -> T
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    Runtime::global().block_on(future)
}

#[inline]
pub fn spawn<F, T>(future: F) -> JoinHandle<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    Runtime::global().spawn(future)
}

impl Runtime {
    pub fn global() -> &'static Runtime {
        RUNTIME.get_or_init(|| {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            Runtime { tokio_rt: rt }
        })
    }

    #[inline]
    pub fn spawn<F, T>(&self, future: F) -> tokio::task::JoinHandle<T>
    where
        F: std::future::Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        self.tokio_rt.spawn(future)
    }

    #[inline]
    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: std::future::Future,
    {
        self.tokio_rt.block_on(future)
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use futures::future;
    use std::time::Duration;

    #[test]
    fn it_works() {
        super::block_on(async {
            let futures = (0..10)
                .map(|i| {
                    super::spawn(async move {
                        let delay = rand::thread_rng().gen_range(0..=500);
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                        println!("Y-Engine async task: {} @ {:>3}ms", i, delay);
                    })
                })
                .collect::<Vec<_>>();

            future::join_all(futures).await;
        });
    }
}
