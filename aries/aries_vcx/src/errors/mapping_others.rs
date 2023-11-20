use std::sync::PoisonError;

use aries_vcx_core::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind};
use shared::errors::http_error::HttpError;

use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind},
    protocols::revocation_notification::sender::state_machine::SenderConfigBuilderError,
};

impl From<SenderConfigBuilderError> for AriesVcxError {
    fn from(err: SenderConfigBuilderError) -> AriesVcxError {
        let vcx_error_kind = AriesVcxErrorKind::InvalidConfiguration;
        AriesVcxError::from_msg(vcx_error_kind, err.to_string())
    }
}

impl From<serde_json::Error> for AriesVcxError {
    fn from(_err: serde_json::Error) -> Self {
        AriesVcxError::from_msg(AriesVcxErrorKind::InvalidJson, "Invalid json".to_string())
    }
}

impl<T> From<PoisonError<T>> for AriesVcxError {
    fn from(err: PoisonError<T>) -> Self {
        AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, err.to_string())
    }
}

impl From<HttpError> for AriesVcxError {
    fn from(err: HttpError) -> Self {
        AriesVcxError::from_msg(AriesVcxErrorKind::PostMessageFailed, err.to_string())
    }
}

impl From<did_parser::ParseError> for AriesVcxError {
    fn from(err: did_parser::ParseError) -> Self {
        AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, err.to_string())
    }
}

impl From<did_doc::error::DidDocumentBuilderError> for AriesVcxError {
    fn from(err: did_doc::error::DidDocumentBuilderError) -> Self {
        AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, err.to_string())
    }
}

impl From<did_doc_sov::error::DidDocumentSovError> for AriesVcxError {
    fn from(err: did_doc_sov::error::DidDocumentSovError) -> Self {
        AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, err.to_string())
    }
}

impl From<did_peer::error::DidPeerError> for AriesVcxError {
    fn from(err: did_peer::error::DidPeerError) -> Self {
        AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, err.to_string())
    }
}

impl From<did_resolver::error::GenericError> for AriesVcxError {
    fn from(err: did_resolver::error::GenericError) -> Self {
        AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, err.to_string())
    }
}

impl From<public_key::PublicKeyError> for AriesVcxError {
    fn from(err: public_key::PublicKeyError) -> Self {
        AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, err.to_string())
    }
}

impl From<did_key::error::DidKeyError> for AriesVcxError {
    fn from(err: did_key::error::DidKeyError) -> Self {
        AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, err.to_string())
    }
}

// TODO
impl From<AriesVcxCoreError> for AriesVcxError {
    fn from(err: AriesVcxCoreError) -> Self {
        let kind = match err.kind() {
            AriesVcxCoreErrorKind::InvalidState => AriesVcxErrorKind::InvalidState,
            AriesVcxCoreErrorKind::InvalidConfiguration => AriesVcxErrorKind::InvalidConfiguration,
            AriesVcxCoreErrorKind::InvalidJson => AriesVcxErrorKind::InvalidJson,
            AriesVcxCoreErrorKind::InvalidOption => AriesVcxErrorKind::InvalidOption,
            AriesVcxCoreErrorKind::InvalidMessagePack => AriesVcxErrorKind::InvalidMessagePack,
            AriesVcxCoreErrorKind::NotReady => AriesVcxErrorKind::NotReady,
            AriesVcxCoreErrorKind::IOError => AriesVcxErrorKind::IOError,
            AriesVcxCoreErrorKind::LibindyInvalidStructure => {
                AriesVcxErrorKind::LibindyInvalidStructure
            }
            AriesVcxCoreErrorKind::InvalidLibindyParam => AriesVcxErrorKind::InvalidLibindyParam,
            AriesVcxCoreErrorKind::ActionNotSupported => AriesVcxErrorKind::ActionNotSupported,
            AriesVcxCoreErrorKind::InvalidInput => AriesVcxErrorKind::InvalidInput,
            AriesVcxCoreErrorKind::UnimplementedFeature => AriesVcxErrorKind::UnimplementedFeature,
            AriesVcxCoreErrorKind::CredDefAlreadyCreated => {
                AriesVcxErrorKind::CredDefAlreadyCreated
            }
            AriesVcxCoreErrorKind::RevDeltaNotFound => AriesVcxErrorKind::RevDeltaNotFound,
            AriesVcxCoreErrorKind::RevDeltaFailedToClear => {
                AriesVcxErrorKind::RevDeltaFailedToClear
            }
            AriesVcxCoreErrorKind::CreateRevRegDef => AriesVcxErrorKind::CreateRevRegDef,
            AriesVcxCoreErrorKind::InvalidRevocationDetails => {
                AriesVcxErrorKind::InvalidRevocationDetails
            }
            AriesVcxCoreErrorKind::InvalidRevocationEntry => {
                AriesVcxErrorKind::InvalidRevocationEntry
            }
            AriesVcxCoreErrorKind::InvalidRevocationTimestamp => {
                AriesVcxErrorKind::InvalidRevocationTimestamp
            }
            AriesVcxCoreErrorKind::RevRegDefNotFound => AriesVcxErrorKind::RevRegDefNotFound,
            AriesVcxCoreErrorKind::InvalidAttributesStructure => {
                AriesVcxErrorKind::InvalidAttributesStructure
            }
            AriesVcxCoreErrorKind::InvalidProof => AriesVcxErrorKind::InvalidProof,
            AriesVcxCoreErrorKind::InvalidSchema => AriesVcxErrorKind::InvalidSchema,
            AriesVcxCoreErrorKind::InvalidProofCredentialData => {
                AriesVcxErrorKind::InvalidProofCredentialData
            }
            AriesVcxCoreErrorKind::InvalidProofRequest => AriesVcxErrorKind::InvalidProofRequest,
            AriesVcxCoreErrorKind::ProofRejected => AriesVcxErrorKind::ProofRejected,
            AriesVcxCoreErrorKind::InvalidSchemaSeqNo => AriesVcxErrorKind::InvalidSchemaSeqNo,
            AriesVcxCoreErrorKind::DuplicationSchema => AriesVcxErrorKind::DuplicationSchema,
            AriesVcxCoreErrorKind::UnknownSchemaRejection => {
                AriesVcxErrorKind::UnknownSchemaRejection
            }
            AriesVcxCoreErrorKind::InvalidGenesisTxnPath => {
                AriesVcxErrorKind::InvalidGenesisTxnPath
            }
            AriesVcxCoreErrorKind::CreatePoolConfig => AriesVcxErrorKind::CreatePoolConfig,
            AriesVcxCoreErrorKind::PoolLedgerConnect => AriesVcxErrorKind::PoolLedgerConnect,
            AriesVcxCoreErrorKind::InvalidLedgerResponse => {
                AriesVcxErrorKind::InvalidLedgerResponse
            }
            AriesVcxCoreErrorKind::LedgerItemNotFound => AriesVcxErrorKind::LedgerItemNotFound,
            AriesVcxCoreErrorKind::NoPoolOpen => AriesVcxErrorKind::NoPoolOpen,
            AriesVcxCoreErrorKind::PostMessageFailed => AriesVcxErrorKind::PostMessageFailed,
            AriesVcxCoreErrorKind::WalletCreate => AriesVcxErrorKind::WalletCreate,
            AriesVcxCoreErrorKind::WalletAccessFailed => AriesVcxErrorKind::WalletAccessFailed,
            AriesVcxCoreErrorKind::InvalidWalletHandle => AriesVcxErrorKind::InvalidWalletHandle,
            AriesVcxCoreErrorKind::DuplicationWallet => AriesVcxErrorKind::DuplicationWallet,
            AriesVcxCoreErrorKind::WalletRecordNotFound => AriesVcxErrorKind::WalletRecordNotFound,
            AriesVcxCoreErrorKind::DuplicationWalletRecord => {
                AriesVcxErrorKind::DuplicationWalletRecord
            }
            AriesVcxCoreErrorKind::WalletNotFound => AriesVcxErrorKind::WalletNotFound,
            AriesVcxCoreErrorKind::WalletAlreadyOpen => AriesVcxErrorKind::WalletAlreadyOpen,
            AriesVcxCoreErrorKind::DuplicationMasterSecret => {
                AriesVcxErrorKind::DuplicationMasterSecret
            }
            AriesVcxCoreErrorKind::DuplicationDid => AriesVcxErrorKind::DuplicationDid,
            AriesVcxCoreErrorKind::LoggingError => AriesVcxErrorKind::LoggingError,
            AriesVcxCoreErrorKind::EncodeError => AriesVcxErrorKind::EncodeError,
            AriesVcxCoreErrorKind::UnknownError => AriesVcxErrorKind::UnknownError,
            AriesVcxCoreErrorKind::InvalidDid => AriesVcxErrorKind::InvalidDid,
            AriesVcxCoreErrorKind::InvalidVerkey => AriesVcxErrorKind::InvalidVerkey,
            AriesVcxCoreErrorKind::InvalidNonce => AriesVcxErrorKind::InvalidNonce,
            AriesVcxCoreErrorKind::InvalidUrl => AriesVcxErrorKind::InvalidUrl,
            AriesVcxCoreErrorKind::SerializationError => AriesVcxErrorKind::SerializationError,
            AriesVcxCoreErrorKind::NotBase58 => AriesVcxErrorKind::NotBase58,
            AriesVcxCoreErrorKind::ParsingError => AriesVcxErrorKind::ParsingError,
            AriesVcxCoreErrorKind::InvalidHttpResponse => AriesVcxErrorKind::InvalidHttpResponse,
            AriesVcxCoreErrorKind::InvalidMessages => AriesVcxErrorKind::InvalidMessages,
            AriesVcxCoreErrorKind::VdrToolsError(u32) => AriesVcxErrorKind::VdrToolsError(u32),
            AriesVcxCoreErrorKind::UrsaError => AriesVcxErrorKind::UrsaError,
            AriesVcxCoreErrorKind::NoAgentInformation => AriesVcxErrorKind::NoAgentInformation,
            AriesVcxCoreErrorKind::InvalidMessageFormat => AriesVcxErrorKind::InvalidMessageFormat,
        };
        AriesVcxError::from_msg(kind, format!("AriesVcxCoreError: {}", err))
    }
}

// TODO
impl From<AriesVcxError> for AriesVcxCoreError {
    fn from(err: AriesVcxError) -> Self {
        let kind = match err.kind() {
            AriesVcxErrorKind::InvalidState => AriesVcxCoreErrorKind::InvalidState,
            AriesVcxErrorKind::InvalidConfiguration => AriesVcxCoreErrorKind::InvalidConfiguration,
            AriesVcxErrorKind::InvalidJson => AriesVcxCoreErrorKind::InvalidJson,
            AriesVcxErrorKind::InvalidOption => AriesVcxCoreErrorKind::InvalidOption,
            AriesVcxErrorKind::InvalidMessagePack => AriesVcxCoreErrorKind::InvalidMessagePack,
            AriesVcxErrorKind::NotReady => AriesVcxCoreErrorKind::NotReady,
            AriesVcxErrorKind::IOError => AriesVcxCoreErrorKind::IOError,
            AriesVcxErrorKind::LibindyInvalidStructure => {
                AriesVcxCoreErrorKind::LibindyInvalidStructure
            }
            AriesVcxErrorKind::InvalidLibindyParam => AriesVcxCoreErrorKind::InvalidLibindyParam,
            AriesVcxErrorKind::ActionNotSupported => AriesVcxCoreErrorKind::ActionNotSupported,
            AriesVcxErrorKind::InvalidInput => AriesVcxCoreErrorKind::InvalidInput,
            AriesVcxErrorKind::UnimplementedFeature => AriesVcxCoreErrorKind::UnimplementedFeature,
            AriesVcxErrorKind::CredDefAlreadyCreated => {
                AriesVcxCoreErrorKind::CredDefAlreadyCreated
            }
            AriesVcxErrorKind::RevDeltaNotFound => AriesVcxCoreErrorKind::RevDeltaNotFound,
            AriesVcxErrorKind::RevDeltaFailedToClear => {
                AriesVcxCoreErrorKind::RevDeltaFailedToClear
            }
            AriesVcxErrorKind::CreateRevRegDef => AriesVcxCoreErrorKind::CreateRevRegDef,
            AriesVcxErrorKind::InvalidRevocationDetails => {
                AriesVcxCoreErrorKind::InvalidRevocationDetails
            }
            AriesVcxErrorKind::InvalidRevocationEntry => {
                AriesVcxCoreErrorKind::InvalidRevocationEntry
            }
            AriesVcxErrorKind::InvalidRevocationTimestamp => {
                AriesVcxCoreErrorKind::InvalidRevocationTimestamp
            }
            AriesVcxErrorKind::RevRegDefNotFound => AriesVcxCoreErrorKind::RevRegDefNotFound,
            AriesVcxErrorKind::InvalidAttributesStructure => {
                AriesVcxCoreErrorKind::InvalidAttributesStructure
            }
            AriesVcxErrorKind::InvalidProof => AriesVcxCoreErrorKind::InvalidProof,
            AriesVcxErrorKind::InvalidSchema => AriesVcxCoreErrorKind::InvalidSchema,
            AriesVcxErrorKind::InvalidProofCredentialData => {
                AriesVcxCoreErrorKind::InvalidProofCredentialData
            }
            AriesVcxErrorKind::InvalidProofRequest => AriesVcxCoreErrorKind::InvalidProofRequest,
            AriesVcxErrorKind::ProofRejected => AriesVcxCoreErrorKind::ProofRejected,
            AriesVcxErrorKind::InvalidSchemaSeqNo => AriesVcxCoreErrorKind::InvalidSchemaSeqNo,
            AriesVcxErrorKind::DuplicationSchema => AriesVcxCoreErrorKind::DuplicationSchema,
            AriesVcxErrorKind::UnknownSchemaRejection => {
                AriesVcxCoreErrorKind::UnknownSchemaRejection
            }
            AriesVcxErrorKind::InvalidGenesisTxnPath => {
                AriesVcxCoreErrorKind::InvalidGenesisTxnPath
            }
            AriesVcxErrorKind::CreatePoolConfig => AriesVcxCoreErrorKind::CreatePoolConfig,
            AriesVcxErrorKind::PoolLedgerConnect => AriesVcxCoreErrorKind::PoolLedgerConnect,
            AriesVcxErrorKind::InvalidLedgerResponse => {
                AriesVcxCoreErrorKind::InvalidLedgerResponse
            }
            AriesVcxErrorKind::LedgerItemNotFound => AriesVcxCoreErrorKind::LedgerItemNotFound,
            AriesVcxErrorKind::NoPoolOpen => AriesVcxCoreErrorKind::NoPoolOpen,
            AriesVcxErrorKind::PostMessageFailed => AriesVcxCoreErrorKind::PostMessageFailed,
            AriesVcxErrorKind::WalletCreate => AriesVcxCoreErrorKind::WalletCreate,
            AriesVcxErrorKind::WalletAccessFailed => AriesVcxCoreErrorKind::WalletAccessFailed,
            AriesVcxErrorKind::InvalidWalletHandle => AriesVcxCoreErrorKind::InvalidWalletHandle,
            AriesVcxErrorKind::DuplicationWallet => AriesVcxCoreErrorKind::DuplicationWallet,
            AriesVcxErrorKind::WalletRecordNotFound => AriesVcxCoreErrorKind::WalletRecordNotFound,
            AriesVcxErrorKind::DuplicationWalletRecord => {
                AriesVcxCoreErrorKind::DuplicationWalletRecord
            }
            AriesVcxErrorKind::WalletNotFound => AriesVcxCoreErrorKind::WalletNotFound,
            AriesVcxErrorKind::WalletAlreadyOpen => AriesVcxCoreErrorKind::WalletAlreadyOpen,
            AriesVcxErrorKind::DuplicationMasterSecret => {
                AriesVcxCoreErrorKind::DuplicationMasterSecret
            }
            AriesVcxErrorKind::DuplicationDid => AriesVcxCoreErrorKind::DuplicationDid,
            AriesVcxErrorKind::LoggingError => AriesVcxCoreErrorKind::LoggingError,
            AriesVcxErrorKind::EncodeError => AriesVcxCoreErrorKind::EncodeError,
            AriesVcxErrorKind::UnknownError => AriesVcxCoreErrorKind::UnknownError,
            AriesVcxErrorKind::InvalidDid => AriesVcxCoreErrorKind::InvalidDid,
            AriesVcxErrorKind::InvalidVerkey => AriesVcxCoreErrorKind::InvalidVerkey,
            AriesVcxErrorKind::InvalidNonce => AriesVcxCoreErrorKind::InvalidNonce,
            AriesVcxErrorKind::InvalidUrl => AriesVcxCoreErrorKind::InvalidUrl,
            AriesVcxErrorKind::SerializationError => AriesVcxCoreErrorKind::SerializationError,
            AriesVcxErrorKind::NotBase58 => AriesVcxCoreErrorKind::NotBase58,
            AriesVcxErrorKind::ParsingError => AriesVcxCoreErrorKind::ParsingError,
            AriesVcxErrorKind::InvalidHttpResponse => AriesVcxCoreErrorKind::InvalidHttpResponse,
            AriesVcxErrorKind::InvalidMessages => AriesVcxCoreErrorKind::InvalidMessages,
            AriesVcxErrorKind::VdrToolsError(u32) => AriesVcxCoreErrorKind::VdrToolsError(u32),
            AriesVcxErrorKind::UrsaError => AriesVcxCoreErrorKind::UrsaError,
            AriesVcxErrorKind::NoAgentInformation => AriesVcxCoreErrorKind::NoAgentInformation,
            AriesVcxErrorKind::InvalidMessageFormat => AriesVcxCoreErrorKind::InvalidMessageFormat,
        };
        AriesVcxCoreError::from_msg(kind, format!("AriesVcxError: {}", err))
    }
}
