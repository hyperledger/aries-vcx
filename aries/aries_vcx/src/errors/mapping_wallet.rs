use aries_vcx_wallet::errors::error::VcxWalletError;

use super::error::{AriesVcxError, AriesVcxErrorKind};

impl From<VcxWalletError> for AriesVcxError {
    fn from(value: VcxWalletError) -> Self {
        match value {
            VcxWalletError::DuplicateRecord(_) => Self::from_msg(
                AriesVcxErrorKind::DuplicationWalletRecord,
                value.to_string(),
            ),
            VcxWalletError::RecordNotFound { .. } => {
                Self::from_msg(AriesVcxErrorKind::WalletRecordNotFound, value.to_string())
            }
            VcxWalletError::UnknownRecordCategory(_) => {
                Self::from_msg(AriesVcxErrorKind::InvalidInput, value.to_string())
            }
            VcxWalletError::InvalidInput(_) => {
                Self::from_msg(AriesVcxErrorKind::InvalidInput, value.to_string())
            }
            VcxWalletError::NoRecipientKeyFound => {
                Self::from_msg(AriesVcxErrorKind::WalletRecordNotFound, value.to_string())
            }
            VcxWalletError::InvalidJson(_) => {
                Self::from_msg(AriesVcxErrorKind::InvalidJson, value.to_string())
            }
            VcxWalletError::PublicKeyError(_) => {
                Self::from_msg(AriesVcxErrorKind::InvalidInput, value.to_string())
            }
            VcxWalletError::Unimplemented(_) => {
                Self::from_msg(AriesVcxErrorKind::UnimplementedFeature, value.to_string())
            }
            VcxWalletError::Unknown(_) => {
                Self::from_msg(AriesVcxErrorKind::UnknownError, value.to_string())
            }
            VcxWalletError::WalletCreate(_) => {
                Self::from_msg(AriesVcxErrorKind::WalletCreate, value.to_string())
            }
            VcxWalletError::NotUtf8(_) => {
                Self::from_msg(AriesVcxErrorKind::ParsingError, value.to_string())
            }
            VcxWalletError::NotBase58(_) => {
                Self::from_msg(AriesVcxErrorKind::NotBase58, value.to_string())
            }
            VcxWalletError::NotBase64(_) => {
                Self::from_msg(AriesVcxErrorKind::ParsingError, value.to_string())
            }
            // can be
            #[allow(unreachable_patterns)]
            _ => Self::from_msg(AriesVcxErrorKind::UnknownError, value.to_string()),
        }
    }
}
