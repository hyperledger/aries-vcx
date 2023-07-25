use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind};

pub mod base_ledger;
pub mod common;

pub mod indy;
pub mod indy_vdr_ledger;
pub mod request_signer;
pub mod request_submitter;
pub mod response_cacher;

fn map_error_not_found_to_none<T, E>(res: Result<T, E>) -> Result<Option<T>, AriesVcxCoreError>
where
    E: Into<AriesVcxCoreError>,
{
    match res {
        Ok(response) => Ok(Some(response)),
        Err(err) => {
            let err_converted = Into::<AriesVcxCoreError>::into(err);
            match err_converted.kind() {
                AriesVcxCoreErrorKind::LedgerItemNotFound => Ok(None),
                _ => Err(err_converted),
            }
        }
    }
}
