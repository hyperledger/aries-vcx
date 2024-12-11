use crate::errors::error::VcxLedgerError;

pub mod arc;
pub mod base_ledger;
pub mod common;

#[cfg(feature = "cheqd")]
pub mod cheqd;
pub mod indy;
pub mod indy_vdr_ledger;
pub mod multi_ledger;
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
