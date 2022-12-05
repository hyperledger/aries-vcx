use crate::AgentResult;

pub(crate) mod object_cache;

pub trait Storage<T> {
    type Value;
    fn get(&self, id: &str) -> AgentResult<T>;
    fn insert(&self, id: &str, obj: T) -> AgentResult<String>;
    fn contains_key(&self, id: &str) -> bool;
    fn find_by<F>(&self, closure: F) -> AgentResult<Vec<String>>
    where
        F: FnMut((&String, &Self::Value)) -> Option<String>;
    fn get_all_keys(&self) -> AgentResult<Vec<String>>;
}
