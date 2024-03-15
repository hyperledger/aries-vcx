use std::{convert::From, num::ParseIntError};

use aries_vcx::{
    did_doc::error::DidDocumentBuilderError,
    errors::error::{AriesVcxError, AriesVcxErrorKind},
    protocols::did_exchange::state_machine::generic::GenericDidExchange,
};
use aries_vcx_core::errors::error::AriesVcxCoreError;
use did_resolver_sov::did_resolver::did_doc::schema::utils::error::DidDocumentLookupError;

use crate::error::*;

impl From<AriesVcxError> for AgentError {
    fn from(err: AriesVcxError) -> AgentError {
        let kind = match err.kind() {
            AriesVcxErrorKind::CredDefAlreadyCreated => AgentErrorKind::CredDefAlreadyCreated,
            _ => AgentErrorKind::GenericAriesVcxError,
        };
        error!("AriesVCX Error: {}", err.to_string());
        let message = format!("AriesVCX Error: {}", err);
        AgentError { message, kind }
    }
}

impl From<serde_json::Error> for AgentError {
    fn from(serde_err: serde_json::Error) -> AgentError {
        let kind = AgentErrorKind::SerializationError;
        let message = format!("(De)serialization failed; err: {:?}", serde_err.to_string());
        AgentError { message, kind }
    }
}

impl From<AriesVcxCoreError> for AgentError {
    fn from(err: AriesVcxCoreError) -> Self {
        let kind = AgentErrorKind::GenericAriesVcxError;
        let message = format!("AriesVcxCore Error; err: {:?}", err.to_string());
        AgentError { message, kind }
    }
}

impl From<DidDocumentBuilderError> for AgentError {
    fn from(err: DidDocumentBuilderError) -> Self {
        let kind = AgentErrorKind::GenericAriesVcxError;
        let message = format!("DidDocumentBuilderError; err: {:?}", err.to_string());
        AgentError { message, kind }
    }
}

impl From<aries_vcx::did_parser::ParseError> for AgentError {
    fn from(err: aries_vcx::did_parser::ParseError) -> Self {
        let kind = AgentErrorKind::GenericAriesVcxError;
        let message = format!("DidParseError; err: {:?}", err.to_string());
        AgentError { message, kind }
    }
}

impl From<did_peer::error::DidPeerError> for AgentError {
    fn from(err: did_peer::error::DidPeerError) -> Self {
        let kind = AgentErrorKind::GenericAriesVcxError;
        let message = format!("DidPeerError; err: {:?}", err.to_string());
        AgentError { message, kind }
    }
}

impl From<public_key::PublicKeyError> for AgentError {
    fn from(err: public_key::PublicKeyError) -> Self {
        let kind = AgentErrorKind::GenericAriesVcxError;
        let message = format!("PublicKeyError; err: {:?}", err.to_string());
        AgentError { message, kind }
    }
}

impl From<did_key::error::DidKeyError> for AgentError {
    fn from(err: did_key::error::DidKeyError) -> Self {
        let kind = AgentErrorKind::GenericAriesVcxError;
        let message = format!("DidKeyError; err: {:?}", err.to_string());
        AgentError { message, kind }
    }
}

impl From<(GenericDidExchange, AriesVcxError)> for AgentError {
    fn from(err: (GenericDidExchange, AriesVcxError)) -> Self {
        let kind = AgentErrorKind::GenericAriesVcxError;
        let message = format!("GenericDidExchange; err: {:?}", err.1.to_string());
        AgentError { message, kind }
    }
}
impl From<DidDocumentLookupError> for AgentError {
    fn from(err: DidDocumentLookupError) -> Self {
        let kind = AgentErrorKind::GenericAriesVcxError;
        let message = format!("DidDocumentLookupError; err: {:?}", err.to_string());
        AgentError { message, kind }
    }
}

impl From<anoncreds_types::Error> for AgentError {
    fn from(err: anoncreds_types::Error) -> Self {
        let kind = AgentErrorKind::GenericAriesVcxError;
        let message = format!("AnoncredsTypesError; err: {:?}", err.to_string());
        AgentError { message, kind }
    }
}

impl From<ParseIntError> for AgentError {
    fn from(err: ParseIntError) -> Self {
        let kind = AgentErrorKind::GenericAriesVcxError;
        let message = format!("ParseIntError; err: {:?}", err.to_string());
        AgentError { message, kind }
    }
}
