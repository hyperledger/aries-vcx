use derive_more::From;
use messages_macros::TransitiveFrom;
use strum_macros::{AsRefStr, EnumString};

use crate::error::{MsgTypeError, MsgTypeResult};

use super::{
    traits::{ResolveMajorVersion, ResolveMinorVersion, ResolveMsgKind},
    MessageFamily,
};

#[derive(Copy, Clone, Debug, From, PartialEq)]
pub enum CredentialIssuance {
    V1(CredentialIssuanceV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom)]
#[transitive(CredentialIssuance, MessageFamily)]
pub enum CredentialIssuanceV1 {
    V1_0(CredentialIssuanceV1_0),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, TransitiveFrom)]
#[transitive(CredentialIssuanceV1, CredentialIssuance, MessageFamily)]
#[strum(serialize_all = "kebab-case")]
pub enum CredentialIssuanceV1_0 {
    OfferCredential,
    ProposeCredential,
    RequestCredential,
    IssueCredential,
    Ack,
}

impl ResolveMsgKind for CredentialIssuanceV1_0 {
    const MINOR: u8 = 0;
}

impl ResolveMinorVersion for CredentialIssuanceV1 {
    const MAJOR: u8 = 1;

    fn resolve_minor_ver(minor: u8, kind: &str) -> MsgTypeResult<Self> {
        match minor {
            CredentialIssuanceV1_0::MINOR => Ok(Self::V1_0(CredentialIssuanceV1_0::resolve_kind(kind)?)),
            _ => Err(MsgTypeError::minor_ver_err(minor)),
        }
    }

    fn as_full_ver_parts(&self) -> (u8, u8, &str) {
        let (minor, kind) = match self {
            Self::V1_0(v) => v.as_minor_ver_parts(),
        };
        (Self::MAJOR, minor, kind)
    }
}

impl ResolveMajorVersion for CredentialIssuance {
    const FAMILY: &'static str = "issue-credential";

    fn resolve_major_ver(major: u8, minor: u8, kind: &str) -> MsgTypeResult<Self> {
        match major {
            CredentialIssuanceV1::MAJOR => Ok(Self::V1(CredentialIssuanceV1::resolve_minor_ver(minor, kind)?)),
            _ => Err(MsgTypeError::major_ver_err(major)),
        }
    }

    fn as_msg_type_parts(&self) -> (&str, u8, u8, &str) {
        let (major, minor, kind) = match self {
            Self::V1(v) => v.as_full_ver_parts(),
        };

        (Self::FAMILY, major, minor, kind)
    }
}
