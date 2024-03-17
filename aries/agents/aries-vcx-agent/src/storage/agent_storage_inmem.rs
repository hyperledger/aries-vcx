use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use super::AgentStorage;
use crate::error::*;

pub struct AgentStorageInMem<T>
where
    T: Clone,
{
    pub name: String,
    pub store: RwLock<HashMap<String, Mutex<T>>>,
}

impl<T> AgentStorageInMem<T>
where
    T: Clone,
{
    pub fn new(name: &str) -> Self {
        Self {
            store: Default::default(),
            name: name.to_string(),
        }
    }

    fn lock_store_read(&self) -> AgentResult<RwLockReadGuard<HashMap<String, Mutex<T>>>> {
        match self.store.read() {
            Ok(g) => Ok(g),
            Err(e) => Err(AgentError::from_msg(
                AgentErrorKind::LockError,
                &format!(
                    "Unable to obtain read lock for in-memory protocol store {} due to error: {:?}",
                    self.name, e
                ),
            )),
        }
    }

    fn lock_store_write(&self) -> AgentResult<RwLockWriteGuard<HashMap<String, Mutex<T>>>> {
        match self.store.write() {
            Ok(g) => Ok(g),
            Err(e) => {
                error!("Unable to write-lock Object Store: {:?}", e);
                Err(AgentError::from_msg(
                    AgentErrorKind::LockError,
                    &format!(
                        "Unable to obtain write lock for in-memory protocol store {} due to \
                         error: {:?}",
                        self.name, e
                    ),
                ))
            }
        }
    }
}

impl<T> AgentStorage<T> for AgentStorageInMem<T>
where
    T: Clone,
{
    type Value = Mutex<T>;

    fn get(&self, id: &str) -> AgentResult<T> {
        let store = self.lock_store_read()?;
        match store.get(id) {
            Some(m) => match m.lock() {
                Ok(obj) => Ok((*obj.deref()).clone()),
                Err(_) => Err(AgentError::from_msg(
                    AgentErrorKind::LockError,
                    &format!(
                        "Unable to obtain lock for object {} in in-memory store {}",
                        id, self.name
                    ),
                )), //TODO better error
            },
            None => Err(AgentError::from_msg(
                AgentErrorKind::NotFound,
                &format!("Object {} not found in in-memory store {}", id, self.name),
            )),
        }
    }

    fn insert(&self, id: &str, obj: T) -> AgentResult<String> {
        info!("Inserting object {} into in-memory store {}", id, self.name);
        let mut store = self.lock_store_write()?;

        match store.insert(id.to_string(), Mutex::new(obj)) {
            Some(_) => Ok(id.to_string()),
            None => Ok(id.to_string()),
        }
    }

    fn contains_key(&self, id: &str) -> bool {
        let store = match self.lock_store_read() {
            Ok(g) => g,
            Err(_) => return false,
        };
        store.contains_key(id)
    }

    fn find_by<F>(&self, closure: F) -> AgentResult<Vec<String>>
    where
        F: FnMut((&String, &Self::Value)) -> Option<String>,
    {
        let store = self.lock_store_read()?;
        Ok(store.iter().filter_map(closure).collect())
    }
}
