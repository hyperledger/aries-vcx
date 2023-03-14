use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use super::Storage;
use crate::error::*;

pub struct ObjectCache<T>
where
    T: Clone,
{
    pub cache_name: String,
    pub store: RwLock<HashMap<String, Mutex<T>>>,
}

impl<T> ObjectCache<T>
where
    T: Clone,
{
    pub fn new(cache_name: &str) -> Self {
        Self {
            store: Default::default(),
            cache_name: cache_name.to_string(),
        }
    }

    fn _lock_store_read(&self) -> AgentResult<RwLockReadGuard<HashMap<String, Mutex<T>>>> {
        match self.store.read() {
            Ok(g) => Ok(g),
            Err(e) => {
                error!("Unable to read-lock Object Store: {:?}", e);
                Err(AgentError::from_msg(
                    AgentErrorKind::LockError,
                    &format!(
                        "[ObjectCache: {}] Unable to lock Object Store: {:?}",
                        self.cache_name, e
                    ),
                ))
            }
        }
    }

    fn _lock_store_write(&self) -> AgentResult<RwLockWriteGuard<HashMap<String, Mutex<T>>>> {
        match self.store.write() {
            Ok(g) => Ok(g),
            Err(e) => {
                error!("Unable to write-lock Object Store: {:?}", e);
                Err(AgentError::from_msg(
                    AgentErrorKind::LockError,
                    &format!(
                        "[ObjectCache: {}] Unable to lock Object Store: {:?}",
                        self.cache_name, e
                    ),
                ))
            }
        }
    }
}

impl<T> Storage<T> for ObjectCache<T>
where
    T: Clone,
{
    type Value = Mutex<T>;

    fn get(&self, id: &str) -> AgentResult<T> {
        let store = self._lock_store_read()?;
        match store.get(id) {
            Some(m) => match m.lock() {
                Ok(obj) => Ok((*obj.deref()).clone()),
                Err(_) => Err(AgentError::from_msg(
                    AgentErrorKind::LockError,
                    &format!("[ObjectCache: {}] Unable to lock Object Store", self.cache_name),
                )), //TODO better error
            },
            None => Err(AgentError::from_msg(
                AgentErrorKind::NotFound,
                &format!("[ObjectCache: {}] Object not found for id: {}", self.cache_name, id),
            )),
        }
    }

    fn insert(&self, id: &str, obj: T) -> AgentResult<String> {
        let mut store = self._lock_store_write()?;

        match store.insert(id.to_string(), Mutex::new(obj)) {
            Some(_) => Ok(id.to_string()),
            None => Ok(id.to_string()),
        }
    }

    fn contains_key(&self, id: &str) -> bool {
        let store = match self._lock_store_read() {
            Ok(g) => g,
            Err(_) => return false,
        };
        store.contains_key(id)
    }

    fn find_by<F>(&self, closure: F) -> AgentResult<Vec<String>>
    where
        F: FnMut((&String, &Self::Value)) -> Option<String>,
    {
        let store = self._lock_store_read()?;
        Ok(store.iter().filter_map(closure).collect())
    }
}
