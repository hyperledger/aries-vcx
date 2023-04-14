use std::{env, path::PathBuf};

pub mod async_fn_iterator;
pub(crate) mod author_agreement;
pub(crate) mod constants;
pub(crate) mod json;
pub(crate) mod mockdata;
pub(crate) mod random;

pub fn get_temp_dir_path(filename: &str) -> PathBuf {
    let mut path = env::temp_dir();
    path.push(filename);
    path
}

pub fn get_temp_dir_path(filename: &str) -> PathBuf {
    let mut path = env::temp_dir();
    path.push(filename);
    path
}
