use vdrtools::types;
use vdrtools::types::errors::IndyErrorKind;

use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind};

impl From<IndyErrorKind> for AriesVcxCoreErrorKind {
    fn from(indy: IndyErrorKind) -> Self {
        use types::errors::IndyErrorKind::*;

        match indy {
            InvalidParam(_) => AriesVcxCoreErrorKind::InvalidLibindyParam,
            InvalidStructure => AriesVcxCoreErrorKind::LibindyInvalidStructure,
            IOError => AriesVcxCoreErrorKind::IOError,
            InvalidWalletHandle => AriesVcxCoreErrorKind::InvalidWalletHandle,
            WalletAlreadyExists => AriesVcxCoreErrorKind::DuplicationWallet,
            WalletNotFound => AriesVcxCoreErrorKind::WalletNotFound,
            WalletAlreadyOpened => AriesVcxCoreErrorKind::WalletAlreadyOpen,
            WalletItemNotFound => AriesVcxCoreErrorKind::WalletRecordNotFound,
            WalletItemAlreadyExists => AriesVcxCoreErrorKind::DuplicationWalletRecord,
            PoolConfigAlreadyExists => AriesVcxCoreErrorKind::CreatePoolConfig,
            MasterSecretDuplicateName => AriesVcxCoreErrorKind::DuplicationMasterSecret,
            CredDefAlreadyExists => AriesVcxCoreErrorKind::CredDefAlreadyCreated,
            DIDAlreadyExists => AriesVcxCoreErrorKind::DuplicationDid,
            InvalidState => AriesVcxCoreErrorKind::InvalidState,
            NoConsensus => AriesVcxCoreErrorKind::InvalidLedgerResponse,
            InvalidTransaction => AriesVcxCoreErrorKind::InvalidLedgerResponse,
            LedgerItemNotFound => AriesVcxCoreErrorKind::LedgerItemNotFound,
            TransactionNotAllowed => AriesVcxCoreErrorKind::InvalidLedgerResponse,
            PoolTimeout => AriesVcxCoreErrorKind::InvalidLedgerResponse,
            PoolIncompatibleProtocolVersion => AriesVcxCoreErrorKind::InvalidConfiguration,
            UnknownWalletStorageType => AriesVcxCoreErrorKind::InvalidConfiguration,
            WalletStorageTypeAlreadyRegistered => AriesVcxCoreErrorKind::InvalidConfiguration,
            WalletAccessFailed => AriesVcxCoreErrorKind::WalletAccessFailed,
            ProofRejected => AriesVcxCoreErrorKind::ProofRejected,
            _ => {
                let err_code = types::ErrorCode::from(indy) as u32;
                AriesVcxCoreErrorKind::VdrToolsError(err_code)
            }
        }
    }
}

impl From<types::errors::IndyError> for AriesVcxCoreError {
    fn from(indy: types::errors::IndyError) -> Self {
        let vcx_kind: AriesVcxCoreErrorKind = indy.kind().into();
        AriesVcxCoreError::from_msg(vcx_kind, indy.to_string())
    }
}
