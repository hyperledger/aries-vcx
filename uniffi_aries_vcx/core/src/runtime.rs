use std::future::Future;

use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().expect("Error creating tokio runtime"));

/// Block the current thread on an async task, when not running inside the scheduler.
pub fn block_on<R>(f: impl Future<Output = R>) -> R {
    RUNTIME.block_on(f)
}