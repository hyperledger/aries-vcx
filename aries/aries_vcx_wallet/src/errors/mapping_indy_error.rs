use indy_api_types::errors::IndyErrorKind;
use vdrtools::IndyError;

use super::error::VcxWalletError;

impl From<IndyError> for VcxWalletError {
    fn from(value: IndyError) -> Self {
        match value.kind() {
            IndyErrorKind::WalletItemNotFound => {
                Self::record_not_found_from_str(&value.to_string())
            }
            _ => Self::IndyApiError(value),
        }
    }
}
