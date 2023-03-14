use vdrtools::{types, types::errors::IndyErrorKind};

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind};

impl From<IndyErrorKind> for AriesVcxErrorKind {
    fn from(indy: IndyErrorKind) -> Self {
        use types::errors::IndyErrorKind::*;

        match indy {
            InvalidParam(_) => AriesVcxErrorKind::InvalidLibindyParam,
            InvalidStructure => AriesVcxErrorKind::LibindyInvalidStructure,
            IOError => AriesVcxErrorKind::IOError,
            InvalidWalletHandle => AriesVcxErrorKind::InvalidWalletHandle,
            WalletAlreadyExists => AriesVcxErrorKind::DuplicationWallet,
            WalletNotFound => AriesVcxErrorKind::WalletNotFound,
            WalletAlreadyOpened => AriesVcxErrorKind::WalletAlreadyOpen,
            WalletItemNotFound => AriesVcxErrorKind::WalletRecordNotFound,
            WalletItemAlreadyExists => AriesVcxErrorKind::DuplicationWalletRecord,
            PoolConfigAlreadyExists => AriesVcxErrorKind::CreatePoolConfig,
            MasterSecretDuplicateName => AriesVcxErrorKind::DuplicationMasterSecret,
            CredDefAlreadyExists => AriesVcxErrorKind::CredDefAlreadyCreated,
            DIDAlreadyExists => AriesVcxErrorKind::DuplicationDid,
            InvalidState => AriesVcxErrorKind::InvalidState,
            NoConsensus => AriesVcxErrorKind::InvalidLedgerResponse,
            InvalidTransaction => AriesVcxErrorKind::InvalidLedgerResponse,
            LedgerItemNotFound => AriesVcxErrorKind::LedgerItemNotFound,
            TransactionNotAllowed => AriesVcxErrorKind::InvalidLedgerResponse,
            PoolTimeout => AriesVcxErrorKind::InvalidLedgerResponse,
            PoolIncompatibleProtocolVersion => AriesVcxErrorKind::InvalidConfiguration,
            UnknownWalletStorageType => AriesVcxErrorKind::InvalidConfiguration,
            WalletStorageTypeAlreadyRegistered => AriesVcxErrorKind::InvalidConfiguration,
            WalletAccessFailed => AriesVcxErrorKind::WalletAccessFailed,
            ProofRejected => AriesVcxErrorKind::ProofRejected,
            _ => {
                let err_code = types::ErrorCode::from(indy) as u32;
                AriesVcxErrorKind::VdrToolsError(err_code)
            }
        }
    }
}

impl From<types::errors::IndyError> for AriesVcxError {
    fn from(indy: types::errors::IndyError) -> Self {
        let vcx_kind: AriesVcxErrorKind = indy.kind().into();
        AriesVcxError::from_msg(vcx_kind, indy.to_string())
    }
}
