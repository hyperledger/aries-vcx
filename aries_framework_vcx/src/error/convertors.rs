use std::{convert::From, error::Error, num::ParseIntError};

use aries_vcx::{
    did_doc::error::DidDocumentBuilderError, errors::error::AriesVcxError,
    protocols::did_exchange::state_machine::generic::GenericDidExchange,
};

use crate::error::*;

impl From<AriesVcxError> for VCXFrameworkError {
    fn from(err: AriesVcxError) -> VCXFrameworkError {
        let kind = match err.kind() {
            // AriesVcxErrorKind::CredDefAlreadyCreated => VCXFrameworkErrorKind::CredDefAlreadyCreated,
            _ => VCXFrameworkErrorKind::GenericVCXFrameworkError,
        };
        error!("AriesVCX Error: {}", err.to_string());
        let message = format!("AriesVCX Error: {}", err);
        VCXFrameworkError { message, kind }
    }
}

impl From<serde_json::Error> for VCXFrameworkError {
    fn from(serde_err: serde_json::Error) -> VCXFrameworkError {
        let kind = VCXFrameworkErrorKind::SerializationError;
        let message = format!("(De)serialization failed; err: {:?}", serde_err.to_string());
        VCXFrameworkError { message, kind }
    }
}

impl From<DidDocumentBuilderError> for VCXFrameworkError {
    fn from(err: DidDocumentBuilderError) -> Self {
        let kind = VCXFrameworkErrorKind::GenericVCXFrameworkError;
        let message = format!("DidDocumentBuilderError; err: {:?}", err.to_string());
        VCXFrameworkError { message, kind }
    }
}

impl From<aries_vcx::did_parser_nom::ParseError> for VCXFrameworkError {
    fn from(err: aries_vcx::did_parser_nom::ParseError) -> Self {
        let kind = VCXFrameworkErrorKind::GenericVCXFrameworkError;
        let message = format!("DidParseError; err: {:?}", err.to_string());
        VCXFrameworkError { message, kind }
    }
}

impl From<did_peer::error::DidPeerError> for VCXFrameworkError {
    fn from(err: did_peer::error::DidPeerError) -> Self {
        let kind = VCXFrameworkErrorKind::GenericVCXFrameworkError;
        let message = format!("DidPeerError; err: {:?}", err.to_string());
        VCXFrameworkError { message, kind }
    }
}

impl From<(GenericDidExchange, AriesVcxError)> for VCXFrameworkError {
    fn from(err: (GenericDidExchange, AriesVcxError)) -> Self {
        let kind = VCXFrameworkErrorKind::GenericVCXFrameworkError;
        let message = format!("GenericDidExchange; err: {:?}", err.1.to_string());
        VCXFrameworkError { message, kind }
    }
}

impl From<ParseIntError> for VCXFrameworkError {
    fn from(err: ParseIntError) -> Self {
        let kind = VCXFrameworkErrorKind::GenericVCXFrameworkError;
        let message = format!("ParseIntError; err: {:?}", err.to_string());
        VCXFrameworkError { message, kind }
    }
}

impl From<Box<dyn Error + Send + Sync + 'static>> for VCXFrameworkError {
    fn from(err: Box<dyn Error + Send + Sync + 'static>) -> Self {
        let kind = VCXFrameworkErrorKind::GenericVCXFrameworkError;
        let message = format!("Generic VCXFramework Error; err: {:?}", err.to_string());
        VCXFrameworkError { message, kind }
    }
}
