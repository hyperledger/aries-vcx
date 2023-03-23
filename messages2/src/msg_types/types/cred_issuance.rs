use std::marker::PhantomData;

use derive_more::{From, TryInto};
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use super::Protocol;
use crate::msg_types::role::Role;

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, MessageType)]
#[msg_type(protocol = "issue-credential")]
pub enum CredentialIssuance {
    V1(CredentialIssuanceV1),
}

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(CredentialIssuance, Protocol))]
#[msg_type(major = 1)]
pub enum CredentialIssuanceV1 {
    #[msg_type(minor = 0, roles = "Role::Holder, Role::Issuer")]
    V1_0(PhantomData<fn() -> CredentialIssuanceV1_0>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum CredentialIssuanceV1_0 {
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
            Protocol::from(CredentialIssuanceV1::new_v1_0()),
            json!("https://didcomm.org/issue-credential/1.0"),
        )
    }

    #[test]
    fn test_version_resolution_issue_credential() {
        test_utils::test_msg_type_resolution("https://didcomm.org/issue-credential/1.255", CredentialIssuanceV1::new_v1_0())
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_issue_credential() {
        test_utils::test_serde(
            Protocol::from(CredentialIssuanceV1::new_v1_0()),
            json!("https://didcomm.org/issue-credential/2.0"),
        )
    }

    #[test]
    fn test_msg_type_offer() {
        test_utils::test_msg_type(
            "https://didcomm.org/issue-credential/1.0",
            "offer-credential",
            CredentialIssuanceV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_propose() {
        test_utils::test_msg_type(
            "https://didcomm.org/issue-credential/1.0",
            "propose-credential",
            CredentialIssuanceV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_request() {
        test_utils::test_msg_type(
            "https://didcomm.org/issue-credential/1.0",
            "request-credential",
            CredentialIssuanceV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_issue() {
        test_utils::test_msg_type(
            "https://didcomm.org/issue-credential/1.0",
            "issue-credential",
            CredentialIssuanceV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_preview() {
        test_utils::test_msg_type(
            "https://didcomm.org/issue-credential/1.0",
            "credential-preview",
            CredentialIssuanceV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_ack() {
        test_utils::test_msg_type(
            "https://didcomm.org/issue-credential/1.0",
            "ack",
            CredentialIssuanceV1::new_v1_0(),
        )
    }
}
