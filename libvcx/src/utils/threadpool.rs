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

pub fn init(size: Option<&str>) {
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

pub fn spawn<F>(future: F)
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
