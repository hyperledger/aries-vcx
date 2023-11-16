use crate::error::AgentErrorKind;

#[derive(Debug)]
pub struct AgentError {
    pub message: String,
    pub kind: AgentErrorKind,
}

impl std::fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        f.write_str(&self.kind.to_string())
    }
}

impl AgentError {
    pub fn from_msg(kind: AgentErrorKind, msg: &str) -> Self {
        AgentError {
            kind,
            message: msg.to_string(),
        }
    }

    pub fn from_kind(kind: AgentErrorKind) -> Self {
        let message = kind.to_string();
        AgentError { kind, message }
    }
}
