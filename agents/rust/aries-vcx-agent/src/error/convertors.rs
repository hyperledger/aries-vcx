use std::convert::From;

use aries_vcx::errors::error::{ErrorAriesVcx, ErrorKindAriesVcx};

use crate::error::*;

impl From<ErrorAriesVcx> for AgentError {
    fn from(err: ErrorAriesVcx) -> AgentError {
        let kind = match err.kind() {
            ErrorKindAriesVcx::CredDefAlreadyCreated => AgentErrorKind::CredDefAlreadyCreated,
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
