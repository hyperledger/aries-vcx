extern crate futures;

use std::collections::HashMap;
use std::future::Future;
use std::ops::FnOnce;
use std::sync::Mutex;
use std::sync::Once;
use std::thread;

use futures::future;
use tokio::runtime::Runtime;
use crate::settings;
use std::sync::atomic::{Ordering, AtomicUsize};

lazy_static! {
    static ref THREADPOOL: Mutex<HashMap<u32, Runtime>> = Default::default();
}

static TP_INIT: Once = Once::new();

pub static mut TP_HANDLE: u32 = 0;

pub fn init_runtime() {
    info!("init_runtime >>>");
    let size = settings::get_threadpool_size();

    if size == 0 {
        warn!("threadpool_size was set to 0; every FFI call will executed on a new thread!");
    } else {
        TP_INIT.call_once(|| {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .thread_name_fn(|| {
                    static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
                    let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
                    format!("tokio-worker-vcxffi-{}", id)
                })
                .on_thread_start(|| debug!("Starting tokio runtime worker thread for vcx ffi."))
                .build()
                .unwrap();

            THREADPOOL.lock().unwrap().insert(1, rt);
            warn!("Tokio runtime with threaded scheduler has been created.");

            unsafe { TP_HANDLE = 1; }
        });
    }
}

pub fn execute<F>(closure: F)
    where
        F: FnOnce() -> Result<(), ()> + Send + 'static {
    trace!("Closure is going to be executed.");
    let handle;
    unsafe { handle = TP_HANDLE; }
    if settings::get_threadpool_size() == 0 || handle == 0 {
        thread::spawn(closure);
    } else {
        execute_on_tokio(future::lazy(|_| closure()));
    }
}

fn execute_on_tokio<F>(future: F)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static {
    let handle;
    unsafe { handle = TP_HANDLE; }
    match THREADPOOL.lock().unwrap().get(&handle) {
        Some(rt) => {
            debug!("Executing a future on tokio runtime");
            rt.spawn(future);

        }
        None => panic!("Tokio runtime not found! Forgot to call init_runtime?"),
    }
}
