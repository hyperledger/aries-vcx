use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use futures::future::BoxFuture;
use rand::Rng;

use crate::errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult};

pub struct ObjectCache<T>
where
    T: Clone,
{
    pub cache_name: String,
    pub store: RwLock<HashMap<u32, Mutex<T>>>,
}

impl<T> ObjectCache<T>
where
    T: Clone,
{
    pub fn new(cache_name: &str) -> ObjectCache<T> {
        ObjectCache {
            store: Default::default(),
            cache_name: cache_name.to_string(),
        }
    }

    fn _lock_store_read(&self) -> LibvcxResult<RwLockReadGuard<HashMap<u32, Mutex<T>>>> {
        match self.store.read() {
            Ok(g) => Ok(g),
            Err(e) => Err(LibvcxError::from_msg(
                LibvcxErrorKind::ObjectAccessError,
                format!(
                    "[ObjectCache: {}] _lock_store_read >> Unable to lock Object Store: {:?}",
                    self.cache_name, e
                ),
            )),
        }
    }

    fn _lock_store_write(&self) -> LibvcxResult<RwLockWriteGuard<HashMap<u32, Mutex<T>>>> {
        match self.store.write() {
            Ok(g) => Ok(g),
            Err(e) => Err(LibvcxError::from_msg(
                LibvcxErrorKind::ObjectAccessError,
                format!(
                    "[ObjectCache: {}] _lock_store_write >> Unable to lock Object Store: {:?}",
                    self.cache_name, e
                ),
            )),
        }
    }

    pub fn has_handle(&self, handle: u32) -> bool {
        let store = match self._lock_store_read() {
            Ok(g) => g,
            Err(_) => return false,
        };
        store.contains_key(&handle)
    }

    pub fn get<F, R>(&self, handle: u32, closure: F) -> LibvcxResult<R>
    where
        F: Fn(&T) -> LibvcxResult<R>,
    {
        let store = self._lock_store_read()?;
        match store.get(&handle) {
            Some(m) => match m.lock() {
                Ok(obj) => closure(obj.deref()),
                Err(_) => Err(LibvcxError::from_msg(
                    LibvcxErrorKind::ObjectAccessError,
                    format!("[ObjectCache: {}] get >> Unable to lock Object Store", self.cache_name),
                )),
            },
            None => Err(LibvcxError::from_msg(
                LibvcxErrorKind::InvalidHandle,
                format!(
                    "[ObjectCache: {}] get >> Object not found for handle: {}",
                    self.cache_name, handle
                ),
            )),
        }
    }

    pub fn get_cloned(&self, handle: u32) -> LibvcxResult<T> {
        let store = self._lock_store_read()?;
        match store.get(&handle) {
            Some(m) => match m.lock() {
                Ok(obj) => Ok((*obj.deref()).clone()),
                Err(_) => Err(LibvcxError::from_msg(
                    LibvcxErrorKind::ObjectAccessError,
                    format!(
                        "[ObjectCache: {}] get_cloned >> Unable to lock Object Store",
                        self.cache_name
                    ),
                )),
            },
            None => Err(LibvcxError::from_msg(
                LibvcxErrorKind::InvalidHandle,
                format!(
                    "[ObjectCache: {}] get_cloned >> Object not found for handle: {}",
                    self.cache_name, handle
                ),
            )),
        }
    }

    pub async fn get_async<'up, F: 'up, R>(&self, handle: u32, closure: F) -> LibvcxResult<R>
    where
        for<'r> F: Fn(&'r T, [&'r &'up (); 0]) -> BoxFuture<'r, LibvcxResult<R>>,
    {
        let store = self._lock_store_read()?;
        match store.get(&handle) {
            Some(m) => match m.lock() {
                Ok(obj) => closure(obj.deref(), []).await,
                Err(_) => Err(LibvcxError::from_msg(
                    LibvcxErrorKind::ObjectAccessError,
                    format!(
                        "[ObjectCache: {}] get_async >> Unable to lock Object Store",
                        self.cache_name
                    ),
                )),
            },
            None => Err(LibvcxError::from_msg(
                LibvcxErrorKind::InvalidHandle,
                format!(
                    "[ObjectCache: {}] get_async >> Object not found for handle: {}",
                    self.cache_name, handle
                ),
            )),
        }
    }

    pub fn get_mut<F, R>(&self, handle: u32, closure: F) -> LibvcxResult<R>
    where
        F: Fn(&mut T) -> LibvcxResult<R>,
    {
        let mut store = self._lock_store_write()?;
        match store.get_mut(&handle) {
            Some(m) => match m.get_mut() {
                Ok(mut obj) => closure(obj.deref_mut()),
                Err(_) => Err(LibvcxError::from_msg(
                    LibvcxErrorKind::ObjectAccessError,
                    format!(
                        "[ObjectCache: {}] get_mut >> Unable to lock Object Store",
                        self.cache_name
                    ),
                )),
            },
            None => Err(LibvcxError::from_msg(
                LibvcxErrorKind::InvalidHandle,
                format!(
                    "[ObjectCache: {}] get_mut >> Object not found for handle: {}",
                    self.cache_name, handle
                ),
            )),
        }
    }

    pub async fn get_mut_async<'up, F: 'up, R>(&self, handle: u32, closure: F) -> LibvcxResult<R>
    where
        for<'r> F: Fn(&'r mut T, [&'r &'up (); 0]) -> BoxFuture<'r, LibvcxResult<R>>,
    {
        let mut store = self._lock_store_write()?;
        match store.get_mut(&handle) {
            Some(m) => match m.get_mut() {
                Ok(mut obj) => closure(obj.deref_mut(), []).await,
                Err(_) => Err(LibvcxError::from_msg(
                    LibvcxErrorKind::ObjectAccessError,
                    format!(
                        "[ObjectCache: {}] get_mut_async >> Unable to lock Object Store",
                        self.cache_name
                    ),
                )),
            },
            None => Err(LibvcxError::from_msg(
                LibvcxErrorKind::InvalidHandle,
                format!(
                    "[ObjectCache: {}] get_mut_async >> Object not found for handle: {}",
                    self.cache_name, handle
                ),
            )),
        }
    }

    pub fn add(&self, obj: T) -> LibvcxResult<u32> {
        trace!("[ObjectCache: {}] add >> Adding object to cache", self.cache_name);
        let mut store = self._lock_store_write()?;

        let mut new_handle = rand::thread_rng().gen::<u32>();
        loop {
            if !store.contains_key(&new_handle) {
                break;
            }
            new_handle = rand::thread_rng().gen::<u32>();
        }

        match store.insert(new_handle, Mutex::new(obj)) {
            Some(_) => {
                warn!(
                    "[ObjectCache: {}] add >> Object already exists for handle: {}",
                    self.cache_name, new_handle
                );
                Err(LibvcxError::from_msg(
                    LibvcxErrorKind::InvalidHandle,
                    format!(
                        "[ObjectCache: {}] add >> generated handle {} conflicts with existing handle, failed to store \
                         object",
                        self.cache_name, new_handle
                    ),
                ))
            }
            None => {
                trace!(
                    "[ObjectCache: {}] add >> Object added to cache for handle: {}",
                    self.cache_name,
                    new_handle
                );
                Ok(new_handle)
            }
        }
    }

    pub fn insert(&self, handle: u32, obj: T) -> LibvcxResult<()> {
        trace!(
            "[ObjectCache: {}] insert >> Inserting object with handle: {}",
            self.cache_name,
            handle
        );
        let mut store = self._lock_store_write()?;

        store.insert(handle, Mutex::new(obj));
        Ok(())
    }

    pub fn release(&self, handle: u32) -> LibvcxResult<()> {
        trace!(
            "[ObjectCache: {}] release >> Releasing object with handle: {}",
            self.cache_name,
            handle
        );
        let mut store = self._lock_store_write()?;
        match store.remove(&handle) {
            Some(_) => {}
            None => {
                warn!(
                    "[ObjectCache: {}] release >> Object not found for handle: {}. Perhaps already released?",
                    self.cache_name, handle
                );
            }
        };
        Ok(())
    }

    pub fn drain(&self) -> LibvcxResult<()> {
        warn!("[ObjectCache: {}] drain >> Draining object cache", self.cache_name);
        let mut store = self._lock_store_write()?;
        store.clear();
        Ok(())
    }

    pub fn len(&self) -> LibvcxResult<usize> {
        let store = self._lock_store_read()?;
        Ok(store.len())
    }
}

#[cfg(test)]
mod tests {
    use aries_vcx::utils::devsetup::SetupDefaults;

    use crate::api_vcx::api_handle::object_cache::ObjectCache;

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
        let string: String = test.get(handle, |_| Ok(String::from("TEST"))).unwrap();

        assert_eq!("TEST", string);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn mut_object_test() {
        let _setup = SetupDefaults::init();

        let test: ObjectCache<String> = ObjectCache::new("cache3-string");
        let handle = test.add(String::from("TEST")).unwrap();

        test.get_mut(handle, |obj| {
            obj.make_ascii_uppercase();
            Ok(())
        })
        .unwrap();

        let string: String = test.get(handle, |obj| Ok(obj.clone())).unwrap();

        assert_eq!("TEST", string);
    }
}
