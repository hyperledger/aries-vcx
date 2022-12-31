use vdrtools::types;
use vdrtools::types::errors::IndyErrorKind;

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind};

impl From<IndyErrorKind> for AriesVcxErrorKind {
    fn from(indy: IndyErrorKind) -> Self {
        use types::errors::IndyErrorKind::*;

        match indy {
            // 100..=111, 115..=129
            InvalidParam(_) => AriesVcxErrorKind::InvalidLibindyParam,

            // 113
            InvalidStructure => AriesVcxErrorKind::LibindyInvalidStructure,

            // 114
            IOError => AriesVcxErrorKind::IOError,

            // 200
            InvalidWalletHandle => AriesVcxErrorKind::InvalidWalletHandle,

            // 203
            WalletAlreadyExists => AriesVcxErrorKind::DuplicationWallet,

            // 204
            WalletNotFound => AriesVcxErrorKind::WalletNotFound,

            // 206
            WalletAlreadyOpened => AriesVcxErrorKind::WalletAlreadyOpen,

            // 212
            WalletItemNotFound => AriesVcxErrorKind::WalletRecordNotFound,

            // 213
            WalletItemAlreadyExists => AriesVcxErrorKind::DuplicationWalletRecord,

            // 306
            PoolConfigAlreadyExists => AriesVcxErrorKind::CreatePoolConfig,

            // 404
            MasterSecretDuplicateName => AriesVcxErrorKind::DuplicationMasterSecret,

            // 405
            ProofRejected => AriesVcxErrorKind::ProofRejected,

            // 407
            CredDefAlreadyExists => AriesVcxErrorKind::CredDefAlreadyCreated,

            // 600
            DIDAlreadyExists => AriesVcxErrorKind::DuplicationDid,

            // 702
            PaymentInsufficientFunds
            | InvalidState
            | RevocationRegistryFull
            | LedgerItemNotFound
            | InvalidPoolHandle
            | UnknownWalletStorageType
            | InvalidUserRevocId
            | CredentialRevoked
            | NoConsensus
            | InvalidTransaction
            | PoolNotCreated
            | PoolTerminated
            | PoolTimeout
            | PoolIncompatibleProtocolVersion
            | UnknownCrypto
            | WalletStorageTypeAlreadyRegistered
            | WalletAccessFailed
            | WalletEncodingError
            | WalletStorageError
            | WalletEncryptionError
            | WalletQueryError
            | UnknownPaymentMethodType
            | IncompatiblePaymentMethods
            | PaymentSourceDoesNotExist
            | PaymentOperationNotSupported
            | PaymentExtraFunds
            | TransactionNotAllowed
            | QueryAccountDoesNotExist
            | InvalidVDRHandle
            | InvalidVDRNamespace
            | IncompatibleLedger => {
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
