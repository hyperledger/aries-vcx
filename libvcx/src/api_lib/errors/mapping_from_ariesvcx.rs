use aries_vcx::errors::error::{ErrorAriesVcx, ErrorKindAriesVcx, VcxResult};
use crate::api_lib::errors::error::{ErrorLibvcx, ErrorKindLibvcx, LibvcxResult};

pub fn map_ariesvcx_result<T>(result: VcxResult<T>) -> LibvcxResult<T> {
    match result {
        Ok(val) => Ok(val),
        Err(err) => Err(err.into())
    }
}

impl From<ErrorAriesVcx> for ErrorLibvcx {
    fn from(error: ErrorAriesVcx) -> ErrorLibvcx {
        ErrorLibvcx {
            kind: error.kind().into(),
            msg: error.to_string(),
        }
    }
}

impl From<ErrorKindAriesVcx> for ErrorKindLibvcx {
    fn from(kind: ErrorKindAriesVcx) -> Self {
        match kind {
            ErrorKindAriesVcx::InvalidState => ErrorKindLibvcx::InvalidState,
            ErrorKindAriesVcx::InvalidConfiguration => ErrorKindLibvcx::InvalidConfiguration,
            ErrorKindAriesVcx::InvalidJson => ErrorKindLibvcx::InvalidJson,
            ErrorKindAriesVcx::InvalidOption => ErrorKindLibvcx::InvalidOption,
            ErrorKindAriesVcx::InvalidMessagePack => ErrorKindLibvcx::InvalidMessagePack,
            ErrorKindAriesVcx::NotReady => ErrorKindLibvcx::NotReady,
            ErrorKindAriesVcx::IOError => ErrorKindLibvcx::IOError,
            ErrorKindAriesVcx::LibindyInvalidStructure => ErrorKindLibvcx::LibindyInvalidStructure,
            ErrorKindAriesVcx::InvalidLibindyParam => ErrorKindLibvcx::InvalidLibindyParam,
            ErrorKindAriesVcx::ActionNotSupported => ErrorKindLibvcx::ActionNotSupported,
            ErrorKindAriesVcx::InvalidInput => ErrorKindLibvcx::InvalidInput,
            ErrorKindAriesVcx::UnimplementedFeature => ErrorKindLibvcx::UnimplementedFeature,
            ErrorKindAriesVcx::CredDefAlreadyCreated => ErrorKindLibvcx::CredDefAlreadyCreated,
            ErrorKindAriesVcx::RevDeltaNotFound => ErrorKindLibvcx::RevDeltaNotFound,
            ErrorKindAriesVcx::RevDeltaFailedToClear => ErrorKindLibvcx::RevDeltaFailedToClear,
            ErrorKindAriesVcx::CreateRevRegDef => ErrorKindLibvcx::CreateRevRegDef,
            ErrorKindAriesVcx::InvalidRevocationDetails => ErrorKindLibvcx::InvalidRevocationDetails,
            ErrorKindAriesVcx::InvalidRevocationEntry => ErrorKindLibvcx::InvalidRevocationEntry,
            ErrorKindAriesVcx::InvalidRevocationTimestamp => ErrorKindLibvcx::InvalidRevocationTimestamp,
            ErrorKindAriesVcx::RevRegDefNotFound => ErrorKindLibvcx::RevRegDefNotFound,
            ErrorKindAriesVcx::InvalidAttributesStructure => ErrorKindLibvcx::InvalidAttributesStructure,
            ErrorKindAriesVcx::InvalidProof => ErrorKindLibvcx::InvalidProof,
            ErrorKindAriesVcx::InvalidSchema => ErrorKindLibvcx::InvalidSchema,
            ErrorKindAriesVcx::InvalidProofCredentialData => ErrorKindLibvcx::InvalidProofCredentialData,
            ErrorKindAriesVcx::InvalidProofRequest => ErrorKindLibvcx::InvalidProofRequest,
            ErrorKindAriesVcx::InvalidSchemaSeqNo => ErrorKindLibvcx::InvalidSchemaSeqNo,
            ErrorKindAriesVcx::DuplicationSchema => ErrorKindLibvcx::DuplicationSchema,
            ErrorKindAriesVcx::UnknownSchemaRejection => ErrorKindLibvcx::UnknownSchemaRejection,
            ErrorKindAriesVcx::InvalidGenesisTxnPath => ErrorKindLibvcx::InvalidGenesisTxnPath,
            ErrorKindAriesVcx::CreatePoolConfig => ErrorKindLibvcx::CreatePoolConfig,
            ErrorKindAriesVcx::PoolLedgerConnect => ErrorKindLibvcx::PoolLedgerConnect,
            ErrorKindAriesVcx::InvalidLedgerResponse => ErrorKindLibvcx::InvalidLedgerResponse,
            ErrorKindAriesVcx::NoPoolOpen => ErrorKindLibvcx::NoPoolOpen,
            ErrorKindAriesVcx::PostMessageFailed => ErrorKindLibvcx::PostMessageFailed,
            ErrorKindAriesVcx::WalletCreate => ErrorKindLibvcx::WalletCreate,
            ErrorKindAriesVcx::WalletAccessFailed => ErrorKindLibvcx::WalletAccessFailed,
            ErrorKindAriesVcx::InvalidWalletHandle => ErrorKindLibvcx::InvalidWalletHandle,
            ErrorKindAriesVcx::DuplicationWallet => ErrorKindLibvcx::DuplicationWallet,
            ErrorKindAriesVcx::WalletRecordNotFound => ErrorKindLibvcx::WalletRecordNotFound,
            ErrorKindAriesVcx::DuplicationWalletRecord => ErrorKindLibvcx::DuplicationWalletRecord,
            ErrorKindAriesVcx::WalletNotFound => ErrorKindLibvcx::WalletNotFound,
            ErrorKindAriesVcx::WalletAlreadyOpen => ErrorKindLibvcx::WalletAlreadyOpen,
            ErrorKindAriesVcx::DuplicationMasterSecret => ErrorKindLibvcx::DuplicationMasterSecret,
            ErrorKindAriesVcx::DuplicationDid => ErrorKindLibvcx::DuplicationDid,
            ErrorKindAriesVcx::LoggingError => ErrorKindLibvcx::LoggingError,
            ErrorKindAriesVcx::EncodeError => ErrorKindLibvcx::EncodeError,
            ErrorKindAriesVcx::UnknownError => ErrorKindLibvcx::UnknownError,
            ErrorKindAriesVcx::InvalidDid => ErrorKindLibvcx::InvalidDid,
            ErrorKindAriesVcx::InvalidVerkey => ErrorKindLibvcx::InvalidVerkey,
            ErrorKindAriesVcx::InvalidNonce => ErrorKindLibvcx::InvalidNonce,
            ErrorKindAriesVcx::InvalidUrl => ErrorKindLibvcx::InvalidUrl,
            ErrorKindAriesVcx::SerializationError => ErrorKindLibvcx::SerializationError,
            ErrorKindAriesVcx::NotBase58 => ErrorKindLibvcx::NotBase58,
            ErrorKindAriesVcx::ParsingError => ErrorKindLibvcx::ParsingError,
            ErrorKindAriesVcx::InvalidHttpResponse => ErrorKindLibvcx::InvalidHttpResponse,
            ErrorKindAriesVcx::InvalidMessages => ErrorKindLibvcx::InvalidMessages,
            ErrorKindAriesVcx::Common(num) => ErrorKindLibvcx::Common(num),
            ErrorKindAriesVcx::LibndyError(num) => ErrorKindLibvcx::LibndyError(num),
            ErrorKindAriesVcx::UnknownLibndyError => ErrorKindLibvcx::UnknownLibndyError,
            ErrorKindAriesVcx::NoAgentInformation => ErrorKindLibvcx::NoAgentInformation,
            ErrorKindAriesVcx::InvalidMessageFormat => ErrorKindLibvcx::InvalidMessageFormat,
        }
    }
}
