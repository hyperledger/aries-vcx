extern crate futures;

use once_cell::sync::Lazy;
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};

use futures::future::BoxFuture;
use tokio::runtime::Runtime;
use crate::api_lib::errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult};

static RT: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .thread_name_fn(|| {
            static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
            let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
            format!("tokio-worker-vcxffi-{}", id)
        })
        .on_thread_start(|| debug!("Starting tokio runtime worker thread for vcx ffi."))
        .enable_all()
        .build()
        .unwrap()
});

#[derive(Deserialize)]
pub struct ThreadpoolConfig {
    pub num_threads: Option<usize>,
}

pub fn init_threadpool(config: &str) -> LibvcxResult<()> {
    let config: ThreadpoolConfig = serde_json::from_str(config).map_err(|err| {
        LibvcxError::from_msg(
            LibvcxErrorKind::InvalidJson,
            format!("Failed to deserialize threadpool config {:?}, err: {:?}", config, err),
        )
    })?;

    init_runtime(config);

    Ok(())
}

fn init_runtime(_config: ThreadpoolConfig) {
    let _check = RT.enter();
}

pub fn execute<F>(closure: F)
where
    F: FnOnce() -> Result<(), ()> + Send + 'static,
{
    execute_on_tokio(async move {
        closure()
    });
}

pub fn execute_async<F>(future: BoxFuture<'static, Result<(), ()>>) {
    execute_on_tokio(future);
}

fn execute_on_tokio<F>(future: F)
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    RT.spawn(future);
}
