use aries_vcx::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};

use crate::errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult};

pub fn map_ariesvcx_result<T>(result: VcxResult<T>) -> LibvcxResult<T> {
    match result {
        Ok(val) => Ok(val),
        Err(err) => Err(err.into()),
    }
}

impl From<AriesVcxError> for LibvcxError {
    fn from(error: AriesVcxError) -> LibvcxError {
        LibvcxError {
            kind: error.kind().into(),
            msg: error.to_string(),
        }
    }
}

impl From<AriesVcxErrorKind> for LibvcxErrorKind {
    fn from(kind: AriesVcxErrorKind) -> Self {
        match kind {
            AriesVcxErrorKind::InvalidState => LibvcxErrorKind::InvalidState,
            AriesVcxErrorKind::InvalidConfiguration => LibvcxErrorKind::InvalidConfiguration,
            AriesVcxErrorKind::InvalidJson => LibvcxErrorKind::InvalidJson,
            AriesVcxErrorKind::InvalidOption => LibvcxErrorKind::InvalidOption,
            AriesVcxErrorKind::InvalidMessagePack => LibvcxErrorKind::InvalidMessagePack,
            AriesVcxErrorKind::NotReady => LibvcxErrorKind::NotReady,
            AriesVcxErrorKind::IOError => LibvcxErrorKind::IOError,
            AriesVcxErrorKind::LibindyInvalidStructure => LibvcxErrorKind::LibindyInvalidStructure,
            AriesVcxErrorKind::InvalidLibindyParam => LibvcxErrorKind::InvalidLibindyParam,
            AriesVcxErrorKind::ActionNotSupported => LibvcxErrorKind::ActionNotSupported,
            AriesVcxErrorKind::InvalidInput => LibvcxErrorKind::InvalidInput,
            AriesVcxErrorKind::UnimplementedFeature => LibvcxErrorKind::UnimplementedFeature,
            AriesVcxErrorKind::CredDefAlreadyCreated => LibvcxErrorKind::CredDefAlreadyCreated,
            AriesVcxErrorKind::RevDeltaNotFound => LibvcxErrorKind::RevDeltaNotFound,
            AriesVcxErrorKind::RevDeltaFailedToClear => LibvcxErrorKind::RevDeltaFailedToClear,
            AriesVcxErrorKind::CreateRevRegDef => LibvcxErrorKind::CreateRevRegDef,
            AriesVcxErrorKind::InvalidRevocationDetails => LibvcxErrorKind::InvalidRevocationDetails,
            AriesVcxErrorKind::InvalidRevocationEntry => LibvcxErrorKind::InvalidRevocationEntry,
            AriesVcxErrorKind::InvalidRevocationTimestamp => LibvcxErrorKind::InvalidRevocationTimestamp,
            AriesVcxErrorKind::RevRegDefNotFound => LibvcxErrorKind::RevRegDefNotFound,
            AriesVcxErrorKind::InvalidAttributesStructure => LibvcxErrorKind::InvalidAttributesStructure,
            AriesVcxErrorKind::InvalidProof => LibvcxErrorKind::InvalidProof,
            AriesVcxErrorKind::InvalidSchema => LibvcxErrorKind::InvalidSchema,
            AriesVcxErrorKind::InvalidProofCredentialData => LibvcxErrorKind::InvalidProofCredentialData,
            AriesVcxErrorKind::InvalidProofRequest => LibvcxErrorKind::InvalidProofRequest,
            AriesVcxErrorKind::InvalidSchemaSeqNo => LibvcxErrorKind::InvalidSchemaSeqNo,
            AriesVcxErrorKind::DuplicationSchema => LibvcxErrorKind::DuplicationSchema,
            AriesVcxErrorKind::UnknownSchemaRejection => LibvcxErrorKind::UnknownSchemaRejection,
            AriesVcxErrorKind::InvalidGenesisTxnPath => LibvcxErrorKind::InvalidGenesisTxnPath,
            AriesVcxErrorKind::CreatePoolConfig => LibvcxErrorKind::CreatePoolConfig,
            AriesVcxErrorKind::PoolLedgerConnect => LibvcxErrorKind::PoolLedgerConnect,
            AriesVcxErrorKind::InvalidLedgerResponse => LibvcxErrorKind::InvalidLedgerResponse,
            AriesVcxErrorKind::NoPoolOpen => LibvcxErrorKind::NoPoolOpen,
            AriesVcxErrorKind::PostMessageFailed => LibvcxErrorKind::PostMessageFailed,
            AriesVcxErrorKind::WalletCreate => LibvcxErrorKind::WalletCreate,
            AriesVcxErrorKind::WalletAccessFailed => LibvcxErrorKind::WalletAccessFailed,
            AriesVcxErrorKind::InvalidWalletHandle => LibvcxErrorKind::InvalidWalletHandle,
            AriesVcxErrorKind::DuplicationWallet => LibvcxErrorKind::DuplicationWallet,
            AriesVcxErrorKind::WalletRecordNotFound => LibvcxErrorKind::WalletRecordNotFound,
            AriesVcxErrorKind::DuplicationWalletRecord => LibvcxErrorKind::DuplicationWalletRecord,
            AriesVcxErrorKind::WalletNotFound => LibvcxErrorKind::WalletNotFound,
            AriesVcxErrorKind::WalletAlreadyOpen => LibvcxErrorKind::WalletAlreadyOpen,
            AriesVcxErrorKind::DuplicationMasterSecret => LibvcxErrorKind::DuplicationMasterSecret,
            AriesVcxErrorKind::DuplicationDid => LibvcxErrorKind::DuplicationDid,
            AriesVcxErrorKind::LoggingError => LibvcxErrorKind::LoggingError,
            AriesVcxErrorKind::EncodeError => LibvcxErrorKind::EncodeError,
            AriesVcxErrorKind::UnknownError => LibvcxErrorKind::UnknownError,
            AriesVcxErrorKind::InvalidDid => LibvcxErrorKind::InvalidDid,
            AriesVcxErrorKind::InvalidVerkey => LibvcxErrorKind::InvalidVerkey,
            AriesVcxErrorKind::InvalidNonce => LibvcxErrorKind::InvalidNonce,
            AriesVcxErrorKind::InvalidUrl => LibvcxErrorKind::InvalidUrl,
            AriesVcxErrorKind::SerializationError => LibvcxErrorKind::SerializationError,
            AriesVcxErrorKind::NotBase58 => LibvcxErrorKind::NotBase58,
            AriesVcxErrorKind::ParsingError => LibvcxErrorKind::ParsingError,
            AriesVcxErrorKind::InvalidHttpResponse => LibvcxErrorKind::InvalidHttpResponse,
            AriesVcxErrorKind::InvalidMessages => LibvcxErrorKind::InvalidMessages,
            AriesVcxErrorKind::VdrToolsError(num) => LibvcxErrorKind::LibndyError(num),
            AriesVcxErrorKind::NoAgentInformation => LibvcxErrorKind::NoAgentInformation,
            AriesVcxErrorKind::InvalidMessageFormat => LibvcxErrorKind::InvalidMessageFormat,
            AriesVcxErrorKind::LedgerItemNotFound => LibvcxErrorKind::LedgerItemNotFound,
            AriesVcxErrorKind::UrsaError => LibvcxErrorKind::UrsaError,
            AriesVcxErrorKind::ProofRejected => LibvcxErrorKind::ProofRejected,
        }
    }
}
