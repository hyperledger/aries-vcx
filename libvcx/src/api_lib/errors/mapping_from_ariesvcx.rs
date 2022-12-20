use aries_vcx::errors::error::{VcxError, VcxErrorKind, VcxResult};
use crate::api_lib::errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult};

pub fn map_ariesvcx_result<T>(result: VcxResult<T>) -> LibvcxResult<T> {
    match result {
        Ok(val) => Ok(val),
        Err(err) => Err(err.into())
    }
}

impl From<VcxError> for LibvcxError {
    fn from(error: VcxError) -> LibvcxError {
        LibvcxError {
            kind: error.kind().into(),
            msg: error.to_string(),
        }
    }
}

impl From<VcxErrorKind> for LibvcxErrorKind {
    fn from(kind: VcxErrorKind) -> Self {
        match kind {
            VcxErrorKind::InvalidState => LibvcxErrorKind::InvalidState,
            VcxErrorKind::InvalidConfiguration => LibvcxErrorKind::InvalidConfiguration,
            VcxErrorKind::InvalidJson => LibvcxErrorKind::InvalidJson,
            VcxErrorKind::InvalidOption => LibvcxErrorKind::InvalidOption,
            VcxErrorKind::InvalidMessagePack => LibvcxErrorKind::InvalidMessagePack,
            VcxErrorKind::NotReady => LibvcxErrorKind::NotReady,
            VcxErrorKind::IOError => LibvcxErrorKind::IOError,
            VcxErrorKind::LibindyInvalidStructure => LibvcxErrorKind::LibindyInvalidStructure,
            VcxErrorKind::InvalidLibindyParam => LibvcxErrorKind::InvalidLibindyParam,
            VcxErrorKind::ActionNotSupported => LibvcxErrorKind::ActionNotSupported,
            VcxErrorKind::InvalidInput => LibvcxErrorKind::InvalidInput,
            VcxErrorKind::UnimplementedFeature => LibvcxErrorKind::UnimplementedFeature,
            VcxErrorKind::CredDefAlreadyCreated => LibvcxErrorKind::CredDefAlreadyCreated,
            VcxErrorKind::RevDeltaNotFound => LibvcxErrorKind::RevDeltaNotFound,
            VcxErrorKind::RevDeltaFailedToClear => LibvcxErrorKind::RevDeltaFailedToClear,
            VcxErrorKind::CreateRevRegDef => LibvcxErrorKind::CreateRevRegDef,
            VcxErrorKind::InvalidRevocationDetails => LibvcxErrorKind::InvalidRevocationDetails,
            VcxErrorKind::InvalidRevocationEntry => LibvcxErrorKind::InvalidRevocationEntry,
            VcxErrorKind::InvalidRevocationTimestamp => LibvcxErrorKind::InvalidRevocationTimestamp,
            VcxErrorKind::RevRegDefNotFound => LibvcxErrorKind::RevRegDefNotFound,
            VcxErrorKind::InvalidAttributesStructure => LibvcxErrorKind::InvalidAttributesStructure,
            VcxErrorKind::InvalidProof => LibvcxErrorKind::InvalidProof,
            VcxErrorKind::InvalidSchema => LibvcxErrorKind::InvalidSchema,
            VcxErrorKind::InvalidProofCredentialData => LibvcxErrorKind::InvalidProofCredentialData,
            VcxErrorKind::InvalidProofRequest => LibvcxErrorKind::InvalidProofRequest,
            VcxErrorKind::InvalidSchemaSeqNo => LibvcxErrorKind::InvalidSchemaSeqNo,
            VcxErrorKind::DuplicationSchema => LibvcxErrorKind::DuplicationSchema,
            VcxErrorKind::UnknownSchemaRejection => LibvcxErrorKind::UnknownSchemaRejection,
            VcxErrorKind::InvalidGenesisTxnPath => LibvcxErrorKind::InvalidGenesisTxnPath,
            VcxErrorKind::CreatePoolConfig => LibvcxErrorKind::CreatePoolConfig,
            VcxErrorKind::PoolLedgerConnect => LibvcxErrorKind::PoolLedgerConnect,
            VcxErrorKind::InvalidLedgerResponse => LibvcxErrorKind::InvalidLedgerResponse,
            VcxErrorKind::NoPoolOpen => LibvcxErrorKind::NoPoolOpen,
            VcxErrorKind::PostMessageFailed => LibvcxErrorKind::PostMessageFailed,
            VcxErrorKind::WalletCreate => LibvcxErrorKind::WalletCreate,
            VcxErrorKind::WalletAccessFailed => LibvcxErrorKind::WalletAccessFailed,
            VcxErrorKind::InvalidWalletHandle => LibvcxErrorKind::InvalidWalletHandle,
            VcxErrorKind::DuplicationWallet => LibvcxErrorKind::DuplicationWallet,
            VcxErrorKind::WalletRecordNotFound => LibvcxErrorKind::WalletRecordNotFound,
            VcxErrorKind::DuplicationWalletRecord => LibvcxErrorKind::DuplicationWalletRecord,
            VcxErrorKind::WalletNotFound => LibvcxErrorKind::WalletNotFound,
            VcxErrorKind::WalletAlreadyOpen => LibvcxErrorKind::WalletAlreadyOpen,
            VcxErrorKind::DuplicationMasterSecret => LibvcxErrorKind::DuplicationMasterSecret,
            VcxErrorKind::DuplicationDid => LibvcxErrorKind::DuplicationDid,
            VcxErrorKind::LoggingError => LibvcxErrorKind::LoggingError,
            VcxErrorKind::EncodeError => LibvcxErrorKind::EncodeError,
            VcxErrorKind::UnknownError => LibvcxErrorKind::UnknownError,
            VcxErrorKind::InvalidDid => LibvcxErrorKind::InvalidDid,
            VcxErrorKind::InvalidVerkey => LibvcxErrorKind::InvalidVerkey,
            VcxErrorKind::InvalidNonce => LibvcxErrorKind::InvalidNonce,
            VcxErrorKind::InvalidUrl => LibvcxErrorKind::InvalidUrl,
            VcxErrorKind::SerializationError => LibvcxErrorKind::SerializationError,
            VcxErrorKind::NotBase58 => LibvcxErrorKind::NotBase58,
            VcxErrorKind::ParsingError => LibvcxErrorKind::ParsingError,
            VcxErrorKind::InvalidHttpResponse => LibvcxErrorKind::InvalidHttpResponse,
            VcxErrorKind::InvalidMessages => LibvcxErrorKind::InvalidMessages,
            VcxErrorKind::Common(num) => LibvcxErrorKind::Common(num),
            VcxErrorKind::LibndyError(num) => LibvcxErrorKind::LibndyError(num),
            VcxErrorKind::UnknownLibndyError => LibvcxErrorKind::UnknownLibndyError,
            VcxErrorKind::NoAgentInformation => LibvcxErrorKind::NoAgentInformation,
            VcxErrorKind::InvalidMessageFormat => LibvcxErrorKind::InvalidMessageFormat,
        }
    }
}
