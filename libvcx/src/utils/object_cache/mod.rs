use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use rand::Rng;

use crate::error::prelude::*;

pub struct ObjectCache<T> {
    pub cache_name: String,
    pub store: RwLock<HashMap<u32, RwLock<T>>>,
}

impl<T> ObjectCache<T> {
    pub fn new(cache_name: &str) -> ObjectCache<T> {
        ObjectCache {
            store: Default::default(),
            cache_name: cache_name.to_string(),
        }
    }

    fn _lock_store_read(&self) -> VcxResult<RwLockReadGuard<HashMap<u32, RwLock<T>>>> {
        match self.store.read() {
            Ok(g) => Ok(g),
            Err(e) => {
                error!("Unable to read-lock Object Store: {:?}", e);
                Err(VcxError::from_msg(VcxErrorKind::Common(10), format!("[ObjectCache: {}] Unable to lock Object Store: {:?}", self.cache_name, e)))
            }
        }
    }

    fn _lock_store_write(&self) -> VcxResult<RwLockWriteGuard<HashMap<u32, RwLock<T>>>> {
        match self.store.write() {
            Ok(g) => Ok(g),
            Err(e) => {
                error!("Unable to write-lock Object Store: {:?}", e);
                Err(VcxError::from_msg(VcxErrorKind::Common(10), format!("[ObjectCache: {}] Unable to lock Object Store: {:?}", self.cache_name, e)))
            }
        }
    }

    pub fn has_handle(&self, handle: u32) -> bool {
        let store = match self._lock_store_read() {
            Ok(g) => g,
            Err(_) => return false
        };
        store.contains_key(&handle)
    }

    pub fn get<F, R>(&self, handle: u32, closure: F) -> VcxResult<R>
        where F: Fn(&T) -> VcxResult<R> {
        let store = self._lock_store_read()?;
        match store.get(&handle) {
            Some(m) => match m.read() {
                Ok(obj) => closure(obj.deref()),
                Err(_) => Err(VcxError::from_msg(VcxErrorKind::Common(10), format!("[ObjectCache: {}] Unable to lock Object Store", self.cache_name))) //TODO better error
            },
            None => Err(VcxError::from_msg(VcxErrorKind::InvalidHandle, format!("[ObjectCache: {}] Object not found for handle: {}", self.cache_name, handle)))
        }
    }

    pub fn get_mut<F, R>(&self, handle: u32, closure: F) -> VcxResult<R>
        where F: Fn(&mut T) -> VcxResult<R> {
        let mut store = self._lock_store_write()?;
        match store.get_mut(&handle) {
            Some(m) => match m.write() {
                Ok(mut obj) => closure(obj.deref_mut()),
                Err(_) => Err(VcxError::from_msg(VcxErrorKind::Common(10), format!("[ObjectCache: {}] Unable to lock Object Store", self.cache_name))) //TODO better error
            },
            None => Err(VcxError::from_msg(VcxErrorKind::InvalidHandle, format!("[ObjectCache: {}] Object not found for handle: {}", self.cache_name, handle)))
        }
    }

    pub fn add(&self, obj: T) -> VcxResult<u32> {
        let mut store = self._lock_store_write()?;

        let mut new_handle = rand::thread_rng().gen::<u32>();
        loop {
            if !store.contains_key(&new_handle) {
                break;
            }
            new_handle = rand::thread_rng().gen::<u32>();
        }

        match store.insert(new_handle, RwLock::new(obj)) {
            Some(_) => Ok(new_handle),
            None => Ok(new_handle)
        }
    }

    pub fn insert(&self, handle: u32, obj: T) -> VcxResult<()> {
        let mut store = self._lock_store_write()?;

        match store.insert(handle, RwLock::new(obj)) {
            _ => Ok(()),
        }
    }

    pub fn release(&self, handle: u32) -> VcxResult<()> {
        let mut store = self._lock_store_write()?;
        match store.remove(&handle) {
            Some(_) => Ok(()),
            None => Err(VcxError::from_msg(VcxErrorKind::InvalidHandle, format!("[ObjectCache: {}] Object not found for handle: {}", self.cache_name, handle)))
        }
    }

    pub fn drain(&self) -> VcxResult<()> {
        let mut store = self._lock_store_write()?;
        Ok(store.clear())
    }

    pub fn len(&self) -> VcxResult<usize> {
        let store = self._lock_store_read()?;
        Ok(store.len())
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::object_cache::ObjectCache;
    use crate::utils::devsetup::SetupDefaults;

    #[test]
    #[cfg(feature = "general_test")]
    fn create_test() {
        let _setup = SetupDefaults::init();

        let _c: ObjectCache<u32> = ObjectCache::new("cache0-u32");
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn get_closure() {
        let _setup = SetupDefaults::init();

        let test: ObjectCache<u32> = ObjectCache::new("cache1-u32");
        let handle = test.add(2222).unwrap();
        let rtn = test.get(handle, |obj| Ok(obj.clone()));
        assert_eq!(2222, rtn.unwrap())
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn to_string_test() {
        let _setup = SetupDefaults::init();

        let test: ObjectCache<u32> = ObjectCache::new("cache2-u32");
        let handle = test.add(2222).unwrap();
        let string: String = test.get(handle, |_| {
            Ok(String::from("TEST"))
        }).unwrap();

        assert_eq!("TEST", string);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn mut_object_test() {
        let _setup = SetupDefaults::init();

        let test: ObjectCache<String> = ObjectCache::new("cache3-string");
        let handle = test.add(String::from("TEST")).unwrap();

        test.get_mut(handle, |obj| {
            obj.to_lowercase();
            Ok(())
        }).unwrap();

        let string: String = test.get(handle, |obj| {
            Ok(obj.clone())
        }).unwrap();

        assert_eq!("TEST", string);
    }
}
