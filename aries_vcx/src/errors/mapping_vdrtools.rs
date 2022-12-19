use vdrtools::types;
use vdrtools::types::errors::IndyErrorKind;

use crate::error::{VcxError, VcxErrorKind};

impl From<IndyErrorKind> for VcxErrorKind {
    fn from(indy: IndyErrorKind) -> Self {
        use types::errors::IndyErrorKind::*;

        match indy {
            // 100..=111, 115..=129
            InvalidParam(_) => VcxErrorKind::InvalidLibindyParam,

            // 112
            // InvalidState => VcxErrorKind::LibndyError(err_code),

            // 113
            InvalidStructure => VcxErrorKind::LibindyInvalidStructure,

            // 114
            IOError => VcxErrorKind::IOError,

            // 200
            InvalidWalletHandle => VcxErrorKind::InvalidWalletHandle,

            // 203
            WalletAlreadyExists => VcxErrorKind::DuplicationWallet,

            // 204
            WalletNotFound => VcxErrorKind::WalletNotFound,

            // 206
            WalletAlreadyOpened => VcxErrorKind::WalletAlreadyOpen,

            // 212
            WalletItemNotFound => VcxErrorKind::WalletRecordNotFound,

            // 213
            WalletItemAlreadyExists => VcxErrorKind::DuplicationWalletRecord,

            // 306
            PoolConfigAlreadyExists => VcxErrorKind::CreatePoolConfig,

            // 404
            MasterSecretDuplicateName => VcxErrorKind::DuplicationMasterSecret,

            // 407
            CredDefAlreadyExists => VcxErrorKind::CredDefAlreadyCreated,

            // 600
            DIDAlreadyExists => VcxErrorKind::DuplicationDid,

            // 702
            PaymentInsufficientFunds => VcxErrorKind::InsufficientTokenAmount,

            InvalidState |
            ProofRejected |
            RevocationRegistryFull |
            LedgerItemNotFound |
            InvalidPoolHandle |
            UnknownWalletStorageType |
            InvalidUserRevocId |
            CredentialRevoked |
            NoConsensus |
            InvalidTransaction |
            PoolNotCreated |
            PoolTerminated |
            PoolTimeout |
            PoolIncompatibleProtocolVersion |
            UnknownCrypto |
            WalletStorageTypeAlreadyRegistered |
            WalletAccessFailed |
            WalletEncodingError |
            WalletStorageError |
            WalletEncryptionError |
            WalletQueryError |
            UnknownPaymentMethodType |
            IncompatiblePaymentMethods |
            PaymentSourceDoesNotExist |
            PaymentOperationNotSupported |
            PaymentExtraFunds |
            TransactionNotAllowed |
            QueryAccountDoesNotExist |
            InvalidVDRHandle |
            InvalidVDRNamespace |
            IncompatibleLedger => {
                let err_code = types::ErrorCode::from(indy) as u32;
                VcxErrorKind::LibndyError(err_code)
            }
        }
    }
}

impl From<types::errors::IndyError> for VcxError {
    fn from(indy: types::errors::IndyError) -> Self {
        let vcx_kind: VcxErrorKind = indy.kind().into();
        VcxError::from_msg(vcx_kind, indy.to_string())
    }
}
