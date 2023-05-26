use std::convert::From;

use aries_vcx::errors::error::{AriesVcxError, AriesVcxErrorKind};
use aries_vcx_core::errors::error::AriesVcxCoreError;

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

// TODO
impl From<AriesVcxCoreError> for AgentError {
    fn from(err: AriesVcxCoreError) -> Self {
        let kind = AgentErrorKind::GenericAriesVcxError;
        let message = format!("AriesVcxCore Error; err: {:?}", err.to_string());
        AgentError { message, kind }
    }
}

impl From<diddoc_legacy::errors::error::DiddocError> for AgentError {
    fn from(err: diddoc_legacy::errors::error::DiddocError) -> Self {
        let kind = AgentErrorKind::GenericAriesVcxError;
        let message = format!("Diddoc Error; err: {:?}", err.to_string());
        AgentError { message, kind }
    }
}
