use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind};

pub mod base_ledger;
pub mod common;

#[cfg(any(feature = "modular_libs", feature = "vdr_proxy_ledger"))]
pub mod indy_vdr_ledger;
#[cfg(any(feature = "modular_libs", feature = "vdr_proxy_ledger"))]
pub mod request_signer;
#[cfg(any(feature = "modular_libs", feature = "vdr_proxy_ledger"))]
pub mod request_submitter;
#[cfg(any(feature = "modular_libs", feature = "vdr_proxy_ledger"))]
pub mod response_cacher;

#[cfg(feature = "vdrtools")]
pub mod indy;
#[cfg(feature = "vdrtools")]
pub mod indy_ledger;

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
