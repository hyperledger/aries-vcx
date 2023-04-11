use std::{env, path::PathBuf};

use vdrtools::types::validation::Validatable;

use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult};

pub mod async_fn_iterator;
pub(crate) mod author_agreement;
pub(crate) mod constants;
pub(crate) mod json;
pub(crate) mod mockdata;
pub(crate) mod random;
pub(crate) mod uuid;

pub fn parse_and_validate<'a, T>(s: &'a str) -> VcxCoreResult<T>
where
    T: Validatable,
    T: serde::Deserialize<'a>,
{
    let data = serde_json::from_str::<T>(s)?;

    match data.validate() {
        Ok(_) => Ok(data),
        Err(s) => Err(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::LibindyInvalidStructure,
            s,
        )),
    }
}

pub fn get_temp_dir_path(filename: &str) -> PathBuf {
    let mut path = env::temp_dir();
    path.push(filename);
    path
}
