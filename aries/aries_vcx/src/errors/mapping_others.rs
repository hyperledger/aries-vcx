use std::{num::ParseIntError, string::FromUtf8Error, sync::PoisonError};

use base64::DecodeError;
use did_doc::schema::{types::uri::UriWrapperError, utils::error::DidDocumentLookupError};
use shared::errors::http_error::HttpError;
use url::ParseError;

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

impl From<did_parser_nom::ParseError> for AriesVcxError {
    fn from(err: did_parser_nom::ParseError) -> Self {
        AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, err.to_string())
    }
}

impl From<did_doc::error::DidDocumentBuilderError> for AriesVcxError {
    fn from(err: did_doc::error::DidDocumentBuilderError) -> Self {
        AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, err.to_string())
    }
}

impl From<DidDocumentLookupError> for AriesVcxError {
    fn from(err: DidDocumentLookupError) -> Self {
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

impl From<UriWrapperError> for AriesVcxError {
    fn from(err: UriWrapperError) -> Self {
        AriesVcxError::from_msg(AriesVcxErrorKind::InvalidInput, err.to_string())
    }
}

impl From<ParseIntError> for AriesVcxError {
    fn from(err: ParseIntError) -> Self {
        AriesVcxError::from_msg(AriesVcxErrorKind::InvalidInput, err.to_string())
    }
}

impl From<DecodeError> for AriesVcxError {
    fn from(err: DecodeError) -> Self {
        AriesVcxError::from_msg(AriesVcxErrorKind::InvalidInput, err.to_string())
    }
}

impl From<FromUtf8Error> for AriesVcxError {
    fn from(err: FromUtf8Error) -> Self {
        AriesVcxError::from_msg(AriesVcxErrorKind::InvalidInput, err.to_string())
    }
}

impl From<ParseError> for AriesVcxError {
    fn from(err: ParseError) -> Self {
        AriesVcxError::from_msg(AriesVcxErrorKind::InvalidInput, err.to_string())
    }
}
