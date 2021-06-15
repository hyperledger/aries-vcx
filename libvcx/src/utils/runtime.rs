extern crate futures;
extern crate tokio_threadpool;

use std::collections::HashMap;
use std::ops::FnOnce;
use std::sync::Mutex;
use std::sync::Once;
use std::thread;

use self::futures::Future;
use self::tokio_threadpool::{Builder, ThreadPool};
use crate::settings::{DEFAULT_THREADPOOL_SIZE, MAX_THREADPOOL_SIZE};

lazy_static! {
    static ref THREADPOOL: Mutex<HashMap<u32, ThreadPool>> = Default::default();
}

static TP_INIT: Once = Once::new();

pub static mut TP_HANDLE: u32 = 0;


pub fn get_threadpool_size(size: Option<&str>) -> usize {
    let size = size.map_or(DEFAULT_THREADPOOL_SIZE, |s: &str| s.parse::<usize>().unwrap_or(DEFAULT_THREADPOOL_SIZE));

    if size > MAX_THREADPOOL_SIZE {
        MAX_THREADPOOL_SIZE
    } else {
        size
    }
}

pub fn init_runtime(size: Option<&str>) {
    trace!("threadpool::init >> size: {:?}", size);
    match get_threadpool_size(size) {
        0 => info!("threadpool_size is 0"),
        size => {
            info!("threadpool::init >> initializing threadpool of size {:?}", size);
            TP_INIT.call_once(|| {
                let pool = Builder::new().pool_size(size).build();

                THREADPOOL.lock().unwrap().insert(1, pool);

                unsafe { TP_HANDLE = 1; }
            });
        }
    }
}

pub fn execute<F>(future: F)
    where
        F: FnOnce() -> Result<(), ()> + Send + 'static {
    let handle;
    unsafe { handle = TP_HANDLE; }
    if get_threadpool_size(None) == 0 || handle == 0 {
        thread::spawn(future);
    } else {
        spawn_thread_in_pool(futures::lazy(future));
    }
}

fn spawn_thread_in_pool<F>(future: F)
    where
        F: Future<Item=(), Error=()> + Send + 'static {
    let handle;
    unsafe { handle = TP_HANDLE; }
    match THREADPOOL.lock().unwrap().get(&handle) {
        Some(x) => {
            let _n = x.spawn(future);
        }
        None => panic!("no threadpool!"),
    }
}

// extern crate futures;
//
// use std::collections::HashMap;
// use std::future::Future;
// use std::ops::FnOnce;
// use std::sync::Mutex;
// use std::sync::Once;
// use std::thread;
//
// use futures::future;
// use tokio::runtime::Runtime;
// use crate::settings;
// use std::sync::atomic::{Ordering, AtomicUsize};
//
// lazy_static! {
//     static ref THREADPOOL: Mutex<HashMap<u32, Runtime>> = Default::default();
// }
//
// static TP_INIT: Once = Once::new();
//
// pub static mut TP_HANDLE: u32 = 0;
//
// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct ThreadpoolConfig {
//     pub num_threads: Option<usize>,
// }

// pub fn init_runtime(config: ThreadpoolConfig) {
//     if config.num_threads == Some(0) {
//         warn!("init_runtime >>> threadpool_size was set to 0; every FFI call will executed on a new thread!");
//     } else {
//         let num_threads = config.num_threads.unwrap_or(4);
//         warn!("init_runtime >>> threadpool is using {} threads.", num_threads);
//         TP_INIT.call_once(|| {
//             let rt = tokio::runlibvcx/src/utils/runtime.rstime::Builder::new_multi_thread()
//                 .thread_name_fn(|| {
//                     static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
//                     let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
//                     format!("tokio-worker-vcxffi-{}", id)
//                 })
//                 .on_thread_start(|| debug!("Starting tokio runtime worker thread for vcx ffi."))
//                 .worker_threads(num_threads)
//                 .build()
//                 .unwrap();
//
//             THREADPOOL.lock().unwrap().insert(1, rt);
//             info!("Tokio runtime with threaded scheduler has been created.");
//
//             unsafe { TP_HANDLE = 1; }
//         });
//     }
// }
//
// pub fn execute<F>(closure: F)
//     where
//         F: FnOnce() -> Result<(), ()> + Send + 'static {
//     if TP_INIT.is_completed() {
//         execute_on_tokio(future::lazy(|_| closure()));
//     } else {
//         thread::spawn(closure);
//     }
// }
//
// fn execute_on_tokio<F>(future: F)
//     where
//         F: Future + Send + 'static,
//         F::Output: Send + 'static {
//     let handle;
//     unsafe { handle = TP_HANDLE; }
//     match THREADPOOL.lock().unwrap().get(&handle) {
//         Some(rt) => {
//             rt.spawn(future);
//         }
//         None => panic!("Tokio runtime not found! Forgot to call init_runtime?"),
//     }
// }
