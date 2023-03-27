use std::num::ParseIntError;

use thiserror::Error as ThisError;

pub type MsgTypeResult<T> = Result<T, MsgTypeError>;

#[derive(Debug, ThisError)]
pub enum MsgTypeError {
    #[error("Unknown message type prefix: {0}")]
    UnknownPrefix(String),
    #[error("Unknown message kind: {0}")]
    UnknownMsgKind(String),
    #[error("Unsupported protocol minor version: {0}")]
    UnsupportedMinorVer(u8),
    #[error("Unsupported protocol major version: {0}")]
    UnsupportedMajorVer(u8),
    #[error("Unknown protocol name: {0}")]
    UnknownProtocol(String),
    #[error("Error parsing version: {0}")]
    InvalidVersion(#[from] ParseIntError),
    #[error("No {0} found in the message type")]
    PartNotFound(&'static str),
}

impl MsgTypeError {
    pub fn unknown_prefix(prefix: String) -> Self {
        Self::UnknownPrefix(prefix)
    }

    pub fn unknown_kind(kind: String) -> Self {
        Self::UnknownMsgKind(kind)
    }

    pub fn minor_ver_err(minor: u8) -> Self {
        Self::UnsupportedMinorVer(minor)
    }

    pub fn major_ver_err(major: u8) -> Self {
        Self::UnsupportedMajorVer(major)
    }

    pub fn unknown_protocol(name: String) -> Self {
        Self::UnknownProtocol(name)
    }

    pub fn not_found(part: &'static str) -> Self {
        Self::PartNotFound(part)
    }
}
