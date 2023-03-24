use std::marker::PhantomData;

use derive_more::{From, TryInto};
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use super::Protocol;
use crate::msg_types::role::Role;

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, MessageType)]
#[msg_type(protocol = "issue-credential")]
pub enum CredentialIssuanceProtocol {
    V1(CredentialIssuanceProtocolV1),
}

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(CredentialIssuanceProtocol, Protocol))]
#[msg_type(major = 1)]
pub enum CredentialIssuanceProtocolV1 {
    #[msg_type(minor = 0, roles = "Role::Holder, Role::Issuer")]
    V1_0(PhantomData<fn() -> CredentialIssuanceProtocolV1_0>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum CredentialIssuanceProtocolV1_0 {
    OfferCredential,
    ProposeCredential,
    RequestCredential,
    IssueCredential,
    CredentialPreview,
    Ack,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils;

    #[test]
    fn test_protocol_issue_credential() {
        test_utils::test_serde(
            Protocol::from(CredentialIssuanceProtocolV1::new_v1_0()),
            json!("https://didcomm.org/issue-credential/1.0"),
        )
    }

    #[test]
    fn test_version_resolution_issue_credential() {
        test_utils::test_msg_type_resolution(
            "https://didcomm.org/issue-credential/1.255",
            CredentialIssuanceProtocolV1::new_v1_0(),
        )
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_issue_credential() {
        test_utils::test_serde(
            Protocol::from(CredentialIssuanceProtocolV1::new_v1_0()),
            json!("https://didcomm.org/issue-credential/2.0"),
        )
    }

    #[test]
    fn test_msg_type_offer() {
        test_utils::test_msg_type(
            "https://didcomm.org/issue-credential/1.0",
            "offer-credential",
            CredentialIssuanceProtocolV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_propose() {
        test_utils::test_msg_type(
            "https://didcomm.org/issue-credential/1.0",
            "propose-credential",
            CredentialIssuanceProtocolV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_request() {
        test_utils::test_msg_type(
            "https://didcomm.org/issue-credential/1.0",
            "request-credential",
            CredentialIssuanceProtocolV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_issue() {
        test_utils::test_msg_type(
            "https://didcomm.org/issue-credential/1.0",
            "issue-credential",
            CredentialIssuanceProtocolV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_preview() {
        test_utils::test_msg_type(
            "https://didcomm.org/issue-credential/1.0",
            "credential-preview",
            CredentialIssuanceProtocolV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_ack() {
        test_utils::test_msg_type(
            "https://didcomm.org/issue-credential/1.0",
            "ack",
            CredentialIssuanceProtocolV1::new_v1_0(),
        )
    }
}
