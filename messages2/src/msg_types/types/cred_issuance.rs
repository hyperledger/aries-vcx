use derive_more::{From, TryInto};
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use crate::{
    error::{MsgTypeError, MsgTypeResult},
    msg_types::actor::Actor,
    msg_types::registry::get_supported_version,
};

use super::{
    traits::{MajorVersion, MessageKind, MinorVersion, ProtocolName},
    Protocol,
};

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, MessageType)]
#[semver(protocol = "issue-credential")]
pub enum CredentialIssuance {
    V1(CredentialIssuanceV1),
}

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(CredentialIssuance, Protocol)))]
#[semver(major = 1, parent = "CredentialIssuance", actors(Actor::Holder, Actor::Issuer))]
pub enum CredentialIssuanceV1 {
    V1_0(CredentialIssuanceV1_0),
}

#[derive(Copy, Clone, Debug, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(CredentialIssuanceV1, CredentialIssuance, Protocol)))]
#[semver(minor = 0, parent = "CredentialIssuanceV1")]
pub struct CredentialIssuanceV1_0;

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, MessageType)]
#[strum(serialize_all = "kebab-case")]
#[semver(parent = "CredentialIssuanceV1_0")]
pub enum CredentialIssuanceV1_0Kind {
    OfferCredential,
    ProposeCredential,
    RequestCredential,
    IssueCredential,
    CredentialPreview,
    Ack,
}
