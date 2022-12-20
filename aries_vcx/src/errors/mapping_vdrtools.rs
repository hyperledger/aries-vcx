use vdrtools::types;
use vdrtools::types::errors::IndyErrorKind;

use crate::errors::error::{ErrorAriesVcx, ErrorKindAriesVcx};

impl From<IndyErrorKind> for ErrorKindAriesVcx {
    fn from(indy: IndyErrorKind) -> Self {
        use types::errors::IndyErrorKind::*;

        match indy {
            // 100..=111, 115..=129
            InvalidParam(_) => ErrorKindAriesVcx::InvalidLibindyParam,

            // 113
            InvalidStructure => ErrorKindAriesVcx::LibindyInvalidStructure,

            // 114
            IOError => ErrorKindAriesVcx::IOError,

            // 200
            InvalidWalletHandle => ErrorKindAriesVcx::InvalidWalletHandle,

            // 203
            WalletAlreadyExists => ErrorKindAriesVcx::DuplicationWallet,

            // 204
            WalletNotFound => ErrorKindAriesVcx::WalletNotFound,

            // 206
            WalletAlreadyOpened => ErrorKindAriesVcx::WalletAlreadyOpen,

            // 212
            WalletItemNotFound => ErrorKindAriesVcx::WalletRecordNotFound,

            // 213
            WalletItemAlreadyExists => ErrorKindAriesVcx::DuplicationWalletRecord,

            // 306
            PoolConfigAlreadyExists => ErrorKindAriesVcx::CreatePoolConfig,

            // 404
            MasterSecretDuplicateName => ErrorKindAriesVcx::DuplicationMasterSecret,

            // 407
            CredDefAlreadyExists => ErrorKindAriesVcx::CredDefAlreadyCreated,

            // 600
            DIDAlreadyExists => ErrorKindAriesVcx::DuplicationDid,

            // 702
            PaymentInsufficientFunds |
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
                ErrorKindAriesVcx::LibndyError(err_code)
            }
        }
    }
}

impl From<types::errors::IndyError> for ErrorAriesVcx {
    fn from(indy: types::errors::IndyError) -> Self {
        let vcx_kind: ErrorKindAriesVcx = indy.kind().into();
        ErrorAriesVcx::from_msg(vcx_kind, indy.to_string())
    }
}
