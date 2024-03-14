use indy_api_types::errors::IndyErrorKind;
use vdrtools::IndyError;

use super::error::VcxWalletError;

impl From<IndyError> for VcxWalletError {
    fn from(value: IndyError) -> Self {
        match value.kind() {
            IndyErrorKind::WalletItemNotFound => Self::RecordNotFound(value.to_string()),
            _ => Self::IndyApiError(value),
        }
    }
}
