use aries_vcx_wallet::errors::error::VcxWalletError;

use super::error::{AriesVcxCoreError, AriesVcxCoreErrorKind};

impl From<VcxWalletError> for AriesVcxCoreError {
    fn from(value: VcxWalletError) -> Self {
        match value {
            VcxWalletError::DuplicateRecord(_) => Self::from_msg(
                AriesVcxCoreErrorKind::DuplicationWalletRecord,
                value.to_string(),
            ),
            VcxWalletError::RecordNotFound { .. } => Self::from_msg(
                AriesVcxCoreErrorKind::WalletRecordNotFound,
                value.to_string(),
            ),
            VcxWalletError::UnknownRecordCategory(_) => {
                Self::from_msg(AriesVcxCoreErrorKind::InvalidInput, value.to_string())
            }
            #[cfg(feature = "vdrtools_wallet")]
            VcxWalletError::IndyApiError(indy_error) => indy_error.into(),
            VcxWalletError::InvalidInput(_) => {
                Self::from_msg(AriesVcxCoreErrorKind::InvalidInput, value.to_string())
            }
            VcxWalletError::NoRecipientKeyFound => Self::from_msg(
                AriesVcxCoreErrorKind::WalletRecordNotFound,
                value.to_string(),
            ),
            VcxWalletError::InvalidJson(_) => {
                Self::from_msg(AriesVcxCoreErrorKind::InvalidJson, value.to_string())
            }
            VcxWalletError::PublicKeyError(_) => {
                Self::from_msg(AriesVcxCoreErrorKind::InvalidInput, value.to_string())
            }
            VcxWalletError::Unimplemented(_) => Self::from_msg(
                AriesVcxCoreErrorKind::UnimplementedFeature,
                value.to_string(),
            ),
            VcxWalletError::Unknown(_) => {
                Self::from_msg(AriesVcxCoreErrorKind::UnknownError, value.to_string())
            }
            VcxWalletError::WalletCreate(_) => {
                Self::from_msg(AriesVcxCoreErrorKind::WalletCreate, value.to_string())
            }
            VcxWalletError::NotUtf8(_) => {
                Self::from_msg(AriesVcxCoreErrorKind::ParsingError, value.to_string())
            }
            VcxWalletError::NotBase58(_) => {
                Self::from_msg(AriesVcxCoreErrorKind::NotBase58, value.to_string())
            }
            VcxWalletError::NotBase64(_) => {
                Self::from_msg(AriesVcxCoreErrorKind::ParsingError, value.to_string())
            }
        }
    }
}
