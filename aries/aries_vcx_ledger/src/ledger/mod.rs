use crate::errors::error::VcxLedgerError;

pub mod base_ledger;
pub mod common;

pub mod indy;
pub mod indy_vdr_ledger;
mod type_conversion;

pub mod request_submitter;
pub mod response_cacher;

fn map_error_not_found_to_none<T, E>(res: Result<T, E>) -> Result<Option<T>, VcxLedgerError>
where
    E: Into<VcxLedgerError>,
{
    match res {
        Ok(response) => Ok(Some(response)),
        Err(err) => {
            let err_converted = Into::<VcxLedgerError>::into(err);
            match err_converted {
                VcxLedgerError::LedgerItemNotFound => Ok(None),
                _ => Err(err_converted),
            }
        }
    }
}
