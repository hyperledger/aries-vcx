use derive_more::{From, TryInto};
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use super::{
    traits::{MajorVersion, MinorVersion, ProtocolName},
    Protocol,
};
use crate::msg_types::{actor::Actor, registry::get_supported_version};

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

#[cfg(test)]
mod tests {
    use crate::misc::test_utils;

    use super::CredentialIssuanceV1_0;

    const PROTOCOL: &str = "https://didcomm.org/issue-credential/1.0";
    const VERSION_RESOLUTION_PROTOCOL: &str = "https://didcomm.org/issue-credential/1.255";
    const UNSUPPORTED_VERSION_PROTOCOL: &str = "https://didcomm.org/issue-credential/2.0";

    const KIND_OFFER: &str = "offer-credential";
    const KIND_PROPOSE: &str = "propose-credential";
    const KIND_REQUEST: &str = "request-credential";
    const KIND_ISSUE: &str = "issue-credential";
    const KIND_PREVIEW: &str = "credential-preview";
    const KIND_ACK: &str = "ack";

    #[test]
    fn test_protocol_issue_credential() {
        test_utils::test_protocol(PROTOCOL, CredentialIssuanceV1_0)
    }

    #[test]
    fn test_version_resolution_issue_credential() {
        test_utils::test_protocol(VERSION_RESOLUTION_PROTOCOL, CredentialIssuanceV1_0)
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_issue_credential() {
        test_utils::test_protocol(UNSUPPORTED_VERSION_PROTOCOL, CredentialIssuanceV1_0)
    }

    #[test]
    fn test_msg_type_offer() {
        test_utils::test_msg_type(PROTOCOL, KIND_OFFER, CredentialIssuanceV1_0)
    }

    #[test]
    fn test_msg_type_propose() {
        test_utils::test_msg_type(PROTOCOL, KIND_PROPOSE, CredentialIssuanceV1_0)
    }

    #[test]
    fn test_msg_type_request() {
        test_utils::test_msg_type(PROTOCOL, KIND_REQUEST, CredentialIssuanceV1_0)
    }

    #[test]
    fn test_msg_type_issue() {
        test_utils::test_msg_type(PROTOCOL, KIND_ISSUE, CredentialIssuanceV1_0)
    }

    #[test]
    fn test_msg_type_preview() {
        test_utils::test_msg_type(PROTOCOL, KIND_PREVIEW, CredentialIssuanceV1_0)
    }

    #[test]
    fn test_msg_type_ack() {
        test_utils::test_msg_type(PROTOCOL, KIND_ACK, CredentialIssuanceV1_0)
    }
}
