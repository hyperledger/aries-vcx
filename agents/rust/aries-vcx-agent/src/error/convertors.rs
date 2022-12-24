use std::convert::From;

use aries_vcx::errors::error::{AriesVcxError, AriesVcxErrorKind};

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
