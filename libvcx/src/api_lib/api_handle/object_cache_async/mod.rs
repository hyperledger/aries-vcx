use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;
use crate::async_std::sync::{Mutex, RwLock};
use futures::future::BoxFuture;

use rand::Rng;

use crate::error::prelude::*;

pub struct ObjectCacheAsync<T> {
    pub cache_name: String,
    pub store: RwLock<HashMap<u32, Mutex<T>>>,
}

impl<T> ObjectCacheAsync<T> {
    pub fn new(cache_name: &str) -> Self {
        Self {
            store: Default::default(),
            cache_name: cache_name.to_string(),
        }
    }

    pub async fn has_handle(&self, handle: u32) -> bool {
        let store = self.store.read().await;
        store.contains_key(&handle)
    }

    pub async fn get<'up, F: 'up, R>(&self, handle: u32, closure: F) -> VcxResult<R>
    where
        for<'r> F: Fn(&'r T, [&'r &'up (); 0]) -> BoxFuture<'r, VcxResult<R>>
    {
        let store = self.store.read().await;
        match store.get(&handle) {
            Some(m) => {
                let obj = m.lock().await;
                closure(obj.deref(), []).await
            },
            None => Err(VcxError::from_msg(VcxErrorKind::InvalidHandle, format!("[ObjectCacheAsync: {}] Object not found for handle: {}", self.cache_name, handle)))
        }
    }

    pub async fn get_mut<'up, F: 'up, R>(&self, handle: u32, closure: F) -> VcxResult<R>
    where
        for<'r> F: Fn(&'r mut T, [&'r &'up (); 0]) -> BoxFuture<'r, VcxResult<R>>
    {
        let mut store = self.store.write().await;
        match store.get_mut(&handle) {
            Some(m) => {
                let mut obj = m.lock().await;
                closure(obj.deref_mut(), []).await
            },
            None => Err(VcxError::from_msg(VcxErrorKind::InvalidHandle, format!("[ObjectCacheAsync: {}] Object not found for handle: {}", self.cache_name, handle)))
        }
    }

    pub async fn add(&self, obj: T) -> VcxResult<u32> {
        let mut store = self.store.write().await;

        let mut new_handle = rand::thread_rng().gen::<u32>();
        loop {
            if !store.contains_key(&new_handle) {
                break;
            }
            new_handle = rand::thread_rng().gen::<u32>();
        }

        match store.insert(new_handle, Mutex::new(obj)) {
            Some(_) => Ok(new_handle),
            None => Ok(new_handle)
        }
    }

    pub async fn insert(&self, handle: u32, obj: T) -> VcxResult<()> {
        let mut store = self.store.write().await;

        match store.insert(handle, Mutex::new(obj)) {
            _ => Ok(()),
        }
    }

    pub async fn release(&self, handle: u32) -> VcxResult<()> {
        let mut store = self.store.write().await;
        match store.remove(&handle) {
            Some(_) => Ok(()),
            None => Err(VcxError::from_msg(VcxErrorKind::InvalidHandle, format!("[ObjectCacheAsync: {}] Object not found for handle: {}", self.cache_name, handle)))
        }
    }

    pub async fn drain(&self) -> VcxResult<()> {
        let mut store = self.store.write().await;
        Ok(store.clear())
    }

    pub async fn len(&self) -> VcxResult<usize> {
        let store = self.store.read().await;
        Ok(store.len())
    }
}
