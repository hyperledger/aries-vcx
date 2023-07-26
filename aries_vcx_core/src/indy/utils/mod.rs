use std::sync::atomic::{AtomicUsize, Ordering};
use vdrtools::{types::validation::Validatable, CommandHandle};

use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult};

static COMMAND_HANDLE_COUNTER: AtomicUsize = AtomicUsize::new(1);

pub fn next_command_handle() -> CommandHandle {
    (COMMAND_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst) + 1) as CommandHandle
}

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
