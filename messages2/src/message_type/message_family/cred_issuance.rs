use derive_more::{From, TryInto};
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveInto;

use crate::{
    error::{MsgTypeError, MsgTypeResult},
    message_type::MessageType,
};

use super::{
    traits::{ResolveMajorVersion, ResolveMinorVersion, ResolveMsgKind},
    MessageFamily,
};

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, TransitiveInto, MessageType)]
#[transitive(MessageFamily, MessageType)]
#[semver(family = "issue-credential")]
pub enum CredentialIssuance {
    V1(CredentialIssuanceV1),
}

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, TransitiveInto, MessageType)]
#[transitive(all(CredentialIssuance, MessageFamily, MessageType))]
#[semver(major = 1)]
pub enum CredentialIssuanceV1 {
    V1_0(CredentialIssuanceV1_0),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, TransitiveInto, MessageType)]
#[transitive(all(CredentialIssuanceV1, CredentialIssuance, MessageFamily, MessageType))]
#[strum(serialize_all = "kebab-case")]
#[semver(minor = 0)]
pub enum CredentialIssuanceV1_0 {
    OfferCredential,
    ProposeCredential,
    RequestCredential,
    IssueCredential,
    CredentialPreview,
    Ack,
}
