use vdrtools::types::validation::Validatable;

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};

pub(crate) mod async_fn_iterator;
pub(crate) mod author_agreement;
pub(crate) mod constants;
pub(crate) mod json;
pub(crate) mod random;
pub(crate) mod uuid;

pub fn parse_and_validate<'a, T>(s: &'a str) -> VcxResult<T>
where
    T: Validatable,
    T: serde::Deserialize<'a>,
{
    let data = serde_json::from_str::<T>(s)?;

    match data.validate() {
        Ok(_) => Ok(data),
        Err(s) => Err(AriesVcxError::from_msg(AriesVcxErrorKind::LibindyInvalidStructure, s)),
    }
}
