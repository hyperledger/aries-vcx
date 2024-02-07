use std::{num::ParseIntError, sync::PoisonError};

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

impl From<anoncreds_types::Error> for AriesVcxError {
    fn from(err: anoncreds_types::Error) -> Self {
        AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, err.to_string())
    }
}

impl From<ParseIntError> for AriesVcxError {
    fn from(err: ParseIntError) -> Self {
        AriesVcxError::from_msg(AriesVcxErrorKind::InvalidInput, err.to_string())
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
            AriesVcxCoreErrorKind::WalletError => AriesVcxErrorKind::WalletError,
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
