use aries_vcx::aries_vcx_core::errors::error::{
    AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult,
};

use crate::errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult};

pub fn map_ariesvcx_core_result<T>(result: VcxCoreResult<T>) -> LibvcxResult<T> {
    match result {
        Ok(val) => Ok(val),
        Err(err) => Err(err.into()),
    }
}

impl From<AriesVcxCoreError> for LibvcxError {
    fn from(error: AriesVcxCoreError) -> LibvcxError {
        LibvcxError {
            kind: error.kind().into(),
            msg: error.to_string(),
        }
    }
}

impl From<AriesVcxCoreErrorKind> for LibvcxErrorKind {
    fn from(kind: AriesVcxCoreErrorKind) -> Self {
        match kind {
            AriesVcxCoreErrorKind::InvalidState => LibvcxErrorKind::InvalidState,
            AriesVcxCoreErrorKind::InvalidConfiguration => LibvcxErrorKind::InvalidConfiguration,
            AriesVcxCoreErrorKind::InvalidJson => LibvcxErrorKind::InvalidJson,
            AriesVcxCoreErrorKind::InvalidOption => LibvcxErrorKind::InvalidOption,
            AriesVcxCoreErrorKind::InvalidMessagePack => LibvcxErrorKind::InvalidMessagePack,
            AriesVcxCoreErrorKind::NotReady => LibvcxErrorKind::NotReady,
            AriesVcxCoreErrorKind::IOError => LibvcxErrorKind::IOError,
            AriesVcxCoreErrorKind::LibindyInvalidStructure => {
                LibvcxErrorKind::LibindyInvalidStructure
            }
            AriesVcxCoreErrorKind::InvalidLibindyParam => LibvcxErrorKind::InvalidLibindyParam,
            AriesVcxCoreErrorKind::ActionNotSupported => LibvcxErrorKind::ActionNotSupported,
            AriesVcxCoreErrorKind::InvalidInput => LibvcxErrorKind::InvalidInput,
            AriesVcxCoreErrorKind::UnimplementedFeature => LibvcxErrorKind::UnimplementedFeature,
            AriesVcxCoreErrorKind::CredDefAlreadyCreated => LibvcxErrorKind::CredDefAlreadyCreated,
            AriesVcxCoreErrorKind::RevDeltaNotFound => LibvcxErrorKind::RevDeltaNotFound,
            AriesVcxCoreErrorKind::RevDeltaFailedToClear => LibvcxErrorKind::RevDeltaFailedToClear,
            AriesVcxCoreErrorKind::CreateRevRegDef => LibvcxErrorKind::CreateRevRegDef,
            AriesVcxCoreErrorKind::InvalidRevocationDetails => {
                LibvcxErrorKind::InvalidRevocationDetails
            }
            AriesVcxCoreErrorKind::InvalidRevocationEntry => {
                LibvcxErrorKind::InvalidRevocationEntry
            }
            AriesVcxCoreErrorKind::InvalidRevocationTimestamp => {
                LibvcxErrorKind::InvalidRevocationTimestamp
            }
            AriesVcxCoreErrorKind::RevRegDefNotFound => LibvcxErrorKind::RevRegDefNotFound,
            AriesVcxCoreErrorKind::InvalidAttributesStructure => {
                LibvcxErrorKind::InvalidAttributesStructure
            }
            AriesVcxCoreErrorKind::InvalidProof => LibvcxErrorKind::InvalidProof,
            AriesVcxCoreErrorKind::InvalidSchema => LibvcxErrorKind::InvalidSchema,
            AriesVcxCoreErrorKind::InvalidProofCredentialData => {
                LibvcxErrorKind::InvalidProofCredentialData
            }
            AriesVcxCoreErrorKind::InvalidProofRequest => LibvcxErrorKind::InvalidProofRequest,
            AriesVcxCoreErrorKind::InvalidSchemaSeqNo => LibvcxErrorKind::InvalidSchemaSeqNo,
            AriesVcxCoreErrorKind::DuplicationSchema => LibvcxErrorKind::DuplicationSchema,
            AriesVcxCoreErrorKind::UnknownSchemaRejection => {
                LibvcxErrorKind::UnknownSchemaRejection
            }
            AriesVcxCoreErrorKind::InvalidGenesisTxnPath => LibvcxErrorKind::InvalidGenesisTxnPath,
            AriesVcxCoreErrorKind::CreatePoolConfig => LibvcxErrorKind::CreatePoolConfig,
            AriesVcxCoreErrorKind::PoolLedgerConnect => LibvcxErrorKind::PoolLedgerConnect,
            AriesVcxCoreErrorKind::InvalidLedgerResponse => LibvcxErrorKind::InvalidLedgerResponse,
            AriesVcxCoreErrorKind::NoPoolOpen => LibvcxErrorKind::NoPoolOpen,
            AriesVcxCoreErrorKind::PostMessageFailed => LibvcxErrorKind::PostMessageFailed,
            AriesVcxCoreErrorKind::WalletCreate => LibvcxErrorKind::WalletCreate,
            AriesVcxCoreErrorKind::WalletAccessFailed => LibvcxErrorKind::WalletAccessFailed,
            AriesVcxCoreErrorKind::InvalidWalletHandle => LibvcxErrorKind::InvalidWalletHandle,
            AriesVcxCoreErrorKind::DuplicationWallet => LibvcxErrorKind::DuplicationWallet,
            AriesVcxCoreErrorKind::WalletRecordNotFound => LibvcxErrorKind::WalletRecordNotFound,
            AriesVcxCoreErrorKind::DuplicationWalletRecord => {
                LibvcxErrorKind::DuplicationWalletRecord
            }
            AriesVcxCoreErrorKind::WalletNotFound => LibvcxErrorKind::WalletNotFound,
            AriesVcxCoreErrorKind::WalletAlreadyOpen => LibvcxErrorKind::WalletAlreadyOpen,
            AriesVcxCoreErrorKind::DuplicationMasterSecret => {
                LibvcxErrorKind::DuplicationMasterSecret
            }
            AriesVcxCoreErrorKind::DuplicationDid => LibvcxErrorKind::DuplicationDid,
            AriesVcxCoreErrorKind::LoggingError => LibvcxErrorKind::LoggingError,
            AriesVcxCoreErrorKind::EncodeError => LibvcxErrorKind::EncodeError,
            AriesVcxCoreErrorKind::UnknownError => LibvcxErrorKind::UnknownError,
            AriesVcxCoreErrorKind::InvalidDid => LibvcxErrorKind::InvalidDid,
            AriesVcxCoreErrorKind::InvalidVerkey => LibvcxErrorKind::InvalidVerkey,
            AriesVcxCoreErrorKind::InvalidNonce => LibvcxErrorKind::InvalidNonce,
            AriesVcxCoreErrorKind::InvalidUrl => LibvcxErrorKind::InvalidUrl,
            AriesVcxCoreErrorKind::SerializationError => LibvcxErrorKind::SerializationError,
            AriesVcxCoreErrorKind::NotBase58 => LibvcxErrorKind::NotBase58,
            AriesVcxCoreErrorKind::ParsingError => LibvcxErrorKind::ParsingError,
            AriesVcxCoreErrorKind::InvalidHttpResponse => LibvcxErrorKind::InvalidHttpResponse,
            AriesVcxCoreErrorKind::InvalidMessages => LibvcxErrorKind::InvalidMessages,
            AriesVcxCoreErrorKind::VdrToolsError(num) => LibvcxErrorKind::LibndyError(num),
            AriesVcxCoreErrorKind::NoAgentInformation => LibvcxErrorKind::NoAgentInformation,
            AriesVcxCoreErrorKind::InvalidMessageFormat => LibvcxErrorKind::InvalidMessageFormat,
            AriesVcxCoreErrorKind::LedgerItemNotFound => LibvcxErrorKind::LedgerItemNotFound,
            AriesVcxCoreErrorKind::UrsaError => LibvcxErrorKind::UrsaError,
            AriesVcxCoreErrorKind::ProofRejected => LibvcxErrorKind::ProofRejected,
        }
    }
}
