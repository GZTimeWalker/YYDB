use once_cell::sync::OnceCell;

static RUNTIME: OnceCell<Runtime> = OnceCell::new();

pub struct Runtime {
    tokio_rt: tokio::runtime::Runtime,
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

pub fn init() {
    Runtime::global();
    info!("Y-Engine Runtime Initialized.");
}
