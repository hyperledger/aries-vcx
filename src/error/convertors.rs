use std::convert::From;

use aries_vcx::error::VcxError;

use crate::error::*;

impl From<VcxError> for AgentError {
    fn from(err: VcxError) -> AgentError {
        let kind = AgentErrorKind::GenericAriesVcxError;
        let message = format!("AriesVCX Error: {}", err.to_string());
        AgentError { message, kind }
    }
}
